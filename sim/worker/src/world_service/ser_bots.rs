use crate::protos::cao_common;
use crate::protos::cao_world;
use caolo_sim::prelude::*;

type BotTables<'a> = (
    View<'a, WorldPosition, EntityComponent>,
    View<'a, EntityId, Bot>,
    View<'a, EntityId, CarryComponent>,
    View<'a, EntityId, HpComponent>,
    View<'a, EntityId, MeleeAttackComponent>,
    View<'a, EntityId, DecayComponent>,
    View<'a, EntityId, OwnedEntity>,
    View<'a, EntityId, EntityScript>,
);

pub fn bot_payload(
    out: &mut ::prost::alloc::vec::Vec<cao_world::RoomBots>,
    (room_entities, bots, carry, hp, melee, decay, owner, script): BotTables,
) {
    let room_entities = room_entities.iter_rooms();

    let mut room = None;
    let mut accumulator = Vec::with_capacity(128);

    for (r, entities) in room_entities {
        // push the accumulator
        if Some(r) != room {
            if !accumulator.is_empty() {
                out.push(cao_world::RoomBots {
                    room_id: room.map(|Room(Axial { q, r })| cao_common::Axial { q, r }),
                    bots: std::mem::take(&mut accumulator),
                });
            }
            room = Some(r);
            accumulator.clear();
        }
        for (pos, EntityComponent(entity_id)) in entities.iter() {
            if bots.contains_id(entity_id) {
                let entity_id = *entity_id;
                accumulator.push(cao_world::Bot {
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
                    carry: carry.get_by_id(entity_id).copied().map(
                        |CarryComponent { carry, carry_max }| cao_world::Bounded {
                            value: carry.into(),
                            value_max: carry_max.into(),
                        },
                    ),
                    decay: decay.get_by_id(entity_id).copied().map(
                        |DecayComponent {
                             hp_amount,
                             interval,
                             time_remaining,
                         }| cao_world::bot::Decay {
                            hp_amount: hp_amount.into(),
                            interval: interval.into(),
                            time_remaining: time_remaining.into(),
                        },
                    ),
                    melee_strength: melee
                        .get_by_id(entity_id)
                        .copied()
                        .map(|MeleeAttackComponent { strength }| strength.into())
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
                    script: script
                        .get_by_id(entity_id)
                        .map(|EntityScript(ScriptId(script_id))| cao_common::Uuid {
                            data: script_id.as_bytes().to_vec(),
                        }),
                });
            }
        }
    }
    // push the last accumulator
    if room.is_some() {
        if !accumulator.is_empty() {
            out.push(cao_world::RoomBots {
                room_id: room.map(|Room(Axial { q, r })| cao_common::Axial { q, r }),
                bots: accumulator,
            });
        }
    }
}
