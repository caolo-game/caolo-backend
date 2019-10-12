use caolo_engine::model::*;
use caolo_engine::storage::Storage;
use caolo_engine::tables::{PositionTable, Table, UserDataTable};
use rand::Rng;

pub fn init_storage(n_fake_users: usize, storage: &mut Storage) {
    debug!("Init InMemoryStorage");

    let mut users = Vec::with_capacity(n_fake_users);
    let bots = Table::default_inmemory();
    let spawning_bots = Table::default_inmemory();
    let bot_decay_table = Table::default_inmemory();
    let mut userdata = Table::default_inmemory();

    let mut structures = Table::default_inmemory();
    let mut hp_table = Table::default_inmemory();
    let mut structure_energy_regen_table = Table::default_inmemory();
    let mut structure_energy_table = Table::default_inmemory();
    let mut structure_spawn_table = Table::default_inmemory();
    let mut positions_table = Table::default_inmemory();
    let mut resources_table = Table::default_inmemory();

    let mut rng = rand::thread_rng();
    for _ in 0..8 {
        let entity_id = storage.insert_entity();
        let pos = uncontested_pos(&positions_table, &mut rng);
        resources_table.insert(entity_id, Resource::Mineral);
        positions_table.insert(entity_id, PositionComponent(pos));
        structure_energy_table.insert(
            entity_id,
            EnergyComponent {
                energy: 250,
                energy_max: 250,
            },
        );
    }
    for _ in 0..n_fake_users {
        let pos = uncontested_pos(&positions_table, &mut rng);
        let ud = UserData::new(None, None);
        let id = userdata.create_new(ud);
        users.push(id);
        // init spawn
        let entity_id = storage.insert_entity();
        structures.insert(entity_id, Structure { owner_id: Some(id) });
        positions_table.insert(entity_id, PositionComponent(pos));
        hp_table.insert(
            entity_id,
            HpComponent {
                hp: 500,
                hp_max: 500,
            },
        );
        structure_energy_regen_table.insert(entity_id, EnergyRegenComponent { amount: 1 });
        structure_energy_table.insert(
            entity_id,
            EnergyComponent {
                energy: 250,
                energy_max: 250,
            },
        );
        structure_spawn_table.insert(
            entity_id,
            SpawnComponent {
                time_to_spawn: 0,
                spawning: None,
            },
        );
    }

    let mut terrain = Table::default_inmemory();
    for _ in 0..100 {
        let pos = uncontested_pos(&positions_table, &mut rng);
        terrain.insert(pos, TileTerrainType::Wall);
    }

    storage.add_entity_table::<Bot>(bots);
    storage.add_entity_table::<SpawnBotComponent>(spawning_bots);
    storage.add_entity_table::<DecayComponent>(bot_decay_table);
    storage.add_entity_table::<CarryComponent>(Table::default_inmemory());
    storage.add_entity_table(hp_table);
    storage.add_entity_table(structures);
    storage.add_entity_table(structure_energy_regen_table);
    storage.add_entity_table(structure_energy_table);
    storage.add_entity_table(structure_spawn_table);
    storage.add_entity_table(positions_table);
    storage.add_entity_table(resources_table);

    storage.add_user_table(userdata);
    storage.add_point_table(terrain);

    debug!("Init InMemoryStorage done");
}

fn uncontested_pos(positions_table: &dyn PositionTable, rng: &mut rand::rngs::ThreadRng) -> Point {
    let mut pos = Point::default();
    loop {
        pos.x = rng.gen_range(-19, 20);
        pos.y = rng.gen_range(-19, 20);

        if positions_table.count_entities_in_range(&Circle {
            center: pos,
            radius: 0,
        }) == 0
        {
            break;
        }
    }
    pos
}
