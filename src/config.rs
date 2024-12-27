use std::fs::File;
use std::io::Read;
use std::path::Path;
use log::debug;

pub mod Schema;

const CONFIG_FILE_DIR_NAME : &str = "prometheus_periodic_commands";

pub fn read_cfg() -> Result<Schema::Schema, String> {
    let found_config_path = explore_config_file_paths();
    debug!("Found config file in '{found_config_path}'");

    let mut file = match File::open(found_config_path) {
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


/// Test for common file paths to find a config file
fn explore_config_file_paths() -> String {
    let config_file_names = [
        String::from("config.yaml")
    ];

    #[cfg(target_os = "linux")]
    let os_specific_config_dirs = [
        String::from(""),
        format!("/etc/{CONFIG_FILE_DIR_NAME}"),
        format!("~/.config/{CONFIG_FILE_DIR_NAME}")
    ];

    #[cfg(target_os = "windows")]
    let os_specific_config_dirs  = [
        "",
        ""
    ];

    // check current dir
    for possible_config_path in os_specific_config_dirs.clone() {
        let expanded_path = shellexpand::full(&possible_config_path)
            .expect(&format!("Could not expand the config path: {possible_config_path}"));

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
                return file_path.as_os_str().to_str().expect("Could not convert to str").to_string();
            }
        }
    };

    panic!("Could not find any valid config file in common paths.");
}