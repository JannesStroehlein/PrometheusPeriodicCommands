mod shell_commands;
mod prometheus;
mod config;

use std::ptr::null;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use actix_web::middleware::Compress;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, Result};
use actix_web::cookie::time::format_description::well_known::iso8601::Config;
use actix_web::rt::task::JoinHandle;
use actix_web::web::Data;
use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};
use prometheus_client::metrics::family::Family;
use prometheus_client::registry::Registry;
use crate::config::read_cfg;
use crate::config::Schema::{CommandTarget, Schema};
use crate::prometheus::Metrics;
use crate::shell_commands::ShellCommand;

pub struct AppState {
    pub registry: Registry,
}

const BIND_ADDR : &str = "127.0.0.1";
const BIND_PORT : u16 = 8080;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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

    println!("Starting Web Server on {BIND_ADDR}:{BIND_PORT}");

    let config = match read_cfg() {
        Ok(s) => s,
        Err(x) => panic!("Could not read config: {x}")
    };
    println!("{:?}", config);

    match start_tasks_worker(metrics.clone().into_inner(), config) {
        Ok(s) => println!("Started background thread"),
        Err(x) => panic!("Could not start background thread {x}")
    }

    HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            .app_data(metrics.clone())
            .app_data(state_data.clone())
            .service(web::resource("/metrics").route(web::get().to(metrics_handler)))
    })
        .bind((BIND_ADDR, BIND_PORT))?
        .run()
        .await
}

pub async fn metrics_handler(state: web::Data<Mutex<AppState>>) -> Result<HttpResponse> {
    let state = state.lock().unwrap();
    let mut body = String::new();
    encode(&mut body, &state.registry).unwrap();
    Ok(HttpResponse::Ok()
        .content_type("application/openmetrics-text; version=1.0.0; charset=utf-8")
        .body(body))
}


fn start_tasks_worker(state : Arc<Metrics>, config: Schema) -> Result<thread::JoinHandle<()>, String>{

    let handle = match thread::Builder::new()
        .name("Target Runner Background task".to_string())
        .spawn(move || tasks_worker(state, config.clone())){
        Err(x) => return Err(x.to_string()),
        Ok(h) => h
    };

    Ok(handle)
}
fn tasks_worker(state : Arc<Metrics>, config: Schema) {
    loop {
        for t in config.targets.clone() {
            sleep(t.run_every.into());

            handle_target(&state, &t).unwrap()
        }

    }
}

fn handle_target(state : &Arc<Metrics>, target: &CommandTarget) -> Result<(), String> {
    println!("Handling command {}", target.command);
    let cmd = ShellCommand::new(&*target.command, "");
    match cmd.execute() {
        Ok(x) => Ok(state.update_requests("echo 2", x.status.code().unwrap(), {
            let stdout_str = String::from_utf8(x.stdout).unwrap();
            stdout_str.trim().parse::<i64>().unwrap()
        })),
        Err(_) => Err("Error executing command".to_string())
    }
}


