//! helper function to handle entity archetypes

use uuid::Uuid;

use crate::prelude::*;
use crate::query;

/// Initialize a spawn at the given position
///
/// ```
/// use caolo_sim::prelude::*;
/// use caolo_sim::entity_archetypes::init_structure_spawn;
///
/// let mut w = World::new();
///
/// let id = w.insert_entity();
/// init_structure_spawn(id, Default::default(), WorldPosition{
///     room: Axial::new(12,12),
///     pos: Axial::new(12,12),
/// }, &mut w);
///
///
/// let _spawn = w.view::<EntityId, SpawnComponent>().get_by_id(id).unwrap();
///
/// ```
pub fn init_structure_spawn(id: EntityId, owner_id: Uuid, pos: WorldPosition, world: &mut World) {
    // TODO tweak these numbas
    query!(
        mutate world
        {
            EntityId, Structure, .insert(id);
            EntityId, SpawnComponent, .insert_or_update(id, SpawnComponent::default());
            EntityId, SpawnQueueComponent, .insert_or_update(id, SpawnQueueComponent::default());
            EntityId, OwnedEntity, .insert_or_update(
                id,
                OwnedEntity {
                    owner_id: UserId(owner_id),
                }
            );
            EntityId, EnergyComponent, .insert_or_update(
                id,
                EnergyComponent {
                    energy: 500,
                    energy_max: 500,
                }
            );
            EntityId, EnergyRegenComponent, .insert_or_update(id, EnergyRegenComponent { amount: 20 });
            EntityId, HpComponent, .insert_or_update(
                id,
                HpComponent {
                    hp: 500,
                    hp_max: 500,
                }
            );
            EntityId, PositionComponent, .insert_or_update(id, PositionComponent(pos));
            WorldPosition, EntityComponent, .insert(pos, EntityComponent(id))
                .expect("entities_by_pos insert failed");

        }
    );
}

type InitBotTables = (
    UnsafeView<EntityId, Bot>,
    UnsafeView<EntityId, HpComponent>,
    UnsafeView<EntityId, DecayComponent>,
    UnsafeView<EntityId, CarryComponent>,
    UnsafeView<EntityId, PositionComponent>,
    UnsafeView<EntityId, OwnedEntity>,
    UnsafeView<EntityId, EntityScript>,
);
pub fn init_bot(
    entity_id: EntityId,
    owner_id: Option<Uuid>,
    pos: WorldPosition,
    (
        mut bots,
        mut hps,
        mut decay,
        mut carry,
        mut positions,
        mut owned,
        mut script_table,
    ): InitBotTables,
    user_default_scripts: View<UserId, EntityScript>,
) {
    bots.insert(entity_id);
    hps.insert_or_update(
        entity_id,
        HpComponent {
            hp: 100,
            hp_max: 100,
        },
    );
    decay.insert_or_update(
        entity_id,
        DecayComponent {
            interval: 10,
            time_remaining: 10,
            hp_amount: 10,
        },
    );
    carry.insert_or_update(
        entity_id,
        CarryComponent {
            carry: 0,
            carry_max: 150,
        },
    );

    positions.insert_or_update(entity_id, PositionComponent(pos));

    if let Some(owner_id) = owner_id {
        if let Some(script) = user_default_scripts.get_by_id(UserId(owner_id)) {
            script_table.insert_or_update(entity_id, *script);
        }

        owned.insert_or_update(
            entity_id,
            OwnedEntity {
                owner_id: UserId(owner_id),
            },
        );
    }
}
