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
/// let mut w = World::new(None);
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
            EntityId, EnergyRegenComponent, .insert_or_update(id, EnergyRegenComponent { amount: 5 });
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
