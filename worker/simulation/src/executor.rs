use std::{convert::Infallible, fmt::Debug, pin::Pin};

use slog::{debug, info, o, Logger};

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
    fn initialize(
        &mut self,
        logger: Option<Logger>,
        config: GameConfig,
    ) -> Result<Pin<Box<World>>, Self::Error>;
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
        let logger = world.logger.new(o!("tick" => tick));

        info!(logger, "Tick starting");

        let mut diag = world.unsafe_view::<EmptyKey, Diagnostics>();
        let diag: &mut Diagnostics = diag.unwrap_mut_or_default();
        diag.clear();

        let scripts_table = world.view::<EntityId, EntityScript>();
        let executions: Vec<(EntityId, EntityScript)> =
            scripts_table.iter().map(|(id, x)| (id, *x)).collect();

        let intents = {
            debug!(logger, "Executing scripts");
            let start = chrono::Utc::now();
            let intents = execute_scripts(executions.as_slice(), world);
            let end = chrono::Utc::now();
            let duration = end - start;
            diag.scripts_execution_ms = duration.num_milliseconds();
            debug!(logger, "Executing scripts Done");
            intents
        };
        {
            let start = chrono::Utc::now();

            debug!(logger, "Got {} intents", intents.len());
            intents::move_into_storage(world, intents);

            debug!(logger, "Executing systems update");
            execute_world_update(world);

            debug!(logger, "Executing post-processing");
            world.post_process();

            let end = chrono::Utc::now();
            let duration = end - start;
            diag.systems_update_ms = duration.num_milliseconds();
        }
        let end = chrono::Utc::now();
        let duration = end - start;

        diag.tick_start = start;
        diag.tick_end = end;
        diag.update_latency_stats(duration.num_milliseconds(), tick);
        info!(
            logger,
            "Tick done. Latency: {:.4}ms Mean latency: {:.4}ms Std latency: {:.4}ms",
            diag.tick_latency_ms,
            diag.tick_latency_mean,
            diag.tick_latency_std,
        );

        Ok(())
    }

    fn initialize(
        &mut self,
        logger: Option<Logger>,
        config: GameConfig,
    ) -> Result<Pin<Box<World>>, Self::Error> {
        let mut world = World::new(logger);

        execute_map_generation(world.logger.clone(), &mut *world, &config)
            .expect("Failed to generate world map");

        world.config.game_config.value = Some(config);

        Ok(world)
    }
}

fn execute_map_generation(
    logger: Logger,
    world: &mut World,
    config: &GameConfig,
) -> Result<(), MapGenError> {
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
    debug!(logger, "generating map {:#?} {:#?}", params, room_params);

    generate_full_map(
        logger.clone(),
        &params,
        &room_params,
        None,
        FromWorldMut::new(world),
    )?;

    debug!(logger, "world generation done");
    Ok(())
}
