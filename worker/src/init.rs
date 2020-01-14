use cao_lang::prelude::*;
use caolo_api::{point::Point, Script, ScriptId};
use caolo_sim::model;
use caolo_sim::storage::Storage;

const PROGRAM: &str = r#"{"nodes":{"0":{"node":{"Start":null},"children":[1]},"1":{"node":{"ScalarInt":{"value":420}},"children":[2]},"2":{"node":{"ScalarInt":{"value":69}},"children":[3]},"3":{"node":{"Call":{"function":"make_point"}},"children":[4]},"4":{"node":{"Call":{"function":"bots::move_bot"}},"children":[]}},"name":"placeholder"}"#;

pub fn init_storage(n_fake_users: usize) -> Storage {
    let mut storage = caolo_sim::init_inmemory_storage();

    let script_id = ScriptId::default(); // TODO randomize
    let script_id = model::ScriptId(script_id);
    let script: CompilationUnit =
        serde_json::from_str(PROGRAM).expect("deserialize example program");
    let compiled = Compiler::compile(script.clone()).expect("failed to compile example program");
    storage
        .scripts_table_mut::<model::ScriptComponent>()
        .insert_or_update(
            script_id,
            model::ScriptComponent(Script {
                compiled: Some(compiled),
                script: script,
            }),
        );

    let mut rng = rand::thread_rng();

    let terrain = storage.point_table_mut::<model::TerrainComponent>();

    for _ in 0..1 << 13 {
        let pos = uncontested_pos(terrain, &mut rng);
        terrain.insert(pos, model::TerrainComponent(model::TileTerrainType::Wall));
    }

    for _ in 0..n_fake_users {
        let id = storage.insert_entity();
        storage
            .entity_table_mut::<model::EntityScript>()
            .insert_or_update(id, model::EntityScript { script_id });
        storage
            .entity_table_mut::<model::Bot>()
            .insert_or_update(id, model::Bot {});
        storage
            .entity_table_mut::<model::CarryComponent>()
            .insert_or_update(id, Default::default());
        storage
            .entity_table_mut::<model::OwnedEntity>()
            .insert_or_update(
                id,
                model::OwnedEntity {
                    owner_id: Default::default(),
                },
            );

        let pos = {
            let entities_by_pos = storage.point_table::<model::EntityComponent>();
            uncontested_pos(entities_by_pos, &mut rng)
        };

        let positions = storage.entity_table_mut::<model::PositionComponent>();
        positions.insert_or_update(id, model::PositionComponent(pos));
    }
    storage
}

fn uncontested_pos<T: caolo_sim::tables::TableRow + Send + Sync>(
    positions_table: &caolo_sim::tables::MortonTable<Point, T>,
    rng: &mut rand::rngs::ThreadRng,
) -> caolo_api::point::Point {
    use rand::Rng;

    let mut pos = Point::default();
    loop {
        pos.x = rng.gen_range(0, 2000);
        pos.y = rng.gen_range(0, 2000);

        if positions_table.get_by_id(&pos).is_none() {
            break;
        }
    }
    pos
}
