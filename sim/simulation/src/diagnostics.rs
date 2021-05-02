mod serde_impl;

use chrono::Duration;
use serde::{Deserialize, Serialize};
use tracing::Level;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DiagDur(pub Duration);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StatsField {
    name: String,
    count: u64,
    current: f64,
    min: f64,
    max: f64,
    mean: f64,
    /// std = (std_aggregator / count).sqrt()
    std_aggregator: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostics {
    tick_stats: StatsField,
    scripts_execution_time: StatsField,
    scripts_ran: StatsField,
    scripts_error: StatsField,
    systems: StatsField,
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self {
            tick_stats: StatsField::new("tick_time".to_string()),
            scripts_execution_time: StatsField::new("scripts_time".to_string()),
            scripts_ran: StatsField::new("scripts_ran".to_string()),
            scripts_error: StatsField::new("scripts_error".to_string()),
            systems: StatsField::new("systems_time".to_string()),
        }
    }
}

impl StatsField {
    pub fn new(name: String) -> Self {
        Self {
            name,
            count: 0,
            current: std::f64::NAN,
            min: std::f64::MAX,
            max: std::f64::MIN,
            mean: 0.0,
            std_aggregator: 0.0,
        }
    }

    pub fn clear(&mut self) {
        self.count = 0;
        self.current = std::f64::NAN;
        self.min = std::f64::MAX;
        self.max = std::f64::MIN;
        self.mean = 0.0;
        self.std_aggregator = 0.0;
    }

    pub fn update(&mut self, value: f64) {
        self.current = value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);

        let tmp = value - self.mean;
        self.mean += tmp / (self.count as f64 + 1.0);
        self.std_aggregator += tmp * (value - self.mean);
        self.count += 1;
    }

    #[allow(unused)]
    pub fn current(&self) -> f64 {
        self.current
    }

    pub fn std(&self) -> f64 {
        (self.std_aggregator / self.count as f64).sqrt()
    }

    #[allow(unused)]
    pub fn mean(&self) -> f64 {
        self.mean
    }

    #[allow(unused)]
    pub fn min_max(&self) -> [f64; 2] {
        [self.min, self.max]
    }

    #[allow(unused)]
    pub fn count(&self) -> u64 {
        self.count
    }
}

impl StatsField {
    /// emits an INFO event
    pub fn emit_tracing_event(&self) {
        tracing::event!(
            Level::INFO,
            name = %self.name,
            current = %self.current,
            min = %self.min,
            max = %self.max,
            mean = %self.mean,
            count = %self.count,
            std = %self.std(),
        );
    }
}

impl Diagnostics {
    /// emits an INFO event
    pub fn emit_tracing_event(&self) {
        self.tick_stats.emit_tracing_event();
        self.scripts_execution_time.emit_tracing_event();
        self.systems.emit_tracing_event();
        self.scripts_ran.emit_tracing_event();
        self.scripts_error.emit_tracing_event();
    }

    pub fn clear(&mut self) {
        self.tick_stats.clear();
        self.scripts_execution_time.clear();
        self.scripts_ran.clear();
        self.scripts_error.clear();
        self.systems.clear();
    }

    pub fn update_latency(&mut self, duration: Duration) {
        let latency = duration.num_milliseconds();
        self.tick_stats.update(latency as f64);
    }

    pub fn update_systems(&mut self, duration: Duration) {
        let latency = duration.num_milliseconds();
        self.systems.update(latency as f64);
    }

    pub fn update_scripts(
        &mut self,
        duration: Duration,
        number_executed: u64,
        number_errored: u64,
    ) {
        let latency = duration.num_milliseconds();
        self.scripts_execution_time.update(latency as f64);
        self.scripts_error.update(number_errored as f64);
        self.scripts_ran.update(number_executed as f64);
    }
}
