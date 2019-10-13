mod traits;

pub use traits::*;

use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;

pub type TPointer = usize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionError {
    InvalidInstruction,
    InvalidArgument,
    FunctionNotFound(String),
    OutOfMemory,
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
    MulFloat = 6,
    /// Divide the first number by the second
    Div = 7,
    /// Divide the first number by the second
    DivFloat = 8,
    /// Moves the bot with entity id to the point and writes an OperationResult to the first
    /// pointer
    Call = 9,

    /// Push an int to the stack
    LiteralInt = 10,
    /// Push a float to the stack
    LiteralFloat = 11,
    /// Push a ptr to the stack
    LiteralPtr = 12,
    /// Pop the next N (positive integer) number of items from the stack and write them to memory
    /// Push the pointer to the beginning of the array to the stack
    LiteralArray = 13,
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
            10 => Ok(LiteralInt),
            11 => Ok(LiteralFloat),
            12 => Ok(LiteralPtr),
            13 => Ok(LiteralArray),
            _ => Err(format!("Unrecognized instruction [{}]", c)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Value {
    Pointer(TPointer),
    IValue(i32),
    FValue(f32),
}

impl AutoByteEncodeProperties for Value {}

/// Cao-Lang bytecode interpreter
#[derive(Debug)]
pub struct VM {
    memory: Vec<u8>,
    memory_limit: usize,
    callables: HashMap<&'static str, Box<dyn Callable>>,
    stack: Vec<Value>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            memory: Vec::with_capacity(512),
            callables: HashMap::with_capacity(128),
            memory_limit: 40000,
            stack: Vec::with_capacity(128),
        }
    }

    pub fn get_value<T: ByteEncodeProperties>(&self, ptr: TPointer) -> Option<T> {
        let size: usize = T::BYTELEN;
        if ptr + size <= self.memory.len() {
            T::decode(&self.memory[ptr..ptr + size])
        } else {
            None
        }
    }

    // TODO: check if maximum size was exceeded
    pub fn set_value<T: ByteEncodeProperties>(&mut self, val: T) -> TPointer {
        let result = self.memory.len();
        let bytes = val.encode();
        self.memory.extend(bytes.iter());

        result
    }

    // TODO: check if maximum size was exceeded
    pub fn set_value_at<T: ByteEncodeProperties>(&mut self, ptr: TPointer, val: T) {
        let bytes = val.encode();
        self.memory[ptr..ptr + bytes.len()].copy_from_slice(&bytes[..]);
    }

    pub fn run(&mut self, program: &[u8]) -> Result<(), ExecutionError> {
        let mut ptr = 0;
        while ptr < program.len() {
            let instr = Instruction::try_from(program[ptr])
                .map_err(|_| ExecutionError::InvalidInstruction)?;
            ptr += 1;
            match instr {
                Instruction::LiteralInt => {
                    let len = i32::BYTELEN;
                    self.stack.push(Value::IValue(
                        i32::decode(&program[ptr..ptr + len])
                            .ok_or(ExecutionError::InvalidArgument)?,
                    ));
                    ptr += len;
                }
                Instruction::LiteralFloat => {
                    let len = f32::BYTELEN;
                    self.stack.push(Value::FValue(
                        f32::decode(&program[ptr..ptr + len])
                            .ok_or(ExecutionError::InvalidArgument)?,
                    ));
                    ptr += len;
                }
                Instruction::LiteralPtr => {
                    let val = self.stack.pop().unwrap();
                    let ptr = self.memory.len();
                    self.memory.append(&mut val.encode());
                    self.stack.push(Value::Pointer(ptr));
                }
                Instruction::LiteralArray => {
                    let len = self
                        .load_int_from_stack()
                        .ok_or(ExecutionError::InvalidArgument)?;
                    if len > 128 || len > self.stack.len() as i32 {
                        return Err(ExecutionError::InvalidArgument)?;
                    }
                    let ptr = self.memory.len();
                    self.stack.pop();
                    for _ in 0..len {
                        let val = self.stack.pop().unwrap();
                        self.memory.append(&mut val.encode());
                    }
                    self.stack.push(Value::Pointer(ptr));
                }
                Instruction::AddInt => self.binary_op::<i32, _, _>(
                    |a, b| Value::IValue(a + b),
                    |s| s.load_int_from_stack(),
                )?,
                Instruction::AddFloat => self.binary_op::<f32, _, _>(
                    |a, b| Value::FValue(a + b),
                    |s| s.load_float_from_stack(),
                )?,
                Instruction::SubInt => self.binary_op::<i32, _, _>(
                    |a, b| Value::IValue(a - b),
                    |s| s.load_int_from_stack(),
                )?,
                Instruction::SubFloat => self.binary_op::<f32, _, _>(
                    |a, b| Value::FValue(a - b),
                    |s| s.load_float_from_stack(),
                )?,
                Instruction::Mul => self.binary_op::<i32, _, _>(
                    |a, b| Value::IValue(a * b),
                    |s| s.load_int_from_stack(),
                )?,
                Instruction::MulFloat => self.binary_op::<f32, _, _>(
                    |a, b| Value::FValue(a * b),
                    |s| s.load_float_from_stack(),
                )?,
                Instruction::Div => self.binary_op::<i32, _, _>(
                    |a, b| Value::IValue(a / b),
                    |s| s.load_int_from_stack(),
                )?,
                Instruction::DivFloat => self.binary_op::<f32, _, _>(
                    |a, b| Value::FValue(a / b),
                    |s| s.load_float_from_stack(),
                )?,
                Instruction::Call => {
                    // read fn name
                    let fun_name: String =
                        Self::read_str(&mut ptr, program).ok_or(ExecutionError::InvalidArgument)?;
                    let mut fun = self
                        .callables
                        .remove(fun_name.as_str())
                        .ok_or_else(|| ExecutionError::FunctionNotFound(fun_name))?;

                    let n_inputs = fun.num_params();
                    let mut inputs = Vec::with_capacity(n_inputs as usize);
                    for _ in 0..n_inputs {
                        inputs.push(
                            Self::read_pointer(&mut ptr, program)
                                .ok_or(ExecutionError::InvalidArgument)?,
                        )
                    }
                    let outptr = self.memory.len();
                    let res_size = fun.call(self, &inputs, outptr)?;
                    self.memory.resize_with(outptr + res_size, Default::default);
                    self.callables.insert(fun.name(), fun);
                }
            }
            if self.memory.len() > self.memory_limit {
                return Err(ExecutionError::OutOfMemory);
            }
        }

        Ok(())
    }

    fn load_int_from_stack(&self) -> Option<i32> {
        let val = self.stack.last()?;
        match val {
            Value::IValue(i) => Some(*i),
            Value::Pointer(p) => self.get_value(*p),
            _ => None,
        }
    }

    fn load_float_from_stack(&self) -> Option<f32> {
        let val = self.stack.last()?;
        match val {
            Value::FValue(i) => Some(*i),
            Value::Pointer(p) => self.get_value(*p),
            _ => None,
        }
    }

    fn binary_op<T: ByteEncodeProperties, F: Fn(T, T) -> Value, FLoader: Fn(&Self) -> Option<T>>(
        &mut self,
        op: F,
        loader: FLoader,
    ) -> Result<(), ExecutionError> {
        let b = loader(self).ok_or(ExecutionError::InvalidArgument)?;
        self.stack.pop().unwrap();
        let a = loader(self).ok_or(ExecutionError::InvalidArgument)?;
        self.stack.pop().unwrap();
        self.stack.push(op(a, b));
        Ok(())
    }

    fn read_pointer(ptr: &mut usize, program: &[u8]) -> Option<TPointer> {
        let len = TPointer::BYTELEN;
        let p = *ptr;
        *ptr += len;
        TPointer::decode(&program[p..p + len])
    }

    fn read_str(ptr: &mut usize, program: &[u8]) -> Option<String> {
        let p = *ptr;
        let s = std::str::from_utf8(&program[p..p + MAX_STR_LEN]).ok()?;
        *ptr += s.len();
        Some(s.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let value: TPointer = 12342;
        let encoded = value.encode();
        let decoded = TPointer::decode(&encoded).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn test_binary_operatons() {
        let mut vm = VM::new();

        vm.stack.push(Value::IValue(512));
        vm.stack.push(Value::IValue(42));

        vm.binary_op::<i32, _, _>(
            |a, b| Value::IValue((a + a / b) * b),
            |s| s.load_int_from_stack(),
        )
        .unwrap();

        let result = vm.stack.last().expect("Expected to read the result");
        match result {
            Value::IValue(result) => assert_eq!(*result, (512 + 512 / 42) * 42),
            _ => panic!("Invalid result type"),
        }
    }

    #[test]
    fn test_simple_program() {
        let mut vm = VM::new();

        let mut program = Vec::with_capacity(512);
        program.push(Instruction::LiteralInt as u8);
        program.append(&mut (512 as i32).encode());
        program.push(Instruction::LiteralInt as u8);
        program.append(&mut (42 as i32).encode());
        program.push(Instruction::SubInt as u8);
        program.push(Instruction::LiteralInt as u8);
        program.append(&mut (68 as i32).encode());
        program.push(Instruction::AddInt as u8);

        vm.run(&program).unwrap();
        assert_eq!(vm.stack.len(), 1);
        let value = vm.stack.last().unwrap();
        match value {
            Value::IValue(i) => assert_eq!(*i, 512 - 42 + 68),
            _ => panic!("Invalid value in the stack"),
        }
    }
}
