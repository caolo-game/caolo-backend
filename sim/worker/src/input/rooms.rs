use crate::protos::cao_commands::TakeRoomCommand;
use anyhow::Context;
use caolo_sim::prelude::*;
use thiserror::Error;
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum TakeRoomError {
    #[error("Target room already has an owner")]
    Owned,
    #[error("Maximum number of rooms ({0}) owned already")]
    MaxRoomsExceeded(usize),
    #[error("Internal error: {0}")]
    InternalError(anyhow::Error),
    #[error("User by id {0} was not registered")]
    NotRegistered(Uuid),
    #[error("Missing expected field {0}")]
    MissingField(&'static str),
    #[error("Failed to parse uuid {0}")]
    UuidError(anyhow::Error),
}

pub fn take_room(world: &mut World, msg: &TakeRoomCommand) -> Result<(), TakeRoomError> {
    debug!("Taking room");

    let user_id = msg
        .user_id
        .as_ref()
        .ok_or(TakeRoomError::MissingField("user_id"))?
        .data
        .as_slice();
    let user_id =
        uuid::Uuid::from_slice(user_id).map_err(|err| TakeRoomError::UuidError(err.into()))?;

    let room_id = msg
        .room_id
        .as_ref()
        .ok_or(TakeRoomError::MissingField("room_id"))?;
    let room_id = Axial::new(room_id.q, room_id.r);

    let has_owner = world.view::<Axial, OwnedEntity>().contains_key(room_id);
    if has_owner {
        return Err(TakeRoomError::Owned);
    }

    let rooms = world
        .view::<UserId, Rooms>()
        .reborrow()
        .get_by_id(UserId(user_id));
    let num_rooms = rooms.map(|x| x.0.len()).unwrap_or(0);

    let props = world
        .view::<UserId, UserProperties>()
        .reborrow()
        .get_by_id(UserId(user_id));

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
        .unsafe_view::<Axial, OwnedEntity>()
        .insert_or_update(
            room_id,
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
