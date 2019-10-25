use crate::{CompiledProgram, Instruction, Value};
use arrayvec::{ArrayString, ArrayVec};
use serde_derive::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;

/// Unique id of each nodes in a single compilation
pub type NodeId = i32;
/// Node by given id has inputs given by nodeids
/// Nodes may only have a finite amount of inputs
pub type Inputs = HashMap<NodeId, ArrayVec<[NodeId; 16]>>;
pub type Nodes = BTreeMap<NodeId, AstNode>;
/// Value of a node if any
pub type Values = HashMap<NodeId, Value>;
/// String of a node if any
pub type Strings = HashMap<NodeId, InputString>;

const INPUT_STR_LEN: usize = 128;
pub type InputString = ArrayString<[u8; INPUT_STR_LEN]>;

impl crate::ByteEncodeProperties for InputString {
    const BYTELEN: usize = INPUT_STR_LEN;

    fn encode(self) -> Vec<u8> {
        let mut rr = (self.len() as i32).encode();
        rr.extend(self.chars().map(|c| c as u8));
        rr
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        let len = i32::decode(bytes)?;
        let mut res = Self::new();
        for byte in bytes
            .iter()
            .skip(i32::BYTELEN)
            .take(len as usize)
            .map(|c| *c as char)
        {
            res.push(byte);
        }
        Some(res)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    instruction: Instruction,
}

impl AstNode {
    pub fn new(instruction: Instruction) -> Self {
        Self { instruction }
    }
}

/// The accepted number of inputs of an instruction
/// None if unspecified
pub fn input_per_instruction(inst: Instruction) -> Option<u8> {
    use Instruction::*;
    match inst {
        AddInt | SubInt | AddFloat | SubFloat | Mul | MulFloat | Div | DivFloat => Some(2),
        Branch => Some(3),
        LiteralLabel | LiteralInt | LiteralFloat | LiteralPtr | Pass | CopyLast => Some(0),
        Exit | Call | LiteralArray => None,
    }
}

/// Single unit of compilation, representing a single program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationUnit {
    nodes: Nodes,
    inputs: Inputs,
    values: Values,
    strings: Strings,
}

pub struct Compiler {
    unit: CompilationUnit,
    program: CompiledProgram,
}

impl Compiler {
    pub fn compile(unit: CompilationUnit) -> Result<CompiledProgram, String> {
        if unit.nodes.is_empty() {
            return Err("Can not compile program with no entry point!".to_owned());
        }
        let mut compiler = Compiler {
            unit,
            program: CompiledProgram::default(),
        };
        let leafs: Vec<NodeId> = compiler
            .unit
            .nodes
            .iter()
            .map(|(k, _)| k)
            .filter(|k| {
                for it in compiler.unit.inputs.values() {
                    for n in it {
                        if n == *k {
                            return false;
                        }
                    }
                }
                true
            })
            .cloned()
            .collect();

        for nodeid in leafs.into_iter() {
            compiler.process_node(nodeid)?;
            compiler.program.bytecode.push(Instruction::Exit as u8);
        }

        Ok(compiler.program)
    }

    fn process_node(&mut self, nodeid: NodeId) -> Result<(), String> {
        use crate::traits::ByteEncodeProperties;
        use Instruction::*;

        Compiler::validate_node(nodeid, &mut self.unit)?;
        self.program
            .labels
            .insert(nodeid, self.program.bytecode.len());

        if let Some(inputs) = self.unit.inputs.get(&nodeid) {
            for nodeid in inputs.clone().into_iter() {
                self.process_node(nodeid)?;
            }
        }
        let instruction = {
            let node = &self.unit.nodes[&nodeid];
            node.instruction
        };
        if input_per_instruction(instruction)
            .map(usize::from)
            .map(|n| {
                n != 0
                    && self
                        .unit
                        .inputs
                        .get(&nodeid)
                        .map(|x| x.len() != n)
                        .unwrap_or(true)
            })
            .unwrap_or(false)
        {
            return Err(format!(
                "{:?} received invalid input. Expected: {:?} Actual: {:?}",
                instruction,
                input_per_instruction(instruction),
                self.unit.inputs.get(&nodeid)
            ));
        }

        match instruction {
            Exit | Pass | CopyLast | Branch | AddFloat | AddInt | SubFloat | SubInt | Mul
            | MulFloat | Div | DivFloat => {
                self.push_node(nodeid);
            }
            Call => {
                self.push_node(nodeid);
                self.program
                    .bytecode
                    .append(&mut self.unit.strings[&nodeid].encode());
            }
            LiteralArray => {
                self.push_node(nodeid);
                match self.unit.values[&nodeid] {
                    Value::IValue(v) => {
                        if self
                            .unit
                            .inputs
                            .get(&nodeid)
                            .map(|x| x.len() != v as usize)
                            .unwrap_or(v != 0)
                        {
                            return Err("Array literal got invalid inputs".to_owned());
                        }

                        self.program.bytecode.append(&mut v.encode());
                    }
                    _ => panic!(
                        "LiteralArray got invalid value {:?}",
                        self.unit.values[&nodeid]
                    ),
                }
            }
            LiteralLabel | LiteralPtr | LiteralFloat | LiteralInt => {
                self.push_node(nodeid);
                match (instruction, self.unit.values[&nodeid]) {
                    (Instruction::LiteralInt, Value::IValue(v)) => {
                        self.program.bytecode.append(&mut v.encode());
                    }
                    (Instruction::LiteralFloat, Value::FValue(v)) => {
                        self.program.bytecode.append(&mut v.encode());
                    }
                    (Instruction::LiteralPtr, Value::Pointer(v)) => {
                        self.program.bytecode.append(&mut v.encode());
                    }
                    (Instruction::LiteralLabel, Value::Label(v)) => {
                        self.program.bytecode.append(&mut v.encode());
                    }
                    _ => panic!(
                        "Literal {:?} got invalid value {:?}",
                        instruction, self.unit.values[&nodeid]
                    ),
                }
            }
        }
        Ok(())
    }

