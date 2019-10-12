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
    MulFloat=6,
    /// Divide the first number by the second
    Div=7,
    /// Divide the first number by the second
    DivFloat=8,
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
            5 => Ok(Mul) ,
            6 => Ok(MulFloat),
            7 => Ok(Div),
            8 => Ok(DivFloat),
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
impl ByteEncodeProperties for u8 {}
impl ByteEncodeProperties for u16 {}
impl ByteEncodeProperties for u32 {}
impl ByteEncodeProperties for f32 {}
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
            T::decode(&self.memory[ptr..ptr+size])
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
        self.memory[ptr..ptr+bytes.len()].copy_from_slice(&bytes[..]);
    }

    pub fn run(&mut self, program: &[u8]) -> Result<(), ExecutionError> {
        let mut ptr = 0;
        while ptr < program.len() {
            let instr = Instruction::try_from(program[ptr]).map_err(|_| {
                ExecutionError::InvalidInstruction
            })?;
            self.stack.push(instr);
            match instr {
                Instruction::AddInt => {
                    ptr += 1;
                    self.binary_op::<i32, _>(&mut ptr, |a,b|a+b, program)?;
                }
                Instruction::SubInt => {
                    ptr += 1;
                    self.binary_op::<i32, _>(&mut ptr, |a,b|a-b, program)?;
                }
                Instruction::AddFloat => {
                    ptr += 1;
                    self.binary_op::<f32, _>(&mut ptr, |a,b|a+b, program)?;
                }
                Instruction::SubFloat => {
                    ptr += 1;
                    self.binary_op::<f32, _>(&mut ptr, |a,b|a-b, program)?;
                }
                Instruction::Mul => {
                    ptr += 1;
                    self.binary_op::<i32, _>(&mut ptr, |a,b|a*b, program)?;
                }
                Instruction::MulFloat => {
                    ptr += 1;
                    self.binary_op::<f32, _>(&mut ptr, |a,b|a*b, program)?;
                }
                Instruction::Div=> {
                    ptr += 1;
                    self.binary_op::<i32, _>(&mut ptr, |a,b|a/b, program)?;
                }
                Instruction::DivFloat => {
                    ptr += 1;
                    self.binary_op::<f32, _>(&mut ptr, |a,b|a/b, program)?;
                }
            }
        }

        Ok(())
    }

    fn binary_op<T: ByteEncodeProperties, F: Fn(T,T)->T>(&mut self, ptr: &mut TPointer, op: F, program: &[u8]) -> Result<(), ExecutionError> {
        let (ptr1, a) = self.load::<T>(ptr, program)
            .ok_or(ExecutionError::InvalidArgument)?;
        let (_, b) = self.load::<T>(ptr, program)
            .ok_or(ExecutionError::InvalidArgument)?;
        self.set_value_at(ptr1, op(a, b));
        self.stack.pop();
        Ok(())
    }

    fn load<T: ByteEncodeProperties>(&self, ptr: &mut usize, program: &[u8]) -> Option<(TPointer, T)> {
        let len = TPointer::BYTELEN;
        let p = *ptr;
        let ptr1 = TPointer::decode(&program[p..p+len])?;
        *ptr = *ptr + len;
        self.get_value::<T>(ptr1).map(|x|(ptr1, x))
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
    fn test_binary_operatons () {
        let mut vm = VM::new();

        let ptr1 = vm.set_value::<i32>(512);
        let ptr2 = vm.set_value::<i32>(42);

        let mut program = vec![];
        program.append(&mut <TPointer as ByteEncodeProperties>::encode(ptr1));
        program.append(&mut <TPointer as ByteEncodeProperties>::encode(ptr2));
        let mut ptr = 0;
        vm.binary_op::<i32, _>(&mut ptr, |a,b|(a+a/b)*b, &program).unwrap();
        let result = vm
            .get_value::<i32>(ptr1)
            .expect("Expected to read the result");
        assert_eq!(result, (512 + 512 / 42)*42);
    }

    fn test_binary_op<T: ByteEncodeProperties+PartialEq+std::fmt::Debug>(val1: T, val2: T, expected: T, inst: Instruction) {
        let mut vm = VM::new();

        let ptr1 = vm.set_value::<T>(val1);
        let ptr2 = vm.set_value::<T>(val2);

        let mut program = vec![inst as u8];
        program.append(&mut <TPointer as ByteEncodeProperties>::encode(ptr1));
        program.append(&mut <TPointer as ByteEncodeProperties>::encode(ptr2));
        vm.run(&program).unwrap();
        let result = vm
            .get_value::<T>(ptr1)
            .expect("Expected to read the result");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_add() {
        test_binary_op::<i32>(512, 42, 512+42, Instruction::AddInt);
    }

    #[test]
    fn test_sub() {
        test_binary_op::<i32>(512, 42, 512-42, Instruction::SubInt);
    }

    #[test]
    fn test_fadd() {
        test_binary_op::<f32>(512., 42., 512.+42., Instruction::AddFloat);
    }

    #[test]
    fn test_fsub() {
        test_binary_op::<f32>(512., 42., 512.-42., Instruction::SubFloat);
    }
}
