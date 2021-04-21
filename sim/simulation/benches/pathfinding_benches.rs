use caolo_sim::{
    components::{EntityComponent, TerrainComponent},
    executor::{Executor, GameConfig, SimpleExecutor},
    indices::WorldPosition,
    pathfinding::find_path,
    prelude::{FromWorld, World},
};
use criterion::{criterion_group, Criterion};
use rand::prelude::SliceRandom;
use rand::{rngs::SmallRng, SeedableRng};

fn get_rand() -> impl rand::Rng {
    SmallRng::seed_from_u64(0xdeadbeef)
}

fn create_world(room_radius: u32) -> std::pin::Pin<Box<World>> {
    let mut exc = SimpleExecutor;
    let world = exc
        .initialize(GameConfig {
            world_radius: 6,
            room_radius,
            ..Default::default()
        })
        .unwrap();

    world
}

fn bench_find_path_in_room(c: &mut Criterion) {
    let mut world = create_world(29);

    let terrain_points;
    let room;
    {
        let rooms = world.view::<WorldPosition, TerrainComponent>();

        let (r, room_terrain) = rooms.iter_rooms().next().expect("room");
        room = r;

        terrain_points = room_terrain
            .iter()
            .filter(|(_, t)| t.0.is_walkable())
            .map(|(pos, _)| pos)
            .collect::<Vec<_>>();
    }

    {
        let mut positions = world.unsafe_view::<WorldPosition, EntityComponent>();
        if positions.table.at(room.0).is_none() {
            positions
                .table
                .insert(room.0, Default::default())
                .expect("Failed to init entites table");
        }
    }

    let mut path = Vec::new();
    let mut rooms2visit = Vec::new();

    c.bench_function("find_path_in_room", move |b| {
        let mut rng = get_rand();

        b.iter(|| {
            let from = *terrain_points.choose(&mut rng).expect("from");
            let to = *terrain_points.choose(&mut rng).expect("to");

            path.clear();
            rooms2visit.clear();
            let room = room.0;

            find_path(
                WorldPosition { room, pos: from },
                WorldPosition { room, pos: to },
                1,
                FromWorld::new(&*world),
                2000,
                &mut path,
                &mut rooms2visit,
            )
        })
    });
}

criterion_group!(pathfinding_benches, bench_find_path_in_room);
