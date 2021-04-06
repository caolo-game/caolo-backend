use crate::protos::cao_commands::{PlaceStructureCommand, StructureType};
use caolo_sim::{join, prelude::*, tables::JoinIterator};
use thiserror::Error;
use tracing::{error, info};
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

    #[error("Missing expected field {0}")]
    MissingField(&'static str),

    #[error("Unrecognized structure type {0}")]
    BadType(i32),
}

pub fn place_structure(
    storage: &mut World,
    command: &PlaceStructureCommand,
) -> Result<(), PlaceStructureError> {
    info!("Handling place_structure command {:?}", command);

    let position = command
        .position
        .as_ref()
        .ok_or(PlaceStructureError::MissingField("position"))?;

    let pos = position
        .pos
        .as_ref()
        .ok_or(PlaceStructureError::MissingField("position.pos"))?;
    let pos = Axial::new(pos.q, pos.r);

    let room = position
        .room
        .as_ref()
        .ok_or(PlaceStructureError::MissingField("position.room"))?;
    let room = Axial::new(room.q, room.r);

    let position = WorldPosition { room, pos };

    let is_valid_terrain = storage
        .view::<WorldPosition, TerrainComponent>()
        .get_by_id(position)
        .map(|TerrainComponent(t)| t.is_walkable())
        .unwrap_or(false);

    if !is_valid_terrain {
        return Err(PlaceStructureError::InvalidPosition(position));
    }

    let is_free = storage
        .view::<WorldPosition, EntityComponent>()
        .get_by_id(position)
        .is_none();

    if !is_free {
        return Err(PlaceStructureError::TakenPosition(position));
    }

    let ty = command.ty;
    let owner = command
        .owner_id
        .as_ref()
        .ok_or(PlaceStructureError::MissingField("owner_id"))?
        .data
        .as_slice();
    let owner = uuid::Uuid::from_slice(owner).map_err(|err| {
        error!("Failed to parse owner id {:?}", err);
        PlaceStructureError::OwnerIdError
    })?;
    let ty = StructureType::from_i32(ty).ok_or(PlaceStructureError::BadType(ty))?;
    let entity_id;
    let owner_id = owner;
    match ty {
        StructureType::Spawn => {
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

            entity_id = storage.insert_entity();

            caolo_sim::entity_archetypes::init_structure_spawn(
                entity_id, owner_id, position, storage,
            );
        }
    }

    Ok(())
}
