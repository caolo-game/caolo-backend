use super::*;
use crate::bots::Bot;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnIntent {
    pub id: EntityId,
    pub bot: Bot,
}
