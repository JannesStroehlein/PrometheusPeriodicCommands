mod cli;
mod config;
mod prometheus;
mod shell_commands;

use crate::cli::CliArgs;
use crate::config::read_cfg;
use crate::config::schema::{CommandTarget, Schema};
use crate::prometheus::Metrics;
use crate::shell_commands::ShellCommand;
use actix_web::middleware::Compress;
use actix_web::web::Data;
use actix_web::{web, App, HttpResponse, HttpServer, Result};
use clap::Parser;
use log::{info, trace, warn};
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::family::Family;
use prometheus_client::registry::Registry;
use regex::Regex;
use simple_logger::SimpleLogger;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::{io, thread};
use tokio::runtime;

// Taken from the Prometheus sample code
pub struct AppState {
    pub registry: Registry,
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    // Init simple log
    SimpleLogger::new().init().unwrap();

    let cli_args = CliArgs::parse();

    // Read the config
    let config: Arc<Schema> = match read_cfg(&cli_args) {
        Ok(s) => Arc::new(s),
        Err(x) => panic!("Could not read config: {x}"),
    };
    trace!("Parsed config data: {config:?}");

    let metrics = Data::new(Metrics {
        last_result: Family::default(),
        last_duration: Family::default(),
    });

    let mut state = AppState {
        registry: Registry::default(),
    };

    state.registry.register(
        "last_result",
        "The last parsed result of a command target command",
        metrics.last_result.clone(),
    );

    state.registry.register(
        "last_duration",
        "Number of milliseconds the last command execution took",
        metrics.last_duration.clone(),
    );

    let state = Mutex::new(state);
    let state_data = Data::new(state);

    info!("Starting Web Server on {}:{}", config.host, config.port);

    match start_tasks_worker(metrics.clone().into_inner(), config.clone()) {
        Ok(_) => info!("Started background thread"),
        Err(x) => panic!("Could not start background thread {x}"),
    }

    HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            .app_data(metrics.clone())
            .app_data(state_data.clone())
            .service(web::resource("/metrics").route(web::get().to(metrics_handler)))
    })
    .bind((config.host.to_owned(), config.port))?
    .run()
    .await
}

/// This method is called when the /metrics endpoint is requested. Respond with the prometheus metrics
pub async fn metrics_handler(state: Data<Mutex<AppState>>) -> Result<HttpResponse> {
    let state = state.lock().unwrap();
    let mut body = String::new();
    encode(&mut body, &state.registry).unwrap();
    Ok(HttpResponse::Ok()
        .content_type("application/openmetrics-text; version=1.0.0; charset=utf-8")
        .body(body))
}

/// Starts the thread that handles the command targets that are defined in the config file
fn start_tasks_worker(state: Arc<Metrics>, config: Arc<Schema>) -> io::Result<()> {
    match thread::Builder::new()
        .name("Target Runner Background task".to_string())
        .spawn(move || tasks_worker(state, config.clone()))
    {
        Err(x) => Err(x),
        Ok(_) => Ok(()),
    }
}

/// The actual code that runs in the command runner thread
///
/// The goal is to run the commands in a "ThreadPool" thread indefinitely with
/// the interval between executions that is specified in the config file
fn tasks_worker(state: Arc<Metrics>, config: Arc<Schema>) {
    let threaded_rt = runtime::Builder::new_multi_thread()
        .enable_io()
        .build()
        .unwrap();

    // Create a map of the indices
    let mut timers: HashMap<usize, Duration> = Default::default();

    for (index, target) in config.targets.iter().enumerate() {
        timers.insert(index, target.run_every.into());
    }

    // 🤡
    let config_deref = config.clone();
    loop {
        // Find the next timer to wait for
        let next_timer = { timers.iter().min_by_key(|&(_, &duration)| duration) };

        let (index, duration) = match next_timer {
            Some((index, duration)) => (*index, duration),
            None => continue,
        };

        let target: CommandTarget = config_deref.targets[index].clone();

        trace!("Waiting for next target '{}'", target.command);
        // Sleep for the duration
        sleep(*duration);
        trace!("Finished waiting for target '{}'", target.command);

        // Clone the state variables so that they can be moved into the "ThreadPool" thread
        let thread_state = state.clone();
        let thread_target = target.clone();
        threaded_rt.spawn(async move {
            match handle_command_target(&thread_state, &thread_target).await {
                Ok(_) => trace!("Command executed successfully"),
                Err(e) => warn!("Command execution failed {e}"),
            };
        });

        // Deduct the time slept from all durations and set the duration of the timer we waited for
        // back to its config value
        let duration_copy = *duration;
        {
            let timer_values_mut = timers.values_mut();

            for value in timer_values_mut {
                *value -= duration_copy;
            }
        }

        let target_interval: Duration = target.run_every.clone().into();
        let current_duration = timers.get_mut(&index).unwrap();
        *current_duration = target_interval;
        trace!(
            "Reset target '{}' to duration {}",
            target.command,
            target.run_every
        );
    }
}

/// This method is called for each command target when the timer ticks
/// The command is executed and the output interpreted
async fn handle_command_target(state: &Arc<Metrics>, target: &CommandTarget) -> Result<(), String> {
    let regex = match Regex::new(&format!(r"(?m){}", &*target.regex)) {
        Ok(x) => x,
        Err(err) => return Err(err.to_string()),
    };

    info!("Handling command '{}'", target.command);

    let cmd = ShellCommand::new(&*target.command, "");

    let execution_result = cmd.execute().await;

    match execution_result.0 {
        Ok(x) => {
            let stdout_str = String::from_utf8(x.stdout).unwrap();

            // Simply parse the stdout to a f64 and return that or explode trying
            match regex.captures(stdout_str.trim()) {
                Some(caps) => {
                    let cap = caps
                        .name(&*target.regex_named_group)
                        .map_or("", |m| m.as_str());

                    match cap.parse::<f64>() {
                        Err(_) => Err(format!(
                            "Could not parse capture to f64.\nCaptures: {caps:?}\nStdout:{stdout_str}"
                        )),
                        Ok(c) => {
                            state.update_result(&target, x.status.code().unwrap(), c);
                            state.update_duration(&target, &execution_result.1);
                            Ok(())
                        }
                    }
                }
                None => Err("RegEx did not find any captures in stdout".to_string()),
            }
        }
        Err(_) => Err("Error executing command".to_string()),
    }
}
