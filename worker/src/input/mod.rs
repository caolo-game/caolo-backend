//! Handle inputs received via the message bus
mod script_update;
use crate::protos::input_messages::{InputMsg, InputMsg_oneof_msg};
use caolo_sim::prelude::*;
use log::{debug, error};
use protobuf::parse_from_bytes;
use redis::Commands;
use std::str::{from_utf8, Utf8Error};
use uuid::{self, Uuid};

pub fn handle_messages(storage: &mut World, client: &redis::Client) {
    debug!("handling incoming messages");
    let mut connection = client.get_connection().expect("Get redis conn");

    // log errors, but otherwise ignore them, so the loop may continue, retrying later
    'a: while let Ok(Some(message)) = connection
        .rpop::<_, Option<Vec<u8>>>("INPUTS")
        .map_err(|e| {
            error!("Failed to GET message {:?}", e);
        })
        .map::<Option<InputMsg>, _>(|message| {
            message.and_then(|message| {
                parse_from_bytes(&message)
                    .map_err(|e| {
                        error!("Failed to deserialize message {:?}", e);
                    })
                    .ok()
            })
        })
    {
        let msg_id = match parse_uuid(&message.msg_id) {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to parse msg_id {:?}", e);
                continue 'a;
            }
        };
        debug!("Handling message {:?}", msg_id);
        match message.msg {
            None => error!("Message did not contain `msg` payload!"),
            Some(InputMsg_oneof_msg::update_script(update)) => {
                script_update::update_program(storage, update)
                    .map_err(|e| {
                        error!("Script update failed {:?}", e);
                        // TODO: return error msg
                    })
                    .unwrap_or(());
            }
            Some(InputMsg_oneof_msg::update_entity_script(update)) => {
                script_update::update_entity_script(storage, update)
                    .map_err(|e| {
                        error!("Entity script update failed {:?}", e);
                        // TODO: return error msg
                    })
                    .unwrap_or(());
            }
        }
    }
    debug!("handling incoming messages done");
}

#[derive(Debug, Clone, Copy)]
enum UuidDeserializeError {
    BadString(Utf8Error),
    BadUuid(uuid::parser::ParseError),
}

fn parse_uuid(id: &[u8]) -> Result<Uuid, UuidDeserializeError> {
    from_utf8(&id)
        .map_err(UuidDeserializeError::BadString)
        .and_then(|id| Uuid::parse_str(id).map_err(UuidDeserializeError::BadUuid))
}
