use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct Labels {
    command: String,
    exit_code: i32
}

#[derive(Default)]
pub struct Metrics {
    pub last_output: Family<Labels, Gauge>
}

impl Metrics {
    pub fn update_requests(&self, command: &str, exit_code : i32, numeric_value: i64) {
        self.last_output
            .get_or_create(&Labels { command: String::from(command), exit_code })
            .set(numeric_value);
    }
}