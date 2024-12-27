use duration_string::DurationString;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Schema {
    pub targets: Vec<CommandTarget>,
    pub host: String,
    pub port: u16
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct CommandTarget {
    pub command: String,
    pub regex: String,
    pub regex_named_group: String,
    pub success_exit_codes: Vec<i32>,
    pub run_every: DurationString,
}