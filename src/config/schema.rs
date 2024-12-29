use duration_string::DurationString;
use serde::{Deserialize, Serialize};

/// The config file schema
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Schema {
    pub targets: Vec<CommandTarget>,
    pub host: String,
    pub port: u16,
}

/// A command that should be executed and parsed
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct CommandTarget {
    /// The name of the target
    pub name: String,
    /// The shell command to execute
    pub command: String,
    /// The regex to parse the stdout
    pub regex: String,
    /// The named group in the regex containing the result
    pub regex_named_group: String,
    /// A list of exit codes that indicate a successful execution
    pub success_exit_codes: Vec<i32>,
    /// The interval to execute the command in
    pub run_every: DurationString,
}
