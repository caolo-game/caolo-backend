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

#[test]
fn test_say() {
    let mut storage = World::new();

    let entity_id = storage.insert_entity();

    let mut vm = Vm::new(ScriptExecutionData::new(
        &*storage.as_ref(),
        Default::default(),
        entity_id,
        Default::default(),
    ))
    .unwrap();

    const PROGRAM: &str = r#"
lanes:
    - cards:
        - ty: StringLiteral
          val: "pog"
        - ty: CallNative
          val: "say"
    "#;

    let program = serde_yaml::from_str(PROGRAM).unwrap();
    let program = compile(program, None).unwrap();

    vm.register_function("say", into_f1(say));
    vm.run(&program).unwrap();

    let intent = vm.unwrap_aux().intents.say_intent.unwrap();
    assert_eq!(intent.entity, entity_id);
    assert_eq!(intent.payload.as_str(), "pog");
}

#[test]
fn test_say_bad_len() {
    let mut storage = World::new();

    let entity_id = storage.insert_entity();

    let mut vm = Vm::new(ScriptExecutionData::new(
        &*storage.as_ref(),
        Default::default(),
        entity_id,
        Default::default(),
    ))
    .unwrap();

    const PROGRAM: &str = r#"
lanes:
    - cards:
        - ty: StringLiteral
          val: "pog"
        - ty: CallNative
          val: "sayasdsdadasdadasdasdasdasdsdsdsdsdsldkjskdjdlsjdklsjdklsjdklsjdaldasljdsldjkldsldjsldjkljaldjaldsljsljdsljd"
    "#;

    let program = serde_yaml::from_str(PROGRAM).unwrap();
    let program = compile(program, None).unwrap();

    vm.register_function("say", into_f1(say));
    vm.run(&program).unwrap_err();
}
