use super::*;
use crate::prelude::*;
use crate::model;
use caolo_api::bots::Bot;
use caolo_api::OperationResult;

#[derive(Debug, Clone)]
pub struct SpawnIntent {
    pub id: EntityId,
    pub bot: Bot,
}

impl SpawnIntent {
    pub fn execute(self, storage: &mut Storage) -> IntentResult {
        debug!("Spawning bot {:?} from structure {:?}", self.bot, self.id);

        let mut spawn = storage
            .entity_table::<model::SpawnComponent>()
            .get_by_id(&self.id)
            .cloned()
            .ok_or_else(|| "structure does not have spawn component")?;

        if spawn.spawning.is_some() {
            Err("busy")?;
        }

        let energy = storage
            .entity_table::<model::EnergyComponent>()
            .get_by_id(&self.id)
            .ok_or_else(|| "structure does not have energy")?;

        if energy.energy < 200 {
            return Err("not enough energy".into());
        }

        let bot_id = storage.insert_entity();
        storage
            .entity_table_mut::<model::SpawnBotComponent>()
            .insert(bot_id, model::SpawnBotComponent { bot: model::Bot {} });
        if let Some(owner_id) = self.bot.owner_id {
            storage
                .entity_table_mut::<model::OwnedEntity>()
                .insert(bot_id, model::OwnedEntity { owner_id: model::UserId(owner_id) });
        }

        spawn.time_to_spawn = 5;
        spawn.spawning = Some(bot_id);

        storage
            .entity_table_mut::<model::SpawnComponent>()
            .insert(self.id, spawn);

        Ok(())
    }
}

pub fn check_spawn_intent(
    intent: &caolo_api::structures::SpawnIntent,
    userid: model::UserId,
    storage: &crate::storage::Storage,
) -> OperationResult {
    let id = model::EntityId(intent.id);
    match storage
        .entity_table::<model::Structure>()
        .get_by_id(&id)
    {
        Some(_) => {
            let owner_id = storage
                .entity_table::<model::OwnedEntity>()
                .get_by_id(&id);
            if owner_id.map(|id| id.owner_id != userid).unwrap_or(true) {
                return OperationResult::NotOwner;
            }
        }
        None => {
            debug!("Structure not found");
            return OperationResult::InvalidInput;
        }
    };

    if let Some(spawn) = storage
        .entity_table::<model::SpawnComponent>()
        .get_by_id(&id)
    {
        if spawn.spawning.is_some() {
            debug!("Structure is busy");
            return OperationResult::InvalidInput;
        }
    } else {
        debug!("Structure has no spawn component");
        return OperationResult::InvalidInput;
    }

    OperationResult::Ok
}
