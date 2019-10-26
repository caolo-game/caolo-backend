mod bot;
mod intents;

pub use bot::*;
pub use intents::*;

use crate::{point::Point, UserId};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bots {
    pub bots: Vec<Bot>,
}

impl Bots {
    pub fn new(bots: Vec<Bot>) -> Self {
        Self { bots }
    }
}
