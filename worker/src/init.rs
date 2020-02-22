use cao_lang::prelude::*;
use caolo_sim::model::{self, components, geometry::Point, terrain, EntityId, ScriptId};
use caolo_sim::storage::{
    views::{UnsafeView, View},
    Storage,
};
use rand::Rng;

const PROGRAM : &str = r#"
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
                function: bots::move_bot_to_position
        child: 5
    4:
        node:
            Call:
                function: make_operation_result
        child: 0
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
pub fn init_storage(n_fake_users: usize) -> Storage {
    let mut storage = caolo_sim::init_inmemory_storage();

    let script_id = ScriptId::default();
    let script: CompilationUnit =
        serde_yaml::from_str(PROGRAM).expect("deserialize example program");
    let compiled = Compiler::compile(script).expect("failed to compile example program");
    storage
        .scripts_table_mut::<components::ScriptComponent>()
        .insert_or_update(script_id, components::ScriptComponent(compiled));

    let mut rng = rand::thread_rng();

    let terrain = storage.point_table_mut::<components::TerrainComponent>();

    for _ in 0..200 {
        let pos = uncontested_pos(terrain, &mut rng);
        terrain.insert(
            pos,
            components::TerrainComponent(terrain::TileTerrainType::Wall),
        );
    }

    for _ in 0..n_fake_users {
        let id = storage.insert_entity();
        let storage = &mut storage;
        unsafe {
            init_bot(id, script_id, &mut rng, storage.into(), storage.into());
        }
    }
    storage
}

type InitBotMuts = (
    UnsafeView<EntityId, components::EntityScript>,
    UnsafeView<EntityId, components::Bot>,
    UnsafeView<EntityId, components::CarryComponent>,
    UnsafeView<EntityId, components::OwnedEntity>,
    UnsafeView<EntityId, components::PositionComponent>,
);

unsafe fn init_bot(
    id: EntityId,
    script_id: model::ScriptId,
    rng: &mut impl Rng,
    (mut entity_scripts, mut bots, mut carry_component, mut ownsers, mut positions): InitBotMuts,
    entities_by_pos: View<Point, components::EntityComponent>,
) {
    entity_scripts
        .as_mut()
        .insert_or_update(id, components::EntityScript { script_id });
    bots.as_mut().insert_or_update(id, components::Bot {});
    carry_component
        .as_mut()
        .insert_or_update(id, Default::default());
    ownsers.as_mut().insert_or_update(
        id,
        components::OwnedEntity {
            owner_id: Default::default(),
        },
    );

    let pos = uncontested_pos(&*entities_by_pos, rng);

    positions
        .as_mut()
        .insert_or_update(id, components::PositionComponent(pos));
}

fn uncontested_pos<T: caolo_sim::tables::TableRow + Send + Sync>(
    positions_table: &caolo_sim::tables::MortonTable<Point, T>,
    rng: &mut impl Rng,
) -> Point {
    loop {
        let x = rng.gen_range(0, 50);
        let y = rng.gen_range(0, 50);

        let pos = Point::new(x, y);

        if positions_table.get_by_id(&pos).is_none() {
            return pos;
        }
    }
}
