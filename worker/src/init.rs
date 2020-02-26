use cao_lang::prelude::*;
use caolo_sim::model::{self, components, geometry::Point, terrain, EntityId, ScriptId};
use caolo_sim::storage::views::{FromWorldMut, UnsafeView};
use caolo_sim::World;
use log::debug;
use rand::Rng;

const PROGRAM: &str = r#"
name: "default"
nodes:
    8:
        node:
            Call:
                function: console_log
    9:
        node:
            Equals: null
        child: 6
    10:
        node:
            StringLiteral:
                value: "No moverino :("
        child: 11
    11:
        node:
            Call:
                function: console_log
    12:
        node:
            Start: null
        child: 0
    0:
      node:
          ScalarInt:
            value: 25
      child: 1
    1:
      node:
        ScalarInt:
            value: 25
      child: 2
    2:
      node:
        Call:
            function: make_point
      child: 3
    3:
        node:
            Call:
                function: move_bot_to_position
        child: 5
    4:
        node:
            Call:
                function: make_operation_result
        child: 9
    5:
        node:
            ScalarInt:
                value: 0
        child: 4
    6:
        node:
            JumpIfTrue:
                nodeid: 7
        child: 10
    7:
        node:
            StringLiteral:
                value: "Moving :)"
        child: 8
        "#;

pub fn init_storage(n_fake_users: usize) -> World {
    assert!(n_fake_users >= 1);

    let mut storage = caolo_sim::init_inmemory_storage();

    let script_id = ScriptId::default();
    let script: CompilationUnit =
        serde_yaml::from_str(PROGRAM).expect("deserialize example program");
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

    let mut terrain = storage.unsafe_view::<Point, components::TerrainComponent>();

    for _ in 0..200 {
        let pos = uncontested_pos(&*terrain, &mut rng);
        unsafe {
            terrain.as_mut().insert(
                pos,
                components::TerrainComponent(terrain::TileTerrainType::Wall),
            );
        }
    }

    for _ in 0..n_fake_users {
        let id = storage.insert_entity();
        let storage = &mut storage;
        unsafe {
            init_bot(id, script_id, &mut rng, FromWorldMut::new(storage));
        }
    }

    for _ in 0..(n_fake_users - 1).max(1) {
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
    rng: &mut impl Rng,
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
    carry_component
        .as_mut()
        .insert_or_update(id, Default::default());
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
        let x = rng.gen_range(0, 50);
        let y = rng.gen_range(0, 50);

        let pos = Point::new(x, y);

        if !positions_table.contains_key(&pos) {
            return pos;
        }
    }
}
