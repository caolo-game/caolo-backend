use crate::protos::cao_common;
use crate::protos::cao_world;
use caolo_sim::prelude::*;

type StructureTables<'a> = (
    View<'a, WorldPosition, EntityComponent>,
    View<'a, EntityId, Structure>,
    View<'a, EntityId, HpComponent>,
    View<'a, EntityId, OwnedEntity>,
    View<'a, EntityId, EnergyComponent>,
    View<'a, EntityId, EnergyRegenComponent>,
    View<'a, EntityId, SpawnComponent>,
    View<'a, EntityId, SpawnQueueComponent>,
);

pub fn structure_payload(
    out: &mut ::prost::alloc::vec::Vec<cao_world::RoomStructures>,
    (room_entities, structures, hp, owner, energy, energy_regen, spawn, spawn_q): StructureTables,
) {
    let room_entities = room_entities.iter_rooms();

    let mut room = None;
    let mut accumulator = Vec::with_capacity(128);

    for (r, entities) in room_entities {
        // push the accumulator
        if Some(r) != room {
            if !accumulator.is_empty() {
                out.push(cao_world::RoomStructures {
                    room_id: room.map(|Room(Axial { q, r })| cao_common::Axial { q, r }),
                    structures: std::mem::take(&mut accumulator),
                });
            }
            room = Some(r);
            accumulator.clear();
        }
        for (pos, EntityComponent(entity_id)) in entities.iter() {
            if structures.contains_id(entity_id) {
                let entity_id = *entity_id;
                let mut pl = cao_world::Structure {
                    id: entity_id.0.into(),
                    pos: Some(cao_common::WorldPosition {
                        room: Some(cao_common::Axial { q: r.0.q, r: r.0.r }),
                        pos: Some(cao_common::Axial { q: pos.q, r: pos.r }),
                    }),
                    hp: hp
                        .get_by_id(entity_id)
                        .copied()
                        .map(|HpComponent { hp, hp_max }| cao_world::Bounded {
                            value: hp.into(),
                            value_max: hp_max.into(),
                        }),
                    energy: energy.get_by_id(entity_id).copied().map(
                        |EnergyComponent { energy, energy_max }| cao_world::Bounded {
                            value: energy.into(),
                            value_max: energy_max.into(),
                        },
                    ),
                    energy_regen: energy_regen
                        .get_by_id(entity_id)
                        .copied()
                        .map(|EnergyRegenComponent { amount }| amount.into())
                        .unwrap_or(0),
                    owner: owner.get_by_id(entity_id).map(
                        |OwnedEntity {
                             owner_id: UserId(owner_id),
                         }| {
                            cao_common::Uuid {
                                data: owner_id.as_bytes().to_vec(),
                            }
                        },
                    ),
                    structure_type: Default::default(),
                };
                if let Some(spawn) = spawn.get_by_id(entity_id) {
                    pl.structure_type = Some(cao_world::structure::StructureType::Spawn(
                        cao_world::structure::Spawn {
                            spawning: spawn.spawning.map(|EntityId(id)| id.into()).unwrap_or(-1),
                            time_to_spawn: spawn.time_to_spawn.into(),
                            spawn_queue: spawn_q
                                .get_by_id(entity_id)
                                .map(|SpawnQueueComponent { queue }| {
                                    queue
                                        .iter()
                                        .copied()
                                        .map(|EntityId(id)| id.into())
                                        .collect()
                                })
                                .unwrap_or_default(),
                        },
                    ));
                }
                accumulator.push(pl);
            }
        }
    }
    // push the last accumulator
    if room.is_some() {
        if !accumulator.is_empty() {
            out.push(cao_world::RoomStructures {
                room_id: room.map(|Room(Axial { q, r })| cao_common::Axial { q, r }),
                structures: accumulator,
            });
        }
    }
}
