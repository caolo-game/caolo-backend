use cao_lang::prelude::*;
use caolo_sim::components;
use caolo_sim::geometry::{Axial, Hexagon};
use caolo_sim::map_generation::generate_full_map;
use caolo_sim::map_generation::overworld::OverworldGenerationParams;
use caolo_sim::map_generation::room::RoomGenerationParams;
use caolo_sim::model::{self, EntityId, Room, ScriptId, WorldPosition};
use caolo_sim::storage::views::{FromWorld, FromWorldMut, UnsafeView, View};
use caolo_sim::World;
use log::{debug, trace};
use rand::Rng;
use std::pin::Pin;

pub fn init_storage(n_fake_users: usize) -> Pin<Box<World>> {
    debug!("initializing world");
    assert!(n_fake_users >= 1);

    let mut storage = caolo_sim::init_inmemory_storage();

    let script_id = ScriptId::default();
    let script: CompilationUnit =
        serde_json::from_str(PROGRAM).expect("deserialize example program");
    debug!("compiling default program");
    let compiled = compile(script).expect("failed to compile example program");
    debug!("compilation done");
    unsafe {
        storage
            .unsafe_view::<ScriptId, components::ScriptComponent>()
            .as_mut()
            .insert_or_update(script_id, components::ScriptComponent(compiled));
    };

    let mut rng = rand::thread_rng();

    let world_radius = std::env::var("CAO_MAP_OVERWORLD_RADIUS")
        .map(|w| {
            w.parse()
                .expect("expected map overworld radius to be an integer")
        })
        .unwrap_or((n_fake_users as f32).sqrt().max(1.0) as usize * 2);
    let width = std::env::var("CAO_MAP_WIDTH")
        .map(|w| w.parse().expect("expected map width to be an integer"))
        .unwrap_or(32);

    let radius = width as u32 / 2;
    assert!(radius > 0);
    let params = OverworldGenerationParams::builder()
        .with_radius(world_radius as u32)
        .with_room_radius(radius)
        .with_min_bridge_len(radius / 2)
        .with_max_bridge_len(radius)
        .build()
        .unwrap();
    let room_params = RoomGenerationParams::builder()
        .with_radius(radius)
        .with_chance_plain(0.33)
        .with_chance_wall(0.33)
        .with_plain_dilation(2)
        .build()
        .unwrap();
    debug!("generating map {:#?} {:#?}", params, room_params);

    generate_full_map(
        &params,
        &room_params,
        None,
        FromWorldMut::new(&mut *storage),
    )
    .unwrap();
    debug!("world generation done");

    unsafe {
        debug!("Reset position storage");
        let mut entities_by_pos =
            storage.unsafe_view::<WorldPosition, components::EntityComponent>();
        entities_by_pos.as_mut().clear();
        entities_by_pos
            .as_mut()
            .table
            .extend(
                storage
                    .view::<Room, components::RoomComponent>()
                    .iter()
                    .map(|(Room(roomid), _)| ((roomid, Default::default()))),
            )
            .expect("entities_by_pos init");
    }

    let bounds = Hexagon {
        center: Axial::new(radius as i32, radius as i32),
        radius: radius as i32,
    };
    let rooms = storage
        .view::<Room, components::RoomComponent>()
        .iter()
        .map(|a| a.0)
        .collect::<Vec<_>>();

    let mut taken_rooms = Vec::with_capacity(n_fake_users);
    for i in 0..n_fake_users {
        debug!("initializing spawn #{}", i);
        let storage = &mut storage;
        let spawnid = storage.insert_entity();

        let room = rng.gen_range(0, rooms.len());
        let room = rooms[room];
        taken_rooms.push(room);

        debug!("initializing spawn #{} in room {:?}", i, room);
        unsafe {
            init_spawn(
                &bounds,
                spawnid,
                room,
                &mut rng,
                FromWorldMut::new(storage),
                FromWorld::new(storage),
            );
            debug!("spawning entities");
            let spawn_pos = storage
                .view::<EntityId, components::PositionComponent>()
                .get_by_id(&spawnid)
                .expect("spawn should have position")
                .0;
            for _ in 0..3 {
                let botid = storage.insert_entity();
                init_bot(botid, script_id, spawn_pos, FromWorldMut::new(storage));
            }
        }
        debug!("initializing spawn #{} done", i);
    }

    for _ in 0..(n_fake_users / 3).max(1) {
        let room = rng.gen_range(0, taken_rooms.len());
        let room = taken_rooms[room];
        let id = storage.insert_entity();
        let storage = &mut storage;
        unsafe {
            init_resource(
                &bounds,
                id,
                room,
                &mut rng,
                FromWorldMut::new(storage),
                FromWorld::new(storage),
            );
        }
    }

    debug!("init done");
    storage
}

