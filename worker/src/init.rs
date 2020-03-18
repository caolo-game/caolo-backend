use cao_lang::prelude::*;
use caolo_sim::model::{self, components, geometry::Point, terrain, EntityId, ScriptId};
use caolo_sim::storage::views::{FromWorldMut, UnsafeView};
use caolo_sim::World;
use log::debug;
use rand::Rng;
use std::pin::Pin;

pub fn init_storage(n_fake_users: usize) -> Pin<Box<World>> {
    assert!(n_fake_users >= 1);

    let mut storage = caolo_sim::init_inmemory_storage();

    let script_id = ScriptId::default();
    let script: CompilationUnit =
        serde_json::from_str(PROGRAM).expect("deserialize example program");
    debug!("compiling default program");
    let compiled = Compiler::compile(script).expect("failed to compile example program");
    debug!("compilation done");
    unsafe {
        storage
            .unsafe_view::<ScriptId, components::ScriptComponent>()
            .as_mut()
            .insert_or_update(script_id, components::ScriptComponent(compiled));
    };

    let mut rng = rand::thread_rng();

    for _ in 0..200 {
        let pos = uncontested_pos(
            &*storage.view::<Point, components::TerrainComponent>(),
            &mut rng,
        );
        unsafe {
            let mut terrain = storage.unsafe_view::<Point, components::TerrainComponent>();
            terrain.as_mut().insert(
                pos,
                components::TerrainComponent(terrain::TileTerrainType::Wall),
            );
        }
    }

    for _ in 0..n_fake_users {
        let storage = &mut storage;
        let spawnid = storage.insert_entity();
        unsafe {
            init_spawn(spawnid, &mut rng, FromWorldMut::new(storage));
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
    }

    for _ in 0..(n_fake_users / 3).max(1) {
        let id = storage.insert_entity();
        let storage = &mut storage;
        unsafe {
            init_resource(id, &mut rng, FromWorldMut::new(storage));
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
    UnsafeView<Point, components::EntityComponent>,
);

unsafe fn init_bot(
    id: EntityId,
    script_id: model::ScriptId,
    pos: Point,
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
        .insert(pos, components::EntityComponent(id));
}

type InitSpawnMuts = (
    UnsafeView<EntityId, components::OwnedEntity>,
    UnsafeView<EntityId, components::SpawnComponent>,
    UnsafeView<EntityId, components::Structure>,
    UnsafeView<EntityId, components::PositionComponent>,
    UnsafeView<Point, components::EntityComponent>,
);

unsafe fn init_spawn(
    id: EntityId,
    rng: &mut impl Rng,
    (mut owners, mut spawns, mut structures, mut positions, mut entities_by_pos): InitSpawnMuts,
) {
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

    let pos = uncontested_pos(&*entities_by_pos, rng);

    positions
        .as_mut()
        .insert_or_update(id, components::PositionComponent(pos));
    entities_by_pos
        .as_mut()
        .insert(pos, components::EntityComponent(id));
}

type InitResourceMuts = (
    UnsafeView<EntityId, components::PositionComponent>,
    UnsafeView<EntityId, components::ResourceComponent>,
    UnsafeView<EntityId, components::EnergyComponent>,
    UnsafeView<Point, components::EntityComponent>,
);

unsafe fn init_resource(
    id: EntityId,
    rng: &mut impl Rng,
    (mut positions_table, mut resources_table, mut energy_table, mut entities_by_pos): InitResourceMuts,
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

    let pos = uncontested_pos(&*entities_by_pos, rng);

    positions_table
        .as_mut()
        .insert_or_update(id, components::PositionComponent(pos));
    entities_by_pos
        .as_mut()
        .insert(pos, components::EntityComponent(id));
}

fn uncontested_pos<T: caolo_sim::tables::TableRow + Send + Sync>(
    positions_table: &caolo_sim::tables::MortonTable<Point, T>,
    rng: &mut impl Rng,
) -> Point {
    loop {
        let x = rng.gen_range(0, 100);
        let y = rng.gen_range(0, 100);

        let pos = Point::new(x, y);

        if !positions_table.contains_key(&pos) {
            return pos;
        }
    }
}

const PROGRAM: &str = r#"
{
  "nodes": {
    "0": {
      "node": {
        "ScalarInt": {
          "value": 0
        }
      },
      "child": 1
    },
    "1": {
      "node": {
        "Call": {
          "function": "make_operation_result"
        }
      },
      "child": 3
    },
    "2": {
      "node": {
        "Call": {
          "function": "find_closest_resource_by_range"
        }
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
        "JumpIfTrue": {
          "nodeid": 6
        }
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
        "Call": {
          "function": "approach_entity"
        }
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
        "Call": {
          "function": "mine_resource"
        }
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
}"#;
