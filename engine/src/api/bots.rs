use super::*;
use crate::intents::{check_dropoff_intent, check_mine_intent, check_move_intent};
use crate::profile;
use crate::model::{self, EntityId};
use crate::storage::Storage;
use crate::tables::BotTable;
use rayon::prelude::*;

pub fn build_bot(id: EntityId, storage: &Storage) -> Option<caolo_api::bots::Bot> {
    let pos = storage
        .entity_table::<model::PositionComponent>()
        .get_by_id(&id)
        .or_else(|| {
            debug!("Bot {:?} could not be build because it has no position", id);
            None
        })?;

    let carry = storage
        .entity_table::<model::CarryComponent>()
        .get_by_id(&id)
        .unwrap_or_else(|| model::CarryComponent {
            carry: 0,
            carry_max: 0,
        });

    let bot = storage.entity_table::<model::Bot>().get_by_id(&id);

    bot.map(|bot| caolo_api::bots::Bot {
        id,
        speed: bot.speed,
        owner_id: bot.owner_id,
        position: pos.0,
        carry: carry.carry,
        carry_max: carry.carry_max,
    })
    .or_else(|| {
        debug!(
            "Bot {:?} could not be build because it has no bot component",
            id
        );
        None
    })
}

/// Return the number of bots of the user
#[no_mangle]
pub fn _get_my_bots_len(ctx: &mut Ctx) -> i32 {
    profile!("_get_my_bots_len");
    debug!("_get_my_bots_len");

    let userid = unsafe { get_current_user_id(ctx) };
    let bots: &dyn BotTable = unsafe { get_storage(ctx).entity_table() };
    let bots = bots.get_bots_by_owner(userid);
    let res = bots.len() as i32;
    debug!("_get_my_bots_len returns: {}", res);
    res
}

/// Takes the output pointer as a parameter
/// Out value: list of Bots, serialized
/// Returns the written values' length in bytes
#[no_mangle]
pub fn _get_my_bots(ctx: &mut Ctx, ptr: i32) -> i32 {
    profile!("_get_my_bots");
    debug!("_get_my_bots");

    let userid = unsafe { get_current_user_id(ctx) };
    let data = {
        let storage = unsafe { get_storage(ctx) };
        let bots = storage
            .entity_table::<model::Bot>()
            .get_bots_by_owner(userid)
            .into_par_iter()
            .filter_map(|(id, _)| build_bot(id, storage))
            .collect();
        let bots = caolo_api::bots::Bots::new(bots);

        bots.serialize()
    };
    let len = data.len();

    save_bytes_to_memory(ctx, ptr as usize, len, &data);

    debug!("_get_my_bots written {} bytes, returns {}", len, len);
    len as i32
}

/// Send a pointer and length to the bytes of the serialzed MoveIntent
/// Return an OperationResult
#[no_mangle]
pub fn _send_move_intent(ctx: &mut Ctx, ptr: i32, len: i32) -> i32 {
    profile!("_send_move_intent");

    if len < 0 || 512 < len {
        return OperationResult::InvalidInput as i32;
    }

    let data = read_bytes(ctx, ptr as usize, len as usize);
    let intent = caolo_api::bots::MoveIntent::deserialize(&data);
    if let Err(e) = intent {
        error!("Failed to deserialize move intent {:?}", e);
        return OperationResult::InvalidInput as i32;
    }
    let intent = intent.unwrap();
    let userid = unsafe { get_current_user_id(ctx) };
    let storage = unsafe { get_storage(ctx) };

    {
        let checkresult = check_move_intent(&intent, *userid, storage);
        match checkresult {
            OperationResult::Ok => {}
            _ => return checkresult as i32,
        }
    }

    let intents = unsafe { get_intents_mut(ctx) };

    intents.push(intents::Intent::new_move(intent.id, intent.position));

    OperationResult::Ok as i32
}

#[no_mangle]
pub fn _send_mine_intent(ctx: &mut Ctx, ptr: i32, len: i32) -> i32 {
    profile!("_send_mine_intent");

    if len < 0 || 512 < len {
        return OperationResult::InvalidInput as i32;
    }
    let data = read_bytes(ctx, ptr as usize, len as usize);
    let intent = caolo_api::bots::MineIntent::deserialize(&data);
    if let Err(e) = intent {
        error!("Failed to deserialize move intent {:?}", e);
        return OperationResult::InvalidInput as i32;
    }
    let intent = intent.unwrap();

    let userid = unsafe { get_current_user_id(ctx) };
    let storage = unsafe { get_storage(ctx) };

    {
        let checkresult = check_mine_intent(&intent, *userid, storage);
        match checkresult {
            OperationResult::Ok => {}
            _ => return checkresult as i32,
        }
    }

    let intents = unsafe { get_intents_mut(ctx) };

    intents.push(intents::Intent::new_mine(intent.id, intent.target));

    OperationResult::Ok as i32
}

#[no_mangle]
pub fn _send_dropoff_intent(ctx: &mut Ctx, ptr: i32, len: i32) -> i32 {
    profile!("_send_dropoff_intent");
    debug!("_send_dropoff_intent");
    if len < 0 || 512 < len {
        return OperationResult::InvalidInput as i32;
    }
    let data = read_bytes(ctx, ptr as usize, len as usize);
    let intent = caolo_api::bots::DropoffIntent::deserialize(&data);
    if let Err(e) = intent {
        error!("Failed to deserialize dropoff intent {:?}", e);
        return OperationResult::InvalidInput as i32;
    }
    let intent = intent.unwrap();

    let userid = unsafe { get_current_user_id(ctx) };
    let storage = unsafe { get_storage(ctx) };

    {
        debug!("checking dropoff intent");
        let checkresult = check_dropoff_intent(&intent, *userid, storage);
        match checkresult {
            OperationResult::Ok => {}
            _ => {
                debug!(
                    "checking dropoff intent failed. returning error {:?}",
                    checkresult
                );
                return checkresult as i32;
            }
        }
    }

    let intents = unsafe { get_intents_mut(ctx) };

    intents.push(intents::Intent::new_dropoff(
        intent.id,
        intent.target,
        intent.amount,
        intent.ty,
    ));

    debug!("_send_dropoff_intent done");
    OperationResult::Ok as i32
}