type InitBotMuts = (
    UnsafeView<EntityId, components::EntityScript>,
    UnsafeView<EntityId, components::Bot>,
    UnsafeView<EntityId, components::CarryComponent>,
    UnsafeView<EntityId, components::OwnedEntity>,
    UnsafeView<EntityId, components::PositionComponent>,
    UnsafeView<WorldPosition, components::EntityComponent>,
);

unsafe fn init_bot(
    id: EntityId,
    script_id: model::ScriptId,
    pos: WorldPosition,
    (
        mut entity_scripts,
        mut bots,
        mut carry_component,
        mut owners,
        mut positions,
        mut entities_by_pos,
    ): InitBotMuts,
) {
    entity_scripts
        .as_mut()
        .insert_or_update(id, components::EntityScript { script_id });
    bots.as_mut().insert_or_update(id, components::Bot {});
    carry_component.as_mut().insert_or_update(
        id,
        components::CarryComponent {
            carry: 0,
            carry_max: 50,
        },
    );
    owners.as_mut().insert_or_update(
        id,
        components::OwnedEntity {
            owner_id: Default::default(),
        },
    );

    positions
        .as_mut()
        .insert_or_update(id, components::PositionComponent(pos));
    entities_by_pos
        .as_mut()
        .table
        .get_by_id_mut(&pos.room)
        .expect("expected bot pos to be in the table")
        .insert(pos.pos, components::EntityComponent(id))
        .expect("entities_by_pos insert");
}

type InitSpawnMuts = (
    UnsafeView<EntityId, components::OwnedEntity>,
    UnsafeView<EntityId, components::SpawnComponent>,
    UnsafeView<EntityId, components::Structure>,
    UnsafeView<EntityId, components::PositionComponent>,
    UnsafeView<WorldPosition, components::EntityComponent>,
);
type InitSpawnConst<'a> = (View<'a, WorldPosition, components::TerrainComponent>,);

unsafe fn init_spawn(
    bounds: &Hexagon,
    id: EntityId,
    room: Room,
    rng: &mut impl Rng,
    (mut owners, mut spawns, mut structures, mut positions, mut entities_by_pos): InitSpawnMuts,
    (terrain,): InitSpawnConst,
) {
    debug!("init_spawn");
    structures
        .as_mut()
        .insert_or_update(id, components::Structure {});
    spawns
        .as_mut()
        .insert_or_update(id, components::SpawnComponent::default());
    owners.as_mut().insert_or_update(
        id,
        components::OwnedEntity {
            owner_id: Default::default(),
        },
    );

    let pos = uncontested_pos(room, bounds, &*entities_by_pos, &*terrain, rng);

    positions
        .as_mut()
        .insert_or_update(id, components::PositionComponent(pos));
    entities_by_pos
        .as_mut()
        .table
        .get_by_id_mut(&room.0)
        .expect("expected room to be in entities_by_pos table")
        .insert(pos.pos, components::EntityComponent(id))
        .expect("entities_by_pos insert");
    debug!("init_spawn done");
}

