use crate::model::EntityId;
use cao_lang::traits::AutoByteEncodeProperties;

#[derive(Clone, Debug, Default, Copy)]
pub struct BotDescription {}

#[derive(Clone, Debug, Default, Copy)]
pub struct SpawnIntent {
    pub id: EntityId,
    pub bot: BotDescription,
}

impl AutoByteEncodeProperties for BotDescription {}
impl AutoByteEncodeProperties for SpawnIntent {}
