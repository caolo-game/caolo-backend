use crate::protos::cao_commands::{PlaceStructureCommand, StructureType};
use caolo_sim::{join, prelude::*, query, tables::JoinIterator};
use slog::{error, Logger};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum PlaceStructureError {
    #[error("user {user_id} already has a spawn ({spawn_id:?})!")]
    UserHasSpawn { user_id: Uuid, spawn_id: EntityId },

    #[error("position {0:?} is not valid!")]
    InvalidPosition(WorldPosition),

    #[error("position {0:?} is taken!")]
    TakenPosition(WorldPosition),

    #[error("Failed to parse owner id")]
    OwnerIdError,
}

pub fn place_structure(
    logger: Logger,
    storage: &mut World,
    command: &PlaceStructureCommand,
) -> Result<(), PlaceStructureError> {
    let entity_id = storage.insert_entity();

    let position = command.get_position();

    let pos = position.get_pos();
    let q = pos.get_q();
    let r = pos.get_r();
    let pos = Axial::new(q, r);

    let room = position.get_room();
    let q = room.get_q();
    let r = room.get_r();
    let room = Axial::new(q, r);

    let position = WorldPosition { room, pos };

    let is_valid_terrain = storage
        .view::<WorldPosition, TerrainComponent>()
        .get_by_id(&position)
        .map(|TerrainComponent(t)| t.is_walkable())
        .unwrap_or(false);

    if !is_valid_terrain {
        return Err(PlaceStructureError::InvalidPosition(position));
    }

    let is_free = storage
        .view::<WorldPosition, EntityComponent>()
        .get_by_id(&position)
        .is_none();

    if !is_free {
        return Err(PlaceStructureError::TakenPosition(position));
    }

    let ty = command.get_ty();
    let owner = command.get_ownerId().get_data();
    let owner = uuid::Uuid::from_slice(owner).map_err(|err| {
        error!(logger, "Failed to parse owner id {:?}", err);
        PlaceStructureError::OwnerIdError
    })?;
    match ty {
        StructureType::SPAWN => {
            // a player may only have 1 spawn atm
            let has_spawn = join!(
                storage
                EntityId
                [ spawn: SpawnComponent, owner: OwnedEntity ]
            )
            .find(|(_, (_, OwnedEntity { ref owner_id }))| owner_id.0 == owner)
            .map(|(id, _)| id);

            if let Some(spawn_id) = has_spawn {
                return Err(PlaceStructureError::UserHasSpawn {
                    user_id: owner,
                    spawn_id,
                });
            }

            query!(
                mutate
                storage
                {
                    EntityId, SpawnComponent,
                        .insert_or_update(entity_id, SpawnComponent::default());
                }
            );
        }
    }

    let owner_id = UserId(owner);
    query!(
        mutate
        storage
        {
            EntityId, Structure,
                .insert(entity_id);

            EntityId, PositionComponent,
                .insert_or_update(entity_id, PositionComponent(position));

            EntityId, OwnedEntity,
                .insert_or_update(entity_id, OwnedEntity{owner_id});

            WorldPosition, EntityComponent,
                .insert(position, EntityComponent(entity_id))
                // expect that position validity is confirmed at this point
                .expect("Failed to insert position");
        }
    );

    Ok(())
}
