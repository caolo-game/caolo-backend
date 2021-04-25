use std::collections::HashMap;

use crate::protos::cao_common;
use crate::protos::cao_world;
use caolo_sim::prelude::*;
use super::util::push_room_pl;

type ResourceTables<'a> = (
    View<'a, WorldPosition, EntityComponent>,
    View<'a, EntityId, ResourceComponent>,
    View<'a, EntityId, EnergyComponent>,
    WorldTime,
);

pub fn resource_payload(
    out: &mut HashMap<Axial, cao_world::RoomEntities>,
    (room_entities, resource, energy, WorldTime(time)): ResourceTables,
) {
    let room_entities = room_entities.iter_rooms();

    let mut room = None;
    let mut accumulator = Vec::with_capacity(128);

    for (r, entities) in room_entities {
        // push the accumulator
        if Some(r) != room {
            if !accumulator.is_empty() {
                push_room_pl(
                    out,
                    r.0,
                    |pl| &mut pl.resources,
                    std::mem::take(&mut accumulator),
                    time as i64,
                );
            }
            room = Some(r);
            accumulator.clear();
        }
        for (pos, EntityComponent(entity_id)) in entities.iter() {
            let entity_id = *entity_id;
            if let Some(resource) = resource.get_by_id(entity_id) {
                match resource.0 {
                    Resource::Empty => {}
                    Resource::Energy => {
                        accumulator.push(cao_world::Resource {
                            id: entity_id.0.into(),
                            pos: Some(cao_common::WorldPosition {
                                room: Some(cao_common::Axial { q: r.0.q, r: r.0.r }),
                                pos: Some(cao_common::Axial { q: pos.q, r: pos.r }),
                            }),
                            resource_type: energy.get_by_id(entity_id).copied().map(
                                |EnergyComponent { energy, energy_max }: EnergyComponent| {
                                    cao_world::resource::ResourceType::Energy(cao_world::Bounded {
                                        value: energy.into(),
                                        value_max: energy_max.into(),
                                    })
                                },
                            ),
                        });
                    }
                }
            }
        }
    }
    // push the last accumulator
    if room.is_some() && !accumulator.is_empty() {
        push_room_pl(out, room.unwrap().0, |pl| &mut pl.resources, accumulator, time as i64);
    }
}
