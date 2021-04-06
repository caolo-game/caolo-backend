use std::{convert::Infallible, fmt::Debug, pin::Pin};

use tracing::{debug, info};

use crate::{
    components::EntityScript,
    diagnostics::Diagnostics,
    intents,
    map_generation::room::RoomGenerationParams,
    map_generation::MapGenError,
    map_generation::{generate_full_map, overworld::OverworldGenerationParams},
    prelude::EntityId,
    prelude::{EmptyKey, FromWorldMut},
    profile,
    systems::{execute_world_update, script_execution::execute_scripts},
    world::World,
};

pub use crate::components::game_config::GameConfig;

/// Execute world state updates
pub trait Executor {
    type Error: Debug;

    /// Initialize this executor's state and return the initial world state
    fn initialize(&mut self, config: GameConfig) -> Result<Pin<Box<World>>, Self::Error>;
    /// Forward the world state by 1 tick
    fn forward(&mut self, world: &mut World) -> Result<(), Self::Error>;
}

/// The simplest executor.
///
/// Just runs a world update
pub struct SimpleExecutor;

impl Executor for SimpleExecutor {
    type Error = Infallible;

    fn forward(&mut self, world: &mut World) -> Result<(), Self::Error> {
        let start = chrono::Utc::now();
        profile!("world_forward");

        let tick = world.time();
        let s = tracing::info_span!("", tick = tick);
        let _e = s.enter();

        debug!("Tick starting");

        let mut diag = world.unsafe_view::<EmptyKey, Diagnostics>();
        let diag: &mut Diagnostics = diag.unwrap_mut_or_default();

        let scripts_table = world.view::<EntityId, EntityScript>();
        let executions: Vec<(EntityId, EntityScript)> =
            scripts_table.iter().map(|(id, x)| (id, *x)).collect();

        let intents = {
            debug!("Executing scripts");
            let start = chrono::Utc::now();
            let intents = execute_scripts(executions.as_slice(), world);
            let end = chrono::Utc::now();
            let duration = end - start;
            diag.scripts_execution_ms = duration.num_milliseconds();
            debug!("Executing scripts Done");
            intents
        };
        {
            let start = chrono::Utc::now();

            debug!("Got {} intents", intents.len());
            intents::move_into_storage(world, intents);

            debug!("Executing systems update");
            execute_world_update(world);

            debug!("Executing post-processing");
            world.post_process();

            let end = chrono::Utc::now();
            let duration = end - start;
            diag.systems_update_ms = duration.num_milliseconds();
        }
        let end = chrono::Utc::now();

        diag.update_latency_stats(tick, start, end);
        debug!("Tick done");
        info!(
            "Latency | Current {:.4}ms | Mean {:.4}ms | STD {:.4}ms",
            diag.tick_latency_ms, diag.tick_latency_mean, diag.tick_latency_std,
        );

        Ok(())
    }

    fn initialize(&mut self, config: GameConfig) -> Result<Pin<Box<World>>, Self::Error> {
        let mut world = World::new();

        execute_map_generation(&mut *world, &config).expect("Failed to generate world map");

        world.config.game_config.value = Some(config);

        Ok(world)
    }
}

fn execute_map_generation(world: &mut World, config: &GameConfig) -> Result<(), MapGenError> {
    let world_radius = config.world_radius;
    let room_radius = config.room_radius;
    assert!(room_radius > 6);
    let params = OverworldGenerationParams::builder()
        .with_radius(world_radius as u32)
        .with_room_radius(room_radius)
        .with_min_bridge_len(3)
        .with_max_bridge_len(room_radius - 3)
        .build()
        .unwrap();
    let room_params = RoomGenerationParams::builder()
        .with_radius(room_radius)
        .with_chance_plain(0.39)
        .with_chance_wall(1.0 - 0.39)
        .with_plain_dilation(1)
        .build()
        .unwrap();
    debug!("generating map {:#?} {:#?}", params, room_params);

    generate_full_map(&params, &room_params, None, FromWorldMut::new(world))?;

    debug!("world generation done");
    Ok(())
}
