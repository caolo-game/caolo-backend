use crate::prelude::World;
use cao_lang::prelude::*;

use super::*;

fn init_basic_storage() -> std::pin::Pin<Box<World>> {
    crate::utils::setup_testing();

    World::new(crate::utils::test_logger())
}

#[test]
fn test_world_position() {
    let storage = init_basic_storage();
    let logger = &storage.logger;
    let data = ScriptExecutionData::new(
        logger.clone(),
        &*storage.as_ref(),
        Default::default(),
        Default::default(),
        Default::default(),
    );
    let mut vm = Vm::new(data);

    vm.stack_push(1).unwrap();
    vm.stack_push(2).unwrap();
    vm.stack_push(3).unwrap();
    vm.stack_push(4).unwrap();

    // TODO

    vm.register_function("WorldPosition", into_f4(world_position));

    const PROGRAM: &str = r#"
lanes:
    - cards:
        - Call: "WorldPosition"
        - SetGlobalVar: "pos"
"#;

    let program = serde_yaml::from_str(PROGRAM).unwrap();
    let program = compile(program, None).unwrap();

    vm.run(&program).unwrap();

    let varid = program.variable_id("pos").unwrap();

    let res = vm.read_var(varid).unwrap();
    match res {
        Scalar::Null | Scalar::Integer(_) | Scalar::Floating(_) => {
            panic!("Expected pointer, found {:?}", res)
        }

        Scalar::Pointer(p) => {
            let pos: WorldPosition = vm.get_value(p).unwrap();

            assert_eq!(
                pos,
                WorldPosition {
                    room: Axial::new(1, 2),
                    pos: Axial::new(3, 4),
                }
            );
        }
    }
}
