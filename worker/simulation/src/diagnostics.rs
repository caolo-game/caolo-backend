use crate::prelude::{tables::unique::UniqueTable, Component, EmptyKey};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostics {
    pub tick: u64,

    pub tick_latency_ms: i64,
    pub tick_start: DateTime<Utc>,
    pub tick_end: DateTime<Utc>,
    pub scripts_execution_ms: i64,
    pub systems_update_ms: i64,
    pub number_of_scripts_ran: i64,
    pub number_of_scripts_errored: i64,
    pub number_of_intents: i64,

    pub tick_latency_mean: f64,
    pub tick_latency_std: f64,

    pub tick_latency_mean_aggregator: f64,
    pub tick_latency_std_aggregator: f64,
}

impl Default for Diagnostics {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            tick: 0,
            tick_latency_ms: 0,
            number_of_scripts_ran: 0,
            number_of_scripts_errored: 0,
            number_of_intents: 0,
            tick_start: now,
            tick_end: now,
            systems_update_ms: 0,
            scripts_execution_ms: 0,

            tick_latency_mean: 0.0,
            tick_latency_std: 0.0,
            tick_latency_mean_aggregator: 0.0,
            tick_latency_std_aggregator: 0.0,
        }
    }
}

impl Diagnostics {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn update_latency_stats(&mut self, latency: i64, tick: u64) {
        self.tick_latency_ms = latency;
        self.tick = tick;

        let latency = latency as f64;

        let t = latency - self.tick_latency_mean_aggregator;
        self.tick_latency_mean_aggregator += t / (tick as f64 + 1.0);
        self.tick_latency_std_aggregator += t * (latency - self.tick_latency_mean_aggregator);

        self.tick_latency_std = (self.tick_latency_std_aggregator / (self.tick + 1) as f64).sqrt();
        self.tick_latency_mean = self.tick_latency_mean_aggregator * (tick + 1) as f64;
    }
}

impl Component<EmptyKey> for Diagnostics {
    type Table = UniqueTable<EmptyKey, Diagnostics>;
}
