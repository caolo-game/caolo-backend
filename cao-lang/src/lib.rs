use arrayvec::ArrayVec;
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::mem;

pub type NodeId = i64;
pub type TPointer = usize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionError {
    InvalidInstruction,
    InvalidArgument,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum Instruction {
    Add = 1,
    Sub = 2,
}

impl TryFrom<u8> for Instruction {
    type Error = String;

    fn try_from(c: u8) -> Result<Instruction, Self::Error> {
        match c {
            1 => Ok(Instruction::Add),
            2 => Ok(Instruction::Sub),
            _ => Err(format!("Unrecognized instruction [{}]", c)),
        }
    }
}

#[derive(Debug)]
pub struct VM {
    stack: ArrayVec<[Instruction; 512]>,
    memory: Vec<u8>,
}

pub trait ByteEncodeProperties: Sized + Clone + Copy {
    const BYTELEN: usize = mem::size_of::<Self>();

    fn encode(self) -> Vec<u8> {
        let size: usize = Self::BYTELEN;

        let mut result = vec![0; size];
        unsafe {
            let dayum = std::mem::transmute::<*const Self, *const u8>(&self as *const Self);
            for i in 0..size {
                result[i] = *(dayum.add(i));
            }
        }
        result
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        let size: usize = Self::BYTELEN;
        if bytes.len() < size {
            None
        } else {
            let result = unsafe { *(bytes.as_ptr() as *const Self) };
            Some(result)
        }
    }
}

impl ByteEncodeProperties for i8 {}
impl ByteEncodeProperties for i16 {}
impl ByteEncodeProperties for i32 {}
impl ByteEncodeProperties for i64 {}
impl ByteEncodeProperties for u8 {}
impl ByteEncodeProperties for u16 {}
impl ByteEncodeProperties for u32 {}
impl ByteEncodeProperties for u64 {}
impl ByteEncodeProperties for f32 {}
impl ByteEncodeProperties for f64 {}
impl ByteEncodeProperties for TPointer {}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Default::default(),
            memory: vec![],
        }
    }

    pub fn get_value<T: ByteEncodeProperties>(&self, ptr: TPointer) -> Option<T> {
        let size: usize = T::BYTELEN;
        if ptr + size <= self.memory.len() {
            T::decode(&self.memory[ptr..])
        } else {
            None
        }
    }

    pub fn set_value<T: ByteEncodeProperties>(&mut self, val: T) -> TPointer {
        let result = self.memory.len();
        let bytes = val.encode();
        self.memory.extend(bytes.iter());

        result
    }

    pub fn set_value_at<T: ByteEncodeProperties>(&mut self, ptr: TPointer, val: T) {
        let bytes = val.encode();

        self.memory[ptr..bytes.len()].copy_from_slice(&bytes);
    }

    pub fn run(&mut self, program: &[u8]) -> Result<(), ExecutionError> {
        let mut ptr = 0;

        while ptr < program.len() {
            let instr = Instruction::try_from(program[ptr]).map_err(|_| {
                ExecutionError::InvalidInstruction
            })?;
            self.stack.push(instr);
            match instr {
                Instruction::Add => {
                    let len = TPointer::BYTELEN;
                    let ptr1 = ptr + 1;
                    let ptr2 = ptr1 + len;
                    ptr = ptr2 + len;
                    let ptr1 = TPointer::decode(&program[ptr1..ptr1+len]).unwrap();
                    let ptr2 = TPointer::decode(&program[ptr2..ptr2+len]).unwrap();
                    let a = self
                        .get_value::<i32>(ptr1)
                        .ok_or(ExecutionError::InvalidArgument)?;
                    let b = self
                        .get_value::<i32>(ptr2)
                        .ok_or(ExecutionError::InvalidArgument)?;
                    self.set_value_at(ptr1, a + b);
                    self.stack.pop();
                }
                Instruction::Sub => {
                    let len = TPointer::BYTELEN;
                    let ptr1 = ptr + 1;
                    let ptr2 = ptr1 + TPointer::BYTELEN;
                    ptr = ptr2 + len;
                    let ptr1 = TPointer::decode(&program[ptr1..ptr1+len]).unwrap();
                    let ptr2 = TPointer::decode(&program[ptr2..ptr2+len]).unwrap();
                    let a = self
                        .get_value::<i32>(ptr1)
                        .ok_or(ExecutionError::InvalidArgument)?;
                    let b = self
                        .get_value::<i32>(ptr2)
                        .ok_or(ExecutionError::InvalidArgument)?;
                    self.set_value_at(ptr1, a - b);
                    self.stack.pop();
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode(){
        let value: TPointer = 12342;
        let encoded = value.encode();
        println!("{:?}", encoded);
        let decoded = TPointer::decode(&encoded).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn test_add() {
        let mut vm = VM::new();

        let ptr = vm.set_value::<i32>(512);
        let ptr2 = vm.set_value::<i32>(42);

        let mut program = vec![Instruction::Add as u8];
        program.append(&mut <TPointer as ByteEncodeProperties>::encode(ptr));
        program.append(&mut <TPointer as ByteEncodeProperties>::encode(ptr2));

        vm.run(&program).unwrap();

        let result = vm
            .get_value::<i32>(ptr)
            .expect("Expected to read the result");

        assert_eq!(result, 512 + 42);
    }

    #[test]
    fn test_sub() {
        let mut vm = VM::new();

        let ptr = vm.set_value::<i32>(512);
        let ptr2 = vm.set_value::<i32>(42);

        let mut program = vec![Instruction::Sub as u8];
        program.append(&mut <TPointer as ByteEncodeProperties>::encode(ptr));
        program.append(&mut <TPointer as ByteEncodeProperties>::encode(ptr2));

        vm.run(&program).unwrap();

        let result = vm
            .get_value::<i32>(ptr)
            .expect("Expected to read the result");

        assert_eq!(result, 512 - 42);
    }
}
