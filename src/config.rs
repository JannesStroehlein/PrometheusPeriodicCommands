use std::fs::File;
use std::io::Read;
use std::path::Path;

pub mod Schema;

const CONFIG_FILE : &str = "G:\\Rust\\PrometheusPeriodicCommands\\config.yaml";

pub fn read_cfg() -> Result<Schema::Schema, String> {
    if !Path::new(CONFIG_FILE).exists() {
        return Err("The config file could not be found".parse().unwrap());
    }

    let mut file = match File::open(CONFIG_FILE) {
        Err(e) => return Err(e.to_string()),
        Ok(x) => x
    };

    let mut file_str = String::new();

    match file.read_to_string(&mut file_str){
        Err(e) => return Err(e.to_string()),
        Ok(_) => {}
    };

    let read_config : Schema::Schema = match serde_yaml::from_str(&file_str) {
        Ok(x) => x,
        Err(err) => return Err(err.to_string())
    };
    Ok(read_config)
}