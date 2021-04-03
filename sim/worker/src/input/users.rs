use super::parse_uuid;
use crate::protos::cao_commands::RegisterUser;
use caolo_sim::{prelude::*, query};
use slog::{debug, Logger};
use std::{convert::TryFrom, num::TryFromIntError};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RegisterUserError {
    #[error("Invalid message {0}")]
    BadMessage(anyhow::Error),
    #[error("User by id {0} has been registered already")]
    AlreadyRegistered(Uuid),
    #[error("{0} is not a valid level")]
    BadLevel(TryFromIntError),
}

pub fn register_user(
    logger: Logger,
    world: &mut World,
    msg: &RegisterUser,
) -> Result<(), RegisterUserError> {
    debug!(logger, "Register user");

    let user_id = parse_uuid(msg.get_userId()).map_err(RegisterUserError::BadMessage)?;

    let level = msg.get_level();
    let level = u16::try_from(level).map_err(RegisterUserError::BadLevel)?;

    if world
        .view::<UserId, UserProperties>()
        .reborrow()
        .contains(UserId(user_id))
    {
        return Err(RegisterUserError::AlreadyRegistered(user_id));
    }

    let user_id = UserId(user_id);

    query!(
        mutate
        world
        {
            UserId, UserComponent,
                .insert(user_id);
            UserId, Rooms,
                .insert_or_update(user_id, Rooms::default());
            UserId, UserProperties,
                .insert_or_update(user_id, UserProperties{level});
        }
    );

    Ok(())
}
