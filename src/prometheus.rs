use crate::config::schema::{Target, TargetCommand};
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use regex::Regex;
use std::process::Output;
use std::sync::atomic::AtomicU64;
use std::time::Duration;

#[derive(Default)]
pub struct TargetMetrics {
    pub last_result: Family<Vec<(String, String)>, Gauge<f64, AtomicU64>>,
    pub last_duration: Family<Vec<(String, String)>, Gauge>,
}

impl TargetMetrics {
    pub fn update_result(
        &self,
        target: &Target,
        command: &TargetCommand,
        regex: &Regex,
        execution_result: &Output,
        duration: &Duration,
    ) -> Result<(), String> {
        let std_out = String::from_utf8_lossy(&execution_result.stdout);

        let captures = match regex.captures(std_out.trim()) {
            Some(caps) => caps,
            None => {
                return Err("RegEx did not find any captures in stdout".to_string());
            }
        };

        let cap = captures
            .name(&*target.regex_named_group)
            .map_or("", |m| m.as_str());

        let mut result_labels = vec![
            ("name".to_owned(), target.name.to_owned()),
            (
                "exit_code".to_owned(),
                execution_result.status.to_string().to_owned(),
            ),
        ];

        for (label, value) in &command.labels {
            let mut templated_value = value.clone();
            for group_name in regex
                .capture_names()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap())
            {
                let group_content = captures.name(group_name).map_or("", |m| m.as_str());

                templated_value = templated_value
                    .replace(("{".to_owned() + group_name + "}").as_str(), group_content)
                    .to_owned();
            }
            result_labels.push((label.to_owned(), templated_value.to_string()));
        }

        // Simply parse the stdout to a f64 and return that or explode trying
        let numeric_value = match cap.parse::<f64>() {
            Err(_) => {
                return Err(format!(
                    "Could not parse capture to f64.\nCaptures: {captures:?}\nStdout:{std_out}"
                ))
            }
            Ok(c) => c,
        };

        self.last_result
            .get_or_create(&result_labels)
            .set(numeric_value);

        self.last_duration
            .get_or_create(&result_labels)
            .set(duration.as_millis() as i64);

        Ok(())
    }
}