    fn push_node(&mut self, nodeid: NodeId) {
        let node = &self.unit.nodes[&nodeid];
        self.program.bytecode.push(node.instruction as u8);
    }

    pub fn validate_node(node: NodeId, cu: &CompilationUnit) -> Result<(), String> {
        if let Some(n) = input_per_instruction(cu.nodes[&node].instruction) {
            let n_inputs = cu.inputs.get(&node).map(|x| x.len()).unwrap_or(0)
                + cu.strings.get(&node).map(|_| 1).unwrap_or(0);
            if n_inputs != n as usize {
                return Err(format!(
                    "Invalid number of inputs, expected {} got {}",
                    n, n_inputs
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VM;

    #[test]
    fn test_compiling_simple_program() {
        let nodes: Nodes = [
            (
                0,
                AstNode {
                    instruction: Instruction::LiteralFloat,
                },
            ),
            (
                1,
                AstNode {
                    instruction: Instruction::LiteralFloat,
                },
            ),
            (
                2,
                AstNode {
                    instruction: Instruction::AddFloat,
                },
            ),
        ]
        .into_iter()
        .cloned()
        .collect();
        let values: Values = [(0, Value::FValue(42.0)), (1, Value::FValue(512.0))]
            .into_iter()
            .map(|x| *x)
            .collect();
        let inputs: Inputs = [(2, [0, 1].into_iter().cloned().collect())]
            .into_iter()
            .cloned()
            .collect();
        let strings: Strings = [].into_iter().cloned().collect();
        let program = CompilationUnit {
            nodes,
            values,
            inputs,
            strings,
        };

        let program = Compiler::compile(program).unwrap();

        println!("{:?}", program);

        // Compilation was successful

        let mut vm = VM::new();
        vm.run(&program).unwrap();

        assert_eq!(vm.stack.len(), 1, "{:?}", vm.stack);

        let value = vm.stack.last().unwrap();
        match value {
            Value::FValue(i) => assert_eq!(*i, 42.0 + 512.0),
            _ => panic!("Invalid value in the stack"),
        }
    }

    /// Add val1 and val2 if true else subtract
    fn simple_branch_test(val1: f32, val2: f32, cond: i32, expected: f32) {
        let nodes: Nodes = [
            (
                10,
                AstNode {
                    instruction: Instruction::LiteralFloat,
                },
            ),
            (
                1,
                AstNode {
                    instruction: Instruction::LiteralFloat,
                },
            ),
            (
                2,
                AstNode {
                    instruction: Instruction::AddFloat,
                },
            ),
            (
                5,
                AstNode {
                    instruction: Instruction::SubFloat,
                },
            ),
            (
                6,
                AstNode {
                    instruction: Instruction::LiteralInt, // Cond
                },
            ),
            (
                7,
                AstNode {
                    instruction: Instruction::LiteralLabel, // True
                },
            ),
            (
                8,
                AstNode {
                    instruction: Instruction::LiteralLabel, // False
                },
            ),
            (
                0,
                AstNode {
                    instruction: Instruction::Branch,
                },
            ),
        ]
        .into_iter()
        .cloned()
        .collect();
        let values: Values = [
            (10, Value::FValue(val1)),
            (1, Value::FValue(val2)),
            (6, Value::IValue(cond)),
            (7, Value::Label(2)),
            (8, Value::Label(5)),
        ]
        .into_iter()
        .map(|x| *x)
        .collect();
        let inputs: Inputs = [
            (2, [10, 1].into_iter().cloned().collect()),
            (5, [10, 1].into_iter().cloned().collect()),
            (0, [6, 7, 8].into_iter().cloned().collect()),
        ]
        .into_iter()
        .cloned()
        .collect();
        let strings: Strings = [].into_iter().cloned().collect();
        let program = CompilationUnit {
            nodes,
            values,
            inputs,
            strings,
        };

        let program = Compiler::compile(program).expect("compile");

        let mut vm = VM::new();
        if let Err(e) = vm.run(&program) {
            panic!("Err:{:?}\n{:?}", e, vm);
        }

        assert_eq!(vm.stack.len(), 1);

        let value = vm.stack.last().unwrap();
        match value {
            Value::FValue(i) => assert_eq!(*i, expected),
            _ => panic!("Invalid value in the stack"),
        }
    }

    #[test]
    fn test_branching_true() {
        simple_logger::init().unwrap_or(());
        simple_branch_test(42.0, 512.0, 1, 42.0 + 512.0);
    }

    #[test]
    fn test_branching_false() {
        simple_logger::init().unwrap_or(());
        simple_branch_test(42.0, 512.0, 0, 42.0 - 512.0);
    }
}
