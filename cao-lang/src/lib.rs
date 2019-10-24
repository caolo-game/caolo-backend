pub mod compiler;
pub mod prelude;
pub mod traits;
pub mod value;

use prelude::*;

use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;

pub type TPointer = usize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionError {
    InvalidInstruction,
    InvalidArgument,
    FunctionNotFound(String),
    Unimplemented,
    OutOfMemory,
}

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
    /// Moves the bot with entity id to the point and writes an OperationResult to the first
    /// pointer
    Call = 9,
    /// Push an int onto the stack
    LiteralInt = 10,
    /// Push a float onto the stack
    LiteralFloat = 11,
    /// Push a ptr onto the stack
    LiteralPtr = 12,
    /// Pop the next N (positive integer) number of items from the stack and write them to memory
    /// Push the pointer to the beginning of the array onto the stack
    LiteralArray = 13,
    /// Empty instruction that has no effects
    Pass = 14,
    /// Clones the last element on the stack
    /// Does nothing if no elements are on the stack
    CopyLast = 15,
    /// Branching (If-Else) instruction
    /// If the value at the top of the stack is truthy jumps to the
    /// first index else jumps to the second index
    Branch = 16,
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
            14 => Ok(Pass),
            15 => Ok(CopyLast),
            16 => Ok(Branch),
            _ => Err(format!("Unrecognized instruction [{}]", c)),
        }
    }
}

/// Cao-Lang bytecode interpreter
#[derive(Debug)]
pub struct VM {
    memory: Vec<u8>,
    memory_limit: usize,
    callables: HashMap<String, FunctionObject>,
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

    pub fn register_function<C: Callable + 'static>(&mut self, name: &str, f: C) {
        self.callables
            .insert(name.to_owned(), FunctionObject::new(f));
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
        if ptr + bytes.len() > self.memory.len() {
            self.memory.resize(ptr + bytes.len(), 0);
        }
        self.memory.as_mut_slice()[ptr..ptr + bytes.len()].copy_from_slice(&bytes[..]);
    }

    pub fn run(&mut self, program: &[u8]) -> Result<(), ExecutionError> {
        let mut ptr = 0;
        while ptr < program.len() {
            let instr = Instruction::try_from(program[ptr])
                .map_err(|_| ExecutionError::InvalidInstruction)?;
            ptr += 1;
            match instr {
                Instruction::Branch => {
                    if self.stack.is_empty() || self.stack.len() < 3 {
                        return Err(ExecutionError::InvalidArgument);
                    }
                    let [iffalse, iftrue, cond] = [
                        self.stack.pop().unwrap(),
                        self.stack.pop().unwrap(),
                        self.stack.pop().unwrap(),
                    ];
                    let iftrue: i32 =
                        i32::try_from(iftrue).map_err(|_| ExecutionError::InvalidArgument)?;
                    let iffalse: i32 =
                        i32::try_from(iffalse).map_err(|_| ExecutionError::InvalidArgument)?;
                    if cond.as_bool() {
                        ptr = iftrue as usize;
                    } else {
                        ptr = iffalse as usize;
                    }
                }
                Instruction::CopyLast => {
                    if !self.stack.is_empty() {
                        self.stack.push(self.stack.last().cloned().unwrap());
                    }
                }
                Instruction::Pass => {}
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
                    let fun_name =
                        Self::read_str(&mut ptr, program).ok_or(ExecutionError::InvalidArgument)?;
                    let mut fun = self.callables.remove(fun_name.as_str()).ok_or_else(|| {
                        ExecutionError::FunctionNotFound(fun_name.as_str().to_owned())
                    })?;

                    let n_inputs = fun.num_params();
                    let mut inputs = Vec::with_capacity(n_inputs as usize);
                    for _ in 0..n_inputs {
                        inputs.push(self.stack.pop().ok_or(ExecutionError::InvalidArgument)?)
                    }
                    let outptr = self.memory.len();
                    let res_size = fun.call(self, &inputs, outptr)?;
                    self.memory.resize_with(outptr + res_size, Default::default);
                    self.stack.push(Value::Pointer(outptr));

                    self.callables.insert(fun_name, fun);
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

    fn read_str(ptr: &mut usize, program: &[u8]) -> Option<String> {
        let p = *ptr;
        let limit = program.len().min(p + MAX_STR_LEN);
        let s = String::decode(&program[p..limit])?;
        *ptr += s.len() + i32::BYTELEN;
        Some(s.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use traits::FunctionWrapper;

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
        let mut program = Vec::with_capacity(512);
        program.push(Instruction::LiteralInt as u8);
        program.append(&mut (512 as i32).encode());
        program.push(Instruction::LiteralInt as u8);
        program.append(&mut (42 as i32).encode());
        program.push(Instruction::SubInt as u8);
        program.push(Instruction::LiteralInt as u8);
        program.append(&mut (68 as i32).encode());
        program.push(Instruction::AddInt as u8);

        let mut vm = VM::new();
        vm.run(&program).unwrap();
        assert_eq!(vm.stack.len(), 1);
        let value = vm.stack.last().unwrap();
        match value {
            Value::IValue(i) => assert_eq!(*i, 512 - 42 + 68),
            _ => panic!("Invalid value in the stack"),
        }
    }

    #[test]
    fn test_function_call() {
        let mut program = Vec::with_capacity(512);
        program.push(Instruction::LiteralFloat as u8);
        program.append(&mut (42.0 as f32).encode());
        program.push(Instruction::LiteralInt as u8);
        program.append(&mut (512 as i32).encode());
        program.push(Instruction::Call as u8);
        program.append(&mut "foo".to_owned().encode());

        let mut vm = VM::new();

        fn foo(vm: &mut VM, (a, b): (i32, f32), out: TPointer) -> Result<usize, ExecutionError> {
            let res = a as f32 * b % 13.;
            let res = res as i32;

            vm.set_value_at(out, res);
            Ok(i32::BYTELEN)
        };

        vm.register_function("foo", FunctionWrapper::new(foo));
        vm.register_function(
            "bar",
            FunctionWrapper::new(|_vm: &mut VM, _a: i32, _out: TPointer| {
                Err::<usize, _>(ExecutionError::Unimplemented)
            }),
        );

        vm.run(&program).unwrap();

        let ptr = 0;
        let res = vm.get_value::<i32>(ptr).unwrap();

        assert_eq!(res, ((512. as f32) * (42. as f32) % 13.) as i32);
    }
}
