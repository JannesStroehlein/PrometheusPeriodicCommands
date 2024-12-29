use crate::config::schema::CommandTarget;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use std::sync::atomic::AtomicU64;
use std::time::Duration;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct LastResultLabels {
    name: String,
    exit_code: i32,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct LastDurationLabels {
    name: String,
}

#[derive(Default)]
pub struct Metrics {
    pub last_result: Family<LastResultLabels, Gauge<f64, AtomicU64>>,
    pub last_duration: Family<LastDurationLabels, Gauge>,
}

impl Metrics {
    pub fn update_result(&self, target: &CommandTarget, exit_code: i32, numeric_value: f64) {
        self.last_result
            .get_or_create(&LastResultLabels {
                name: target.name.clone(),
                exit_code,
            })
            .set(numeric_value);
    }

    pub fn update_duration(&self, target: &CommandTarget, duration: &Duration) {
        self.last_duration
            .get_or_create(&LastDurationLabels {
                name: target.name.clone(),
            })
            .set(duration.as_millis() as i64);
    }
}
