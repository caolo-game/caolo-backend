use caolo_api::{AstNode, CompilationUnit, InputString, Instruction, Script, ScriptId, Value};
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
                nodes: [
                    (
                        0,
                        AstNode {
                            instruction: Instruction::LiteralPtr,
                        },
                    ),
                    (
                        1,
                        AstNode {
                            instruction: Instruction::Call,
                        },
                    ),
                ]
                .into_iter()
                .cloned()
                .collect(),
                values: [(0, Value::Pointer(0))].into_iter().cloned().collect(),
                inputs: [(1, [0].into_iter().cloned().collect())]
                    .into_iter()
                    .cloned()
                    .collect(),
                strings: [(1, InputString::from("say_hi").unwrap())]
                    .into_iter()
                    .cloned()
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
