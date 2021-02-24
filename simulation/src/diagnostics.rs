use crate::prelude::{tables::unique::UniqueTable, Component, EmptyKey};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostics {
    pub tick_latency_ms: i64,
    pub tick_start: DateTime<Utc>,
    pub tick_end: DateTime<Utc>,
    pub number_of_scripts_ran: i64,
    pub number_of_scripts_errored: i64,
    pub number_of_intents: i64,
}

impl Default for Diagnostics {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            tick_latency_ms: 0,
            number_of_scripts_ran: 0,
            number_of_scripts_errored: 0,
            number_of_intents: 0,
            tick_start: now,
            tick_end: now,
        }
    }
}

impl Diagnostics {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

impl Component<EmptyKey> for Diagnostics {
    type Table = UniqueTable<EmptyKey, Diagnostics>;
}
