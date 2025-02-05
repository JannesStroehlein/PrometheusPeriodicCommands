use duration_string::DurationString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The config file schema
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Schema {
    pub targets: Vec<Target>,
    pub host: String,
    pub port: u16,
}

/// A command that should be executed and parsed
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Target {
    /// The name of the target
    pub name: String,
    /// The shell command to execute
    pub commands: Vec<TargetCommand>,
    /// The regex to parse the stdout
    pub regex: String,
    /// The named group in the regex containing the result
    pub regex_named_group: String,
    /// A list of exit codes that indicate a successful execution
    pub success_exit_codes: Vec<i32>,
    /// The interval to execute the command in
    pub run_every: DurationString,
}

/// A command that should be executed and parsed
/// This is a separate struct to allow to add labels to the metric that is created
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TargetCommand {
    /// The command to execute
    pub exec: String,
    /// A list of additional labels to add to the metric
    /// The values can be templated with the result of the command regex
    pub labels: HashMap<String, String>,
}
