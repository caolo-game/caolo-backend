use caolo_api::{AstNode, CompilationUnit, InputString, Instruction, Script, ScriptId};
use caolo_engine::model::EntityScript;
use caolo_engine::storage::Storage;

pub fn init_storage(n_fake_users: usize) -> Storage {
    let mut storage = caolo_engine::init_inmemory_storage();

    let script_id = ScriptId::default(); // TODO randomize
    storage.scripts_table_mut::<Script>().insert(
        script_id,
        Script {
            compiled: None,
            script: CompilationUnit {
                nodes: vec![(
                    0,
                    AstNode {
                        instruction: Instruction::Call,
                    },
                )]
                .into_iter()
                .collect(),
                values: vec![].into_iter().collect(),
                inputs: vec![].into_iter().collect(),
                strings: vec![(0, InputString::from("say_hi").unwrap())]
                    .into_iter()
                    .collect(),
            },
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
