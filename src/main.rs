mod shell_commands;
mod prometheus;
mod config;

use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use actix_web::middleware::Compress;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, Result};
use actix_web::web::Data;
use log::{debug, info, trace, warn};
use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};
use prometheus_client::metrics::family::Family;
use prometheus_client::registry::Registry;
use simple_logger::SimpleLogger;
use crate::config::read_cfg;
use crate::config::schema::{CommandTarget, Schema};
use crate::prometheus::Metrics;
use crate::shell_commands::ShellCommand;

// Taken from the Prometheus sample code
pub struct AppState {
    pub registry: Registry,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Init simple log
    SimpleLogger::new().init().unwrap();

    // Read the config
    let config : Arc<Schema>= match read_cfg() {
        Ok(s) => Arc::new(s),
        Err(x) => panic!("Could not read config: {x}")
    };
    trace!("Parsed config data: {config:?}");

    let metrics = Data::new(Metrics {
        last_output: Family::default(),
    });

    let mut state = AppState {
        registry: Registry::default()
    };

    state
        .registry
        .register("last_output", "The last output of the command", metrics.last_output.clone());

    let state = Mutex::new(state);
    let state_data = Data::new(state);

    info!("Starting Web Server on {}:{}", config.host, config.port);

    match start_tasks_worker(metrics.clone().into_inner(), config.clone()) {
        Ok(s) => info!("Started background thread"),
        Err(x) => panic!("Could not start background thread {x}")
    }

    HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            .app_data(metrics.clone())
            .app_data(state_data.clone())
            .service(web::resource("/metrics").route(web::get().to(metrics_handler)))
    })
        .bind(((config.host.to_owned()), config.port))?
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
fn start_tasks_worker(state : Arc<Metrics>, config: Arc<Schema>) -> Result<thread::JoinHandle<()>, String>{
    match thread::Builder::new()
        .name("Target Runner Background task".to_string())
        .spawn(move || tasks_worker(state, config.clone()))
    {
        Err(x) => Err(x.to_string()),
        Ok(h) => Ok(h)
    }
}

/// The actual code that runs in the command runner thread
///
/// The goal is to run the commands in a "ThreadPool" thread indefinitely with
/// the interval between executions that is specified in the config file
fn tasks_worker(state : Arc<Metrics>, config: Arc<Schema>) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build().unwrap();

    // Create a map of the indices
    let mut timers : HashMap<usize, Duration> = Default::default();

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
            None => continue
        };

        let target : CommandTarget = config_deref.targets[index].clone();

        debug!("Waiting for next target '{}'", target.command);
        // Sleep for the duration
        sleep(*duration);
        debug!("Finished waiting for target '{}'", target.command);

        // Clone the state variables so that they can be moved into the "ThreadPool" thread
        let thread_state = state.clone();
        let thread_target = target.clone();
        rt.spawn(async move {
            match handle_command_target(&thread_state, &thread_target) {
                Ok(_) => info!("Command executed successfully"),
                Err(e) => warn!("Command execution failed {e}")
            };
        });

        // Deduct the time slept from all durations and set the duration of the timer we waited for
        // back to its config value
        let duration_copy = *duration;

        {
            let mut timer_values_mut = timers.values_mut();

            for value in timer_values_mut {
                *value -= duration_copy;
            }
        }

        let target_interval : Duration = target.run_every.clone().into();
        let mut current_duration = timers.get_mut(&index).unwrap();
        *current_duration = target_interval;
        debug!("Reset target {} to duration {}", target.command, target.run_every);
    }
}

/// This method is called for each command target when the timer ticks
/// The command is executed and the output interpreted (TODO: implement RegEx logic)
fn handle_command_target(state : &Arc<Metrics>, target: &CommandTarget) -> Result<(), String> {
    info!("Handling command {}", target.command);

    let cmd = ShellCommand::new(&*target.command, "");

    match cmd.execute() {
        Ok(x) => Ok(state.update_requests("echo 2", x.status.code().unwrap(), {
            let stdout_str = String::from_utf8(x.stdout).unwrap();
            // Simply parse the stdout to an i64 and return that or explode trying
            stdout_str.trim().parse::<i64>().unwrap()
        })),
        Err(_) => Err("Error executing command".to_string())
    }
}


