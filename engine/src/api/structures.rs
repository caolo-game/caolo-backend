use super::*;
use crate::intents::check_spawn_intent;
use crate::model::{self, EntityId};
use crate::profile;
use crate::storage::Storage;
use crate::tables::StructureTable;
use caolo_api::structures::Spawn;
use rayon::prelude::*;

/// Return the number of structures of the user
#[no_mangle]
pub fn _get_my_structures_len(ctx: &mut Ctx) -> i32 {
    profile!("_get_my_structures_len");
    debug!("_get_my_structures_len");
    let userid = unsafe { get_current_user_id(ctx) };
    let structures = unsafe { get_storage(ctx).entity_table::<model::Structure>() };
    let structures = structures.get_structures_by_owner(userid);
    let res = structures.len() as i32;
    debug!("_get_my_structures_len returns: {}", res);
    res
}

/// Takes the output pointer as a parameter
/// Out value: list of Structures, serialized
/// Returns the written values' length in bytes
#[no_mangle]
pub fn _get_my_structures(ctx: &mut Ctx, ptr: i32) -> i32 {
    profile!("_get_my_structures_len");
    debug!("_get_my_structures");

    let userid = unsafe { get_current_user_id(ctx) };
    let data = {
        let storage = unsafe { get_storage(ctx) };
        let structures = storage.entity_table::<model::Structure>();
        let structures = structures
            .get_structures_by_owner(userid)
            .into_par_iter()
            .map(|(id, structure)| {
                if let Some(spawn) = assemble_spawn(id, &structure, storage) {
                    spawn
                } else {
                    panic!("Unimplemented structure type")
                }
            })
            .collect();

        let structures = caolo_api::structures::Structures::new(structures);
        structures.serialize()
    };
    let len = data.len();

    save_bytes_to_memory(ctx, ptr as usize, len, &data);

    debug!("_get_my_structures written {} bytes, returns {}", len, len);
    len as i32
}

fn assemble_spawn(
    id: crate::model::EntityId,
    structure: &crate::model::Structure,
    storage: &Storage,
) -> Option<caolo_api::structures::Structure> {
    debug!("Assembling spawn {} {:?}", id, structure);
    storage
        .entity_table::<model::SpawnComponent>()
        .get_by_id(&id)
        .and_then(|spawn| {
            let hp = storage
                .entity_table::<model::HpComponent>()
                .get_by_id(&id)
                .or_else(|| {
                    debug!("Spawn should have hp");
                    None
                })?;
            let energy = storage
                .entity_table::<model::EnergyComponent>()
                .get_by_id(&id)
                .or_else(|| {
                    debug!("Spawn should have energy");
                    None
                })?;
            let position = storage
                .entity_table::<model::PositionComponent>()
                .get_by_id(&id)
                .or_else(|| {
                    debug!("Structures should have position");
                    None
                })?;
            let spawn = caolo_api::structures::Structure::Spawn(Spawn {
                id,
                owner_id: structure.owner_id,
                position: position.0,

                energy: energy.energy,
                energy_max: energy.energy_max,

                hp: hp.hp,
                hp_max: hp.hp_max,

                time_to_spawn: spawn.time_to_spawn,
                spawning: spawn.spawning,
            });
            Some(spawn)
        })
}

#[no_mangle]
pub fn _send_spawn_intent(ctx: &mut Ctx, ptr: i32, len: i32) -> i32 {
    if len < 0 || 512 < len {
        return OperationResult::InvalidInput as i32;
    }

    let data = read_bytes(ctx, ptr as usize, len as usize);
    let intent = caolo_api::structures::SpawnIntent::deserialize(&data);
    if let Err(e) = intent {
        error!("Failed to deserialize spawn intent {:?}", e);
        return OperationResult::InvalidInput as i32;
    }
    let mut intent = intent.unwrap();
    let userid = unsafe { get_current_user_id(ctx) };
    let storage = unsafe { get_storage(ctx) };

    {
        let checkresult = check_spawn_intent(&intent, *userid, storage);
        match checkresult {
            OperationResult::Ok => {}
            _ => return checkresult as i32,
        }
    }

    let pos = storage
        .entity_table::<model::PositionComponent>()
        .get_by_id(&intent.id)
        .unwrap()
        .0;

    {
        let bot = &mut intent.bot;
        bot.owner_id = Some(*userid);
        bot.position = pos;
    }

    let intents = unsafe { get_intents_mut(ctx) };
    intents.push(intents::Intent::new_spawn(intent.bot, intent.id));

    OperationResult::Ok as i32
}

pub fn build_structure(
    id: EntityId,
    storage: &Storage,
) -> Option<caolo_api::structures::Structure> {
    let structure = storage.entity_table::<model::Structure>().get_by_id(&id)?;
    assemble_spawn(id, &structure, storage)
}
