//! Parse cao_message's
//! TODO: return errors instead of panics pls
//!
use crate::model::script::Function;
use crate::model::world::{
    AxialPoint, Bot, LogEntry, Resource, ResourceType, RoomState, ScriptHistoryEntry, Structure,
    StructurePayload, StructureSpawn, WorldPosition,
};
use cao_messages::point_capnp::world_position;
use std::collections::HashMap;

pub type CaoUuid<'a> = cao_messages::world_capnp::uuid::Reader<'a>;

pub fn parse_bot(
    bot: &cao_messages::world_capnp::bot::Reader,
    rooms: &mut HashMap<AxialPoint, RoomState>,
) {
    let world_pos = parse_world_pos(&bot.get_position().expect("bot.pos"));
    rooms.entry(world_pos.room).or_default().bots.push(Bot {
        id: bot.get_id(),
        position: world_pos,
        owner: if bot.has_owner() {
            Some(parse_uuid(&bot.get_owner().expect("bot.owner")))
        } else {
            None
        },

        body: serde_json::from_str(
            bot.get_body()
                .expect("bot.body")
                .get_value()
                .expect("bot.body.value"),
        )
        .expect("failed to parse bot body"),
    });
}

pub fn parse_structure(
    structure: &cao_messages::world_capnp::structure::Reader,
    rooms: &mut HashMap<AxialPoint, RoomState>,
) {
    let world_pos = parse_world_pos(&structure.get_position().expect("structure.pos"));

    let payload = if structure.has_spawn() {
        let spawn = structure.get_spawn().unwrap();
        let spawn = StructureSpawn {
            time_to_spawn: spawn.get_time_to_spawn(),
            spawning: spawn.get_spawning(),
            energy: spawn.get_energy(),
            energy_max: spawn.get_energy_max(),
            energy_regen: spawn.get_energy_regen(),
        };
        StructurePayload::Spawn(spawn)
    } else {
        panic!("structure type not handled");
    };

    rooms
        .entry(world_pos.room)
        .or_default()
        .structures
        .push(Structure {
            id: structure.get_id(),
            position: world_pos,
            owner: if structure.has_owner() {
                Some(parse_uuid(&structure.get_owner().expect("structure.owner")))
            } else {
                None
            },

            payload,
        });
}

pub fn parse_resource(
    resource: &cao_messages::world_capnp::resource::Reader,
    rooms: &mut HashMap<AxialPoint, RoomState>,
) {
    let world_pos = parse_world_pos(&resource.get_position().expect("resource.pos"));

    let payload = if resource.has_energy() {
        let energy = resource.get_energy().unwrap();
        ResourceType::Energy {
            energy: energy.get_energy(),
            energy_max: energy.get_energy_max(),
        }
    } else {
        panic!("resource type not implemented");
    };

    rooms
        .entry(world_pos.room)
        .or_default()
        .resources
        .push(Resource {
            id: resource.get_id(),
            position: world_pos,
            ty: payload,
        });
}

pub fn parse_world_pos(pos: &world_position::Reader) -> WorldPosition {
    let room = pos.get_room().expect("pos.room");

    let room = AxialPoint {
        q: room.get_q(),
        r: room.get_r(),
    };

    let pos = pos.get_room_pos().expect("pos.room_pos");

    let room_pos = AxialPoint {
        q: pos.get_q(),
        r: pos.get_r(),
    };
    WorldPosition { room, room_pos }
}

pub fn parse_uuid(id: &CaoUuid) -> uuid::Uuid {
    let data = id.get_data().expect("data");
    uuid::Uuid::from_slice(data).expect("parse uuid failed")
}

pub fn parse_script_history(
    entry: &cao_messages::world_capnp::script_history_entry::Reader,
) -> ScriptHistoryEntry {
    let entity_id = entry.reborrow().get_entity_id();
    let payload = entry.reborrow().get_payload().expect("payload");
    let len = payload.len();
    let mut result = ScriptHistoryEntry {
        entity_id,
        payload: Vec::with_capacity(len as usize),
    };
    let pl = &mut result.payload;
    for e in payload.iter() {
        pl.push(e)
    }
    result
}

pub fn parse_log(entry: &cao_messages::world_capnp::log_entry::Reader) -> LogEntry {
    let mut result = LogEntry {
        entity_id: entry.reborrow().get_entity_id(),
        time: entry.reborrow().get_time(),
        payload: Vec::new(),
    };

    for txt in entry
        .reborrow()
        .get_payload()
        .expect("entry.payload")
        .iter()
    {
        let txt = txt.expect("Failed to get text");
        result.payload.push(txt.to_string());
    }

    result
}

pub fn parse_function_desc<'a>(
    fun: cao_messages::script_capnp::function::Reader<'a>,
) -> Function<'a> {
    let mut res = Function {
        name: fun.get_name().expect("function.name"),
        description: fun.get_description().expect("function.description"),
        ty: fun
            .get_ty()
            .ok()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(cao_lang::SubProgramType::Undefined),
        input: Vec::with_capacity(4),
        output: Vec::with_capacity(4),
        constants: Vec::with_capacity(4),
    };

    for input in fun.get_input().expect("function.input").iter() {
        res.input.push(input.expect("failed to read input"));
    }

    for output in fun.get_input().expect("function.output").iter() {
        res.output.push(output.expect("failed to read output"));
    }

    for param in fun.get_input().expect("function.constants").iter() {
        res.constants.push(param.expect("failed to read param"));
    }

    res
}