type InitResourceMuts = (
    UnsafeView<EntityId, components::PositionComponent>,
    UnsafeView<EntityId, components::ResourceComponent>,
    UnsafeView<EntityId, components::EnergyComponent>,
    UnsafeView<WorldPosition, components::EntityComponent>,
);

type InitResourceConst<'a> = (View<'a, WorldPosition, components::TerrainComponent>,);

unsafe fn init_resource(
    bounds: &Hexagon,
    id: EntityId,
    room: Room,
    rng: &mut impl Rng,
    (mut positions_table, mut resources_table, mut energy_table, mut entities_by_pos, ): InitResourceMuts,
    (terrain,): InitResourceConst,
) {
    resources_table.as_mut().insert_or_update(
        id,
        components::ResourceComponent(components::Resource::Energy),
    );
    energy_table.as_mut().insert_or_update(
        id,
        components::EnergyComponent {
            energy: 250,
            energy_max: 250,
        },
    );

    let pos = uncontested_pos(room, bounds, &*entities_by_pos, &*terrain, rng);

    positions_table
        .as_mut()
        .insert_or_update(id, components::PositionComponent(pos));
    entities_by_pos
        .as_mut()
        .table
        .get_by_id_mut(&room.0)
        .expect("expected room to be in entities_by_pos table")
        .insert(pos.pos, components::EntityComponent(id))
        .expect("entities_by_pos insert");
}

fn uncontested_pos<T: caolo_sim::tables::TableRow + Send + Sync>(
    room: Room,
    bounds: &Hexagon,
    positions_table: &caolo_sim::tables::morton_hierarchy::RoomMortonTable<T>,
    terrain_table: &caolo_sim::tables::morton_hierarchy::RoomMortonTable<
        components::TerrainComponent,
    >,
    rng: &mut impl Rng,
) -> WorldPosition {
    const TRIES: usize = 10_000;
    let from = bounds.center - Axial::new(bounds.radius, bounds.radius);
    let to = bounds.center + Axial::new(bounds.radius, bounds.radius);
    for _ in 0..TRIES {
        let x = rng.gen_range(from.q, to.q);
        let y = rng.gen_range(from.r, to.r);

        let pos = Axial::new(x, y);

        trace!("checking pos {:?}", pos);

        if !bounds.contains(&pos) {
            trace!("point {:?} is out of bounds {:?}", pos, bounds);
            continue;
        }

        let pos = WorldPosition { room: room.0, pos };

        if let Some(components::TerrainComponent(terrain)) = terrain_table.get_by_id(&pos) {
            if terrain.is_walkable() && !positions_table.contains_key(&pos) {
                return pos;
            }
        }
    }
    panic!(
        "Failed to find an uncontested_pos in {:?} {:?} in {} iterations",
        from, to, TRIES
    );
}

const PROGRAM: &str = r#"
{
  "nodes": {
    "0": {
      "node": {
        "ScalarInt": 0
      },
      "child": 1
    },
    "1": {
      "node": {
        "Call": "make_operation_result"
      },
      "child": 3
    },
    "2": {
      "node": {
        "Call": "find_closest_resource_by_range"
      },
      "child": 0
    },
    "3": {
      "node": {
        "Equals": null
      },
      "child": 4
    },
    "4": {
      "node": {
        "JumpIfTrue": 6
      },
      "child": 5
    },
    "5": {
      "node": {
        "Exit": null
      },
      "child": 6
    },
    "6": {
      "node": {
        "CopyLast": null
      },
      "child": 7
    },
    "7": {
      "node": {
        "Call": "approach_entity"
      },
      "child": 8
    },
    "8": {
      "node": {
        "Pop": null
      },
      "child": 9
    },
    "9": {
      "node": {
        "Call": "mine_resource"
      },
      "child": null
    },
    "-1": {
      "node": {
        "Start": null
      },
      "child": 2
    }
  }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(|| {
            env_logger::init();
        });
    }

    #[test]
    fn can_init_the_game() {
        setup();
        // smoke test: can the game be even initialized?
        init_storage(5);
    }
}
