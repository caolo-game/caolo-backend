use super::*;
use cao_lang::traits::AutoByteEncodeProperties;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Copy)]
#[serde(rename_all = "camelCase")]
pub struct BotDescription {}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Copy)]
#[serde(rename_all = "camelCase")]
pub struct SpawnIntent {
    pub id: EntityId,
    pub bot: BotDescription,
}

impl AutoByteEncodeProperties for BotDescription {}
impl AutoByteEncodeProperties for SpawnIntent {}
