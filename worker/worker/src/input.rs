//! Handle inputs received via the message bus
mod rooms;
mod script_update;
mod structures;
mod users;
use crate::protos;
use anyhow::Context;
use caolo_sim::prelude::*;

use protobuf::Message;
use redis::AsyncCommands;
use redis::Client;
use slog::{error, info, o, trace, warn, Logger};
use uuid::Uuid;

type InputMsg = protos::cao_commands::InputMessage;

fn parse_uuid(id: &protos::cao_common::Uuid) -> anyhow::Result<uuid::Uuid> {
    uuid::Uuid::from_slice(id.data.as_slice()).with_context(|| "Failed to parse uuid")
}

/// Write the response and return the msg id
fn handle_single_message(
    logger: &Logger,
    message: InputMsg,
    storage: &mut World,
    response: &mut protos::cao_commands::CommandResult,
) -> anyhow::Result<Uuid> {
    let msg_id = message
        .messageId
        .as_ref()
        .with_context(|| "Failed to get message id")?;
    let msg_id = parse_uuid(msg_id)?;
    let logger = logger.new(o!("msg_id" => format!("{:?}",msg_id)));
    trace!(logger, "Handling message");
    let res = match message
        .payload
        .with_context(|| format!("Failed to get msg body of message {:?}", msg_id))?
    {
        protos::cao_commands::InputMessage_oneof_payload::placeStructure(cmd) => {
            structures::place_structure(logger.clone(), storage, &cmd).map_err(|e| {
                warn!(logger, "Structure placement failed {:?}", e);
                e.to_string()
            })
        }
        protos::cao_commands::InputMessage_oneof_payload::updateScript(update) => {
            script_update::update_program(logger.clone(), storage, &update).map_err(|e| {
                warn!(logger, "Script update failed {:?}", e);
                e.to_string()
            })
        }
        protos::cao_commands::InputMessage_oneof_payload::updateEntityScript(update) => {
            script_update::update_entity_script(storage, &update).map_err(|e| {
                warn!(logger, "Entity script update failed {:?}", e);
                e.to_string()
            })
        }
        protos::cao_commands::InputMessage_oneof_payload::setDefaultScript(update) => {
            script_update::set_default_script(storage, &update).map_err(|e| {
                warn!(logger, "Setting dewfault script failed {:?}", e);
                e.to_string()
            })
        }
        protos::cao_commands::InputMessage_oneof_payload::takeRoom(cmd) => {
            rooms::take_room(logger.clone(), storage, &cmd).map_err(|e| {
                warn!(logger, "Failed to take room {:?}", e);
                e.to_string()
            })
        }
        protos::cao_commands::InputMessage_oneof_payload::registerUser(cmd) => {
            users::register_user(logger.clone(), storage, &cmd).map_err(|e| {
                warn!(logger, "Failed to register user {:?}", e);
                e.to_string()
            })
        }
    };

    match res {
        Ok(_) => {}
        Err(err) => response.set_error(err),
    };

    Ok(msg_id)
}

pub async fn handle_messages<'a>(
    logger: Logger,
    storage: &'a mut World,
    queue: &'a Client,
) -> anyhow::Result<()> {
    trace!(logger, "handling incoming messages");

    let mut queue = queue
        .get_async_connection()
        .await
        .with_context(|| "Failed to get Redis connection")?;

    let mut response_payload = Vec::new();
    while let Ok(Some(message)) = queue
        .rpop("CAO_COMMANDS")
        .await
        .map_err(|e| {
            error!(logger, "Failed to GET message {:?}", e);
        })
        .map::<Option<InputMsg>, _>(|message: Option<Vec<u8>>| {
            message.and_then(|message| {
                Message::parse_from_bytes(message.as_slice())
                    .map_err(|err| {
                        error!(logger, "Failed to parse protobuf message {:?}", err);
                    })
                    .ok()
            })
        })
    {
        let mut response = protos::cao_commands::CommandResult::new();
        match handle_single_message(&logger, message, storage, &mut response) {
            Ok(msg_id) => {
                response_payload.clear();
                response
                    .write_to_vec(&mut response_payload)
                    .with_context(|| "Failed to write response to byte vector")?;
                queue
                    .set_ex(format!("{}", msg_id), response_payload.as_slice(), 10)
                    .await?;
                info!(logger, "Message {:?} response sent!", msg_id);
            }
            Err(err) => {
                error!(logger, "Message handling failed, {:?}", err);
            }
        }
    }
    trace!(logger, "handling incoming messages done");
    Ok(())
}
