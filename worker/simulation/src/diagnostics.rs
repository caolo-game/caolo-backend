use crate::prelude::{tables::unique::UniqueTable, Component, EmptyKey};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostics {
    /// start time of diagnostics collection
    pub start: DateTime<Utc>,
    /// current tick
    pub tick: u64,

    /// total latency of the current tick
    pub tick_latency_ms: i64,
    pub scripts_execution_ms: i64,
    pub systems_update_ms: i64,

    // aggregated stats
    pub tick_latency_min: i64,
    pub tick_latency_max: i64,
    pub tick_latency_mean: f64,
    pub tick_latency_std: f64,
    pub tick_latency_count: u64,

    pub number_of_scripts_ran: u64,
    pub number_of_scripts_errored: u64,
    pub number_of_intents: u64,

    /// total time since the beginning of stats collection. a.k.a [start](Diagnostics::start)
    pub uptime_ms: i64,

    pub __tick_latency_std_aggregator: f64,
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
            start: now,
            systems_update_ms: 0,
            scripts_execution_ms: 0,
            uptime_ms: 0,

            tick_latency_min: std::i64::MAX,
            tick_latency_max: 0,
            tick_latency_count: 0,
            tick_latency_mean: 0.0,
            tick_latency_std: 0.0,
            __tick_latency_std_aggregator: 0.0,
        }
    }
}

impl Diagnostics {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn update_latency_stats(&mut self, tick: u64, start: DateTime<Utc>, end: DateTime<Utc>) {
        let duration = end - start;

        let latency = duration.num_milliseconds();

        debug_assert!(
            (latency.abs() - self.systems_update_ms - self.scripts_execution_ms).abs() < 2,
            "Latency should equal the sum of the sub-categories"
        );

        self.uptime_ms = (end - self.start).num_milliseconds();

        self.tick_latency_ms = latency;
        self.tick = tick;

        self.tick_latency_min = self.tick_latency_min.min(latency);
        self.tick_latency_max = self.tick_latency_max.max(latency);

        let latency = latency as f64;

        let tick = self.tick_latency_count as f64;
        let tmp = latency - self.tick_latency_mean;
        self.tick_latency_mean += tmp / (tick + 1.0);
        self.__tick_latency_std_aggregator += tmp * (latency - self.tick_latency_mean);

        self.tick_latency_std = (self.__tick_latency_std_aggregator / tick).sqrt();
        self.tick_latency_count += 1;
    }
}

impl Component<EmptyKey> for Diagnostics {
    type Table = UniqueTable<EmptyKey, Diagnostics>;
}
