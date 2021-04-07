use crate::protos::cao_common;
use crate::protos::cao_world;
use caolo_sim::prelude::*;

type ResourceTables<'a> = (
    View<'a, WorldPosition, EntityComponent>,
    View<'a, EntityId, ResourceComponent>,
    View<'a, EntityId, EnergyComponent>,
);

pub fn resource_payload(
    out: &mut ::prost::alloc::vec::Vec<cao_world::RoomResources>,
    (room_entities, resource, energy): ResourceTables,
) {
    let room_entities = room_entities.iter_rooms();

    let mut room = None;
    let mut accumulator = Vec::with_capacity(128);

    for (r, entities) in room_entities {
        // push the accumulator
        if Some(r) != room {
            if !accumulator.is_empty() {
                out.push(cao_world::RoomResources {
                    room_id: room.map(|Room(Axial { q, r })| cao_common::Axial { q, r }),
                    resources: std::mem::take(&mut accumulator),
                });
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
    if room.is_some() {
        if !accumulator.is_empty() {
            out.push(cao_world::RoomResources {
                room_id: room.map(|Room(Axial { q, r })| cao_common::Axial { q, r }),
                resources: accumulator,
            });
        }
    }
}
