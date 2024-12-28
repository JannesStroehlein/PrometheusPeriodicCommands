use crate::cli::CliArgs;
use log::{debug, error, info, warn};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub mod schema;

const CONFIG_FILE_DIR_NAME: &str = "prometheus_periodic_commands";

/// Attempts to find the config file in one of the OS specific paths.
///
/// ### Cli Args
/// If the config path is specified in the cli arguments, it will have priority over
/// the OS specific paths.
/// Any cli overrides (host, port) will be applied to the config read from the file before it
/// is returned by this function.
pub fn read_cfg(cli_args: &CliArgs) -> Result<schema::Schema, String> {
    let found_config_path = {
        if cli_args.config_file == None {
            explore_config_file_paths()
        } else {
            cli_args.config_file.clone().unwrap()
        }
    };

    if !Path::new(&found_config_path).exists() {
        error!("The specified config path does not point to a file.");
        return Err("Config file could be found".to_string());
    }

    info!("Loading config file: '{found_config_path}'");

    let mut file = match File::open(found_config_path) {
        Err(e) => return Err(e.to_string()),
        Ok(x) => x,
    };

    // Read the file to a Vec<u8>
    let mut file_buf: Vec<u8> = vec![];
    match file.read_to_end(&mut file_buf) {
        Ok(size) => debug!("Read {size} bytes from the config file"),
        Err(e) => return Err(e.to_string()),
    };

    if file_buf.starts_with(&[0xEF, 0xBB, 0xBF]) {
        // Remove the first three bytes (the BOM)
        file_buf = file_buf[3..].to_vec();
        warn!("Stripped the BOM of the config file");
    }

    let file_str = String::from_utf8(file_buf).expect("Could not read the file contents");

    let mut read_config: schema::Schema = match serde_yaml::from_str(&file_str) {
        Ok(x) => x,
        Err(err) => return Err(err.to_string()),
    };

    let config_host = read_config.host.clone();
    let config_port = read_config.port.clone();

    read_config.host = match cli_args.host.clone() {
        None => config_host,
        Some(cli_host) => {
            debug!("Config host value was overridden by cli argument");
            cli_host
        }
    };

    read_config.port = match cli_args.port {
        None => config_port,
        Some(cli_port) => {
            debug!("Config port value was overridden by cli argument");
            cli_port
        }
    };

    Ok(read_config)
}

/// Test for common file paths to find a config file
fn explore_config_file_paths() -> String {
    let config_file_names = [String::from("config.yaml")];

    #[cfg(target_os = "linux")]
    let os_specific_config_dirs = [
        String::from(""),
        format!("/etc/{CONFIG_FILE_DIR_NAME}"),
        format!("~/.config/{CONFIG_FILE_DIR_NAME}"),
    ];

    #[cfg(target_os = "windows")]
    let os_specific_config_dirs = [
        format!("~\\AppData\\Local\\{CONFIG_FILE_DIR_NAME}"),
        String::from(""),
    ];

    // check current dir
    for possible_config_path in os_specific_config_dirs.clone() {
        let expanded_path = shellexpand::full(&possible_config_path).expect(&format!(
            "Could not expand the config path: {possible_config_path}"
        ));

        let expanded_path_str = expanded_path.to_string();
        debug!("Checking dir: {expanded_path_str}");

        let config_dir_path = Path::new(&expanded_path_str);

        if !&config_dir_path.exists() {
            debug!("Path {expanded_path} does not exist.");
            continue;
        }

        // Check if a file with a valid config file name exists in the directory
        for possible_config_file_name in config_file_names.clone() {
            let file_path = config_dir_path.join(possible_config_file_name);
            debug!("Checking for file {}", file_path.to_str().unwrap());
            if file_path.exists() {
                return file_path
                    .as_os_str()
                    .to_str()
                    .expect("Could not convert to str")
                    .to_string();
            }
        }
    }

    panic!("Could not find any valid config file in common paths.");
}
