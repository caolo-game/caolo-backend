use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
/// Single instruction of the interpreter
pub enum Instruction {
    /// Add two numbers, write the result in the first memory location
    AddInt = 1,
    /// Subtract two numbers, write the result in the first memory location
    SubInt = 2,
    /// Add two numbers, write the result in the first memory location
    AddFloat = 3,
    /// Subtract two numbers, write the result in the first memory location
    SubFloat = 4,
    /// Multiply two numbers, write the result in the first memory location
    Mul = 5,
    /// Multiply two numbers, write the result in the first memory location
    MulFloat = 6,
    /// Divide the first number by the second
    Div = 7,
    /// Divide the first number by the second
    DivFloat = 8,
    /// Call a function provided by the runtime
    /// Requires function name as a string as input
    Call = 9,
    /// Push an int onto the stack
    ScalarInt = 10,
    /// Push a float onto the stack
    ScalarFloat = 11,
    /// Push a ptr onto the stack
    ScalarPtr = 12,
    /// Push a label onto the stack
    ScalarLabel = 17,
    /// Pop the next N (positive integer) number of items from the stack and write them to memory
    /// Push the pointer to the beginning of the array onto the stack
    ScalarArray = 13,
    /// Empty instruction that has no effects
    Pass = 14,
    /// Clones the last element on the stack
    /// Does nothing if no elements are on the stack
    CopyLast = 15,
    /// Branching (If-Else) instruction
    /// If the value at the top of the stack is truthy jumps to the
    /// first index else jumps to the second index
    Branch = 16,
    /// Quit the program
    /// Implicitly inserted by the compiler after every leaf node
    Exit = 18,
}

impl TryFrom<u8> for Instruction {
    type Error = String;

    fn try_from(c: u8) -> Result<Instruction, Self::Error> {
        use Instruction::*;
        match c {
            1 => Ok(AddInt),
            2 => Ok(SubInt),
            3 => Ok(AddFloat),
            4 => Ok(SubFloat),
            5 => Ok(Mul),
            6 => Ok(MulFloat),
            7 => Ok(Div),
            8 => Ok(DivFloat),
            9 => Ok(Call),
            10 => Ok(ScalarInt),
            11 => Ok(ScalarFloat),
            12 => Ok(ScalarPtr),
            13 => Ok(ScalarArray),
            14 => Ok(Pass),
            15 => Ok(CopyLast),
            16 => Ok(Branch),
            17 => Ok(ScalarLabel),
            18 => Ok(Exit),
            _ => Err(format!("Unrecognized instruction [{}]", c)),
        }
    }
}

