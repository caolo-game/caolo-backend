use super::*;
use crate::scalar::Scalar;
use crate::vm::VM;

#[test]
fn compiling_simple_program() {
    simple_logger::init().unwrap_or(());
    let nodes: Nodes = [
        (
            999,
            AstNode {
                node: InstructionNode::Start,
                child: Some(0),
            },
        ),
        (
            0,
            AstNode {
                node: InstructionNode::ScalarFloat(FloatNode { value: 42.0 }),
                child: Some(1),
            },
        ),
        (
            1,
            AstNode {
                node: InstructionNode::ScalarFloat(FloatNode { value: 512.0 }),
                child: Some(2),
            },
        ),
        (
            2,
            AstNode {
                node: InstructionNode::Add,
                child: None,
            },
        ),
    ]
    .iter()
    .cloned()
    .collect();

    let program = CompilationUnit { nodes };
    let program = Compiler::compile(program).unwrap();

    log::warn!("Program: {:?}", program);

    // Compilation was successful

    let mut vm = VM::new(());
    vm.run(&program).unwrap();

    assert_eq!(vm.stack().len(), 1, "{:?}", vm.stack());

    let value = vm.stack().last().unwrap();
    match value {
        Scalar::Floating(i) => assert_eq!(*i, 42.0 + 512.0),
        _ => panic!("Invalid value in the stack"),
    }
}

#[test]
fn simple_looping_program() {
    simple_logger::init().unwrap_or(());
    let nodes: Nodes = [
        (
            999,
            AstNode {
                node: InstructionNode::Start,
                child: Some(0),
            },
        ),
        (
            0,
            AstNode {
                node: InstructionNode::ScalarInt(IntegerNode { value: 4 }),
                child: Some(1),
            },
        ),
        (
            1,
            AstNode {
                node: InstructionNode::ScalarInt(IntegerNode { value: 1 }),
                child: Some(2),
            },
        ),
        (
            3,
            AstNode {
                node: InstructionNode::CopyLast,
                child: Some(4),
            },
        ),
        (
            2,
            AstNode {
                node: InstructionNode::Sub,
                child: Some(3),
            },
        ),
        (
            4,
            AstNode {
                node: InstructionNode::JumpIfTrue(JumpNode { nodeid: 5 }),
                child: None,
            },
        ),
        (
            5,
            AstNode {
                node: InstructionNode::CopyLast,
                child: Some(1),
            },
        ),
    ]
    .iter()
    .cloned()
    .collect();

    let program = CompilationUnit { nodes };
    let program = Compiler::compile(program).unwrap();

    log::warn!("Program: {:?}", program);

    // Compilation was successful

    let mut vm = VM::new(()).with_max_iter(50);
    let exit_code = vm.run(&program).unwrap();

    assert_eq!(exit_code, 0);
    assert_eq!(vm.stack().len(), 3, "{:?}", vm.stack());

    println!("stack: {:?}", vm.stack());
    for (i, value) in vm.stack().iter().enumerate() {
        match value {
            Scalar::Integer(num) => assert_eq!(*num, 3 - i as i32),
            _ => panic!("Invalid value on the stack"),
        }
    }
}
