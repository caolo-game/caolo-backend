use caolo_api::{Script, ScriptId};
use caolo_engine::model::EntityScript;
use caolo_engine::storage::Storage;

const PROGRAM: &str = r#"{
    "nodes": {
        "0": {
            "instruction": "ScalarInt",
            "scalar": {
                "Integer": 4
            }
        },
        "1": {
            "instruction": "ScalarInt",
            "scalar": {
                "Integer": 5
            }
        },
        "2": {
            "instruction": "Add"
        },
        "3": {
            "instruction": "Call",
            "string": "log_scalar"
        }
    },
    "inputs": {
        "2": [
            0,
            1
        ],
        "3": [
            2
        ]
    }
}"#;

pub fn init_storage(n_fake_users: usize) -> Storage {
    let mut storage = caolo_engine::init_inmemory_storage();

    let script_id = ScriptId::default(); // TODO randomize
    storage.scripts_table_mut::<Script>().insert(
        script_id,
        Script {
            compiled: None,
            script: serde_json::from_str(PROGRAM).expect("deserialize"),
        },
    );

    for _ in 0..n_fake_users {
        let id = storage.insert_entity();
        storage
            .entity_table_mut::<EntityScript>()
            .insert(id, EntityScript { script_id });
    }
    storage
}
