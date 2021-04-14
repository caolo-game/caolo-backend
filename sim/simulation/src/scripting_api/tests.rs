use crate::prelude::World;
use cao_lang::prelude::*;

use super::*;

fn init_basic_storage() -> std::pin::Pin<Box<World>> {
    World::new()
}

#[test]
fn test_parse_world_position() {
    let storage = init_basic_storage();
    let data = ScriptExecutionData::new(
        &*storage.as_ref(),
        Default::default(),
        Default::default(),
        Default::default(),
    );
    let mut vm = Vm::new(data).unwrap();

    fn test_parse(
        _vm: &mut Vm<ScriptExecutionData>,
        inp: *mut FieldTable,
    ) -> Result<(), ExecutionError> {
        let res = unsafe { parse_world_pos(&*inp)? };
        assert_eq!(
            res,
            WorldPosition {
                room: Axial::new(1, 2),
                pos: Axial::new(3, 4)
            }
        );
        Ok(())
    }

    vm.register_function("WorldPosition", into_f1(test_parse));

    const PROGRAM: &str = r#"
lanes:
    - cards:
        - ty: CreateTable
        - ty: SetVar
          val: pos

        - ty: ScalarInt
          val: 1
        - ty: ReadVar
          val: pos
        - ty: SetProperty
          val: rq

        - ty: ScalarInt
          val: 2
        - ty: ReadVar
          val: pos
        - ty: SetProperty
          val: rr

        - ty: ScalarInt
          val: 3
        - ty: ReadVar
          val: pos
        - ty: SetProperty
          val: q

        - ty: ScalarInt
          val: 4
        - ty: ReadVar
          val: pos
        - ty: SetProperty
          val: r

        - ty: ReadVar
          val: pos
        - ty: CallNative
          val: "WorldPosition"
"#;

    let program = serde_yaml::from_str(PROGRAM).unwrap();
    let program = compile(program, None).unwrap();

    vm.run(&program).unwrap();
}
