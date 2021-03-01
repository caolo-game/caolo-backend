use super::parse_uuid;
use crate::protos::cao_commands::TakeRoom;
use anyhow::Context;
use caolo_sim::prelude::*;
use slog::{debug, Logger};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum TakeRoomError {
    #[error("Invalid message {0}")]
    BadMessage(anyhow::Error),
    #[error("Target room already has an owner")]
    Owned,
    #[error("Maximum number of rooms ({0}) owned already")]
    MaxRoomsExceeded(usize),
    #[error("Internal error: {0}")]
    InternalError(anyhow::Error),
    #[error("User by id {0} was not registered")]
    NotRegistered(Uuid),
}

pub fn take_room(logger: Logger, world: &mut World, msg: &TakeRoom) -> Result<(), TakeRoomError> {
    debug!(logger, "Taking room");

    let user_id = parse_uuid(msg.get_userId()).map_err(TakeRoomError::BadMessage)?;

    let room_id = msg.get_roomId();
    let room_id = Axial::new(room_id.get_q(), room_id.get_r());

    let has_owner = world
        .view::<Room, OwnedEntity>()
        .contains_key(&Room(room_id));
    if has_owner {
        return Err(TakeRoomError::Owned);
    }

    let rooms = world
        .view::<UserId, Rooms>()
        .reborrow()
        .get_by_id(&UserId(user_id));
    let num_rooms = rooms.map(|x| x.0.len()).unwrap_or(0);

    let props = world
        .view::<UserId, UserProperties>()
        .reborrow()
        .get_by_id(&UserId(user_id));

    let available_rooms = match props.map(|p| p.level) {
        Some(l) => l,
        None => {
            return Err(TakeRoomError::NotRegistered(user_id));
        }
    };

    if num_rooms > available_rooms as usize {
        return Err(TakeRoomError::MaxRoomsExceeded(available_rooms as usize));
    }
    let mut rooms = rooms.cloned().unwrap_or_else(Rooms::default);
    rooms.0.push(Room(room_id));

    world
        .unsafe_view::<Room, OwnedEntity>()
        .insert_or_update(
            Room(room_id),
            OwnedEntity {
                owner_id: UserId(user_id),
            },
        )
        .with_context(|| "Failed to insert the new owner")
        .map_err(TakeRoomError::InternalError)?;

    world
        .unsafe_view::<UserId, Rooms>()
        .insert_or_update(UserId(user_id), rooms);

    Ok(())
}
