//! Handle inputs received via the message bus
mod script_update;
use crate::protos::input_messages::{InputMsg, InputMsg_oneof_msg};
use protobuf::parse_from_bytes;
use caolo_sim::storage::Storage;
use log::{debug, error};
use redis::Commands;

pub fn handle_messages(storage: &mut Storage, client: &redis::Client) {
    debug!("handling incoming messages");
    let mut connection = client.get_connection().expect("Get redis conn");
    while let Ok(Some(message)) = connection
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
        }
    }
    debug!("handling incoming messages done");
}
