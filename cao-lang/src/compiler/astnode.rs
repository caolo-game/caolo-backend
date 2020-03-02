use super::*;
use crate::InputString;
use crate::VarName;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    pub node: InstructionNode,
    pub child: Option<NodeId>,
}

impl Default for AstNode {
    fn default() -> Self {
        Self {
            node: InstructionNode::Pass,
            child: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstructionNode {
    Start,
    Pass,
    Add,
    Sub,
    Mul,
    Div,
    Exit,
    CopyLast,
    Less,
    LessOrEq,
    Equals,
    NotEquals,
    Pop,
    ScalarInt(IntegerNode),
    ScalarFloat(FloatNode),
    ScalarLabel(IntegerNode),
    ScalarArray(IntegerNode),
    StringLiteral(StringNode),
    Call(CallNode),
    JumpIfTrue(JumpNode),
    Jump(JumpNode),
    SetVar(VarNode),
    ReadVar(VarNode),
}

impl InstructionNode {
    pub fn instruction(&self) -> Instruction {
        use InstructionNode::*;
        match self {
            Start => Instruction::Start,
            Pass => Instruction::Pass,
            Add => Instruction::Add,
            Sub => Instruction::Sub,
            Mul => Instruction::Mul,
            Div => Instruction::Div,
            Exit => Instruction::Exit,
            CopyLast => Instruction::CopyLast,
            Less => Instruction::Less,
            LessOrEq => Instruction::LessOrEq,
            Equals => Instruction::Equals,
            NotEquals => Instruction::NotEquals,
            Pop => Instruction::Pop,
            ScalarInt(_) => Instruction::ScalarInt,
            ScalarFloat(_) => Instruction::ScalarFloat,
            ScalarArray(_) => Instruction::ScalarArray,
            ScalarLabel(_) => Instruction::ScalarLabel,
            Call(_) => Instruction::Call,
            JumpIfTrue(_) => Instruction::JumpIfTrue,
            Jump(_) => Instruction::Jump,
            StringLiteral(_) => Instruction::StringLiteral,
            SetVar(_) => Instruction::SetVar,
            ReadVar(_) => Instruction::ReadVar,
        }
    }

    // Trigger compilation errors for newly added instructions so we don't forget implementing them
    // here
    #[allow(unused)]
    fn _instruction_to_node(instr: Instruction) {
        use Instruction::*;
        match instr {
            SetVar | ReadVar | Pop | Less | LessOrEq | Equals | NotEquals | Exit
            | StringLiteral | Start | JumpIfTrue | Jump | CopyLast | Call | Sub | Mul | Div
            | ScalarArray | ScalarLabel | ScalarFloat | ScalarInt | Add | Pass => {}
        };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Copy)]
pub struct IntegerNode {
    pub value: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default, Copy)]
pub struct FloatNode {
    pub value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CallNode {
    pub function: InputString,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StringNode {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Copy)]
pub struct JumpNode {
    pub nodeid: NodeId,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Copy)]
pub struct VarNode {
    pub name: VarName,
}
