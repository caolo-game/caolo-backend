use std::collections::HashMap;

use crate::protos::cao_common;
use crate::protos::cao_world;
use caolo_sim::prelude::*;

use super::util::push_room_pl;

type BotTables<'a> = (
    View<'a, WorldPosition, EntityComponent>,
    View<'a, EntityId, Bot>,
    View<'a, EntityId, CarryComponent>,
    View<'a, EntityId, HpComponent>,
    View<'a, EntityId, MeleeAttackComponent>,
    View<'a, EntityId, DecayComponent>,
    View<'a, EntityId, OwnedEntity>,
    View<'a, EntityId, EntityScript>,
    View<'a, EntityTime, LogEntry>,
    WorldTime,
);

pub fn bot_payload(
    out: &mut HashMap<Axial, cao_world::RoomEntities>,
    (room_entities, bots, carry, hp, melee, decay, owner, script, logs, WorldTime(time)): BotTables,
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
                    |pl| &mut pl.bots,
                    std::mem::take(&mut accumulator),
                    time as i64,
                );
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
                    logs: logs
                        .get_by_id(EntityTime(entity_id, time - 1)) // send the logs of the last tick
                        .map(|logs| logs.payload.clone())
                        .unwrap_or_default(),
                });
            }
        }
    }
    // push the last accumulator
    if room.is_some() && !accumulator.is_empty() {
        push_room_pl(
            out,
            room.unwrap().0,
            |pl| &mut pl.bots,
            accumulator,
            time as i64,
        );
    }
}
