//! ## Terminology:
//! - overworld: the large-scale overview of the map.
//! - room: a self-contained slice of the map. Hexagon shaped.
//!
pub mod overworld;
pub mod room;

use self::overworld::{generate_room_layout, OverworldGenerationError, OverworldGenerationParams};
use self::room::{generate_room, RoomGenerationError, RoomGenerationParams};
use crate::storage::views::UnsafeView;
use crate::{
    components::{RoomComponent, RoomConnections, RoomProperties, TerrainComponent},
    prelude::Axial,
};
use crate::{
    indices::{ConfigKey, Room, WorldPosition},
    tables::hex_grid::HexGrid,
};
use arrayvec::ArrayVec;
use rand::{rngs::SmallRng, thread_rng, RngCore, SeedableRng};
use slog::{o, Logger};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum MapGenError {
    #[error("Failed to generate room: {err}")]
    RoomGenerationError {
        err: RoomGenerationError,
        room: Room,
    },

    #[error("Failed to generate overworld: {err}")]
    OverworldGenerationError { err: OverworldGenerationError },
}

pub type MapGenerationTables = (
    UnsafeView<WorldPosition, TerrainComponent>,
    UnsafeView<Axial, RoomComponent>,
    UnsafeView<ConfigKey, RoomProperties>,
    UnsafeView<Axial, RoomConnections>,
);

pub fn generate_full_map(
    logger: Logger,
    overworld_params: &OverworldGenerationParams,
    room_params: &RoomGenerationParams,
    seed: Option<[u8; 16]>,
    (mut terrain, rooms, mut room_props, room_connections): MapGenerationTables,
) -> Result<(), MapGenError> {
    let seed = seed.unwrap_or_else(|| {
        let mut bytes = [0; 16];
        thread_rng().fill_bytes(&mut bytes);
        bytes
    });
    let mut rng = SmallRng::from_seed(seed);
    generate_room_layout(
        logger.clone(),
        overworld_params,
        &mut rng,
        (rooms, room_connections),
    )
    .map_err(|err| MapGenError::OverworldGenerationError { err })?;

    let radius = room_params.radius as usize;

    // setup properties table
    {
        use std::convert::TryInto;

        let room_radius = room_params.radius;
        room_props.value = Some(RoomProperties {
            radius: room_radius,
            center: crate::prelude::Hexagon::from_radius(room_radius.try_into().unwrap()).center,
        });
    }

    let terrain_tables = rooms.iter().try_fold(
        Vec::with_capacity(rooms.len()),
        |mut terrain_tables, (room, _)| {
            // TODO: do this in parallel?
            let mut terrain_table = HexGrid::new(radius as usize);
            let room_connections = room_connections
                .at(room)
                .expect("Expected just built room to have room_connections");
            let room_connections = room_connections
                .0
                .iter()
                .filter_map(|c| c.as_ref())
                .cloned()
                .collect::<ArrayVec<_, 6>>();
            let room_params = RoomGenerationParams {
                room: Room(room),
                ..room_params.clone()
            };
            generate_room(
                logger.new(o!("room.q" => room.q,"room.r" => room.r)),
                &room_params,
                room_connections.as_slice(),
                (UnsafeView::from_table(&mut terrain_table),),
            )
            .map_err(|err| MapGenError::RoomGenerationError {
                err,
                room: Room(room),
            })?;

            terrain_tables.push((room, terrain_table));
            Ok(terrain_tables)
        },
    )?;
    terrain
        .table
        .extend(terrain_tables.into_iter())
        .expect("expected to be able to insert the room terrain tables");
    Ok(())
}
