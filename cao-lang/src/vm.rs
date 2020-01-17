use crate::compiler::NodeId;
use crate::instruction::Instruction;
use crate::prelude::*;
use crate::scalar::Scalar;
use crate::{binary_compare, pop_stack};
use log::{debug, error};
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, Default)]
pub struct Object {
    /// index of the Object's data in the VM memory
    data_index: u32,
    /// size of the data in the VM memory
    data_size: i32,
}

impl Object {
    pub fn index(&self) -> u32 {
        self.data_index
    }

    pub fn size(&self) -> u32 {
        self.data_size as u32
    }
}

/// Cao-Lang bytecode interpreter.
/// Aux is an auxiliary data structure passed to custom functions.
#[derive(Debug)]
pub struct VM<Aux = ()> {
    auxiliary_data: Aux,
    memory: Vec<u8>,
    memory_limit: usize,
    callables: HashMap<String, FunctionObject<Aux>>,
    stack: Vec<Scalar>,
    registers: [Scalar; 16],
    objects: HashMap<TPointer, Object>,
}

impl<Aux> VM<Aux> {
    pub fn new(auxiliary_data: Aux) -> Self {
        Self {
            auxiliary_data,
            memory: Vec::with_capacity(512),
            callables: HashMap::with_capacity(128),
            memory_limit: 40000,
            stack: Vec::with_capacity(128),
            registers: Default::default(),
            objects: HashMap::with_capacity(128),
        }
    }

    pub fn registers(&self) -> &[Scalar] {
        &self.registers
    }

    pub fn stack(&self) -> &[Scalar] {
        &self.stack
    }

    pub fn get_aux(&self) -> &Aux {
        &self.auxiliary_data
    }

    pub fn get_aux_mut(&mut self) -> &mut Aux {
        &mut self.auxiliary_data
    }

    pub fn unwrap_aux(self) -> Aux {
        self.auxiliary_data
    }

    pub fn register_function<C: Callable<Aux> + 'static>(&mut self, name: &str, f: C) {
        self.callables
            .insert(name.to_owned(), FunctionObject::new(f));
    }

    pub fn register_function_obj(&mut self, name: &str, f: FunctionObject<Aux>) {
        self.callables.insert(name.to_owned(), f);
    }

    pub fn get_value<T: ByteEncodeProperties>(&self, ptr: TPointer) -> Option<T> {
        let size = T::BYTELEN as i32;
        let object = self.objects.get(&ptr)?;
        if object.data_size != size {
            debug!(
                "Attempting to reference an object with the wrong type ({}) at address {}",
                T::displayname(),
                ptr
            );
            return None;
        }
        let head = object.data_index as usize;
        let tail = (head + size as usize).min(self.memory.len());
        T::decode(&self.memory[head..tail])
    }

    // TODO: check if maximum size was exceeded
    pub fn set_value<T: ByteEncodeProperties>(&mut self, val: T) -> Result<Object, ExecutionError> {
        let result = self.memory.len();
        let bytes = val.encode();
        let object = Object {
            data_index: result as u32,
            data_size: T::BYTELEN as i32,
        };

        self.memory.extend(bytes.iter());

        self.objects.insert(result as TPointer, object);
        debug!(
            "Set value {:?} {:?} {}",
            object,
            T::BYTELEN,
            T::displayname()
        );

        Ok(object)
    }

    pub fn run(&mut self, program: &CompiledProgram) -> Result<i32, ExecutionError> {
        debug!("Running program");
        let mut ptr = 0;
        let mut max_iter = 1000;
        while ptr < program.bytecode.len() {
            max_iter -= 1;
            if max_iter <= 0 {
                return Err(ExecutionError::Timeout);
            }
            let instr = Instruction::try_from(program.bytecode[ptr]).map_err(|_| {
                error!(
                    "Byte at {}: {:?} was not a valid instruction",
                    ptr, program.bytecode[ptr]
                );
                ExecutionError::InvalidInstruction
            })?;
            debug!(
                "Instruction: {:?}({:?}) Pointer: {:?}",
                instr, program.bytecode[ptr], ptr
            );
            ptr += 1;
            match instr {
                Instruction::Start => {}
                Instruction::WriteReg => {
                    let value = self.stack.pop().ok_or_else(|| {
                        debug!("Value not found");
                        ExecutionError::InvalidArgument
                    })?;
                    let len = i32::BYTELEN;
                    let index = i32::decode(&program.bytecode[ptr..ptr + len])
                        .filter(|x| *x < self.registers.len() as i32)
                        .ok_or(ExecutionError::InvalidArgument)?;
                    self.registers[index as usize] = value;
                    ptr += len;
                }
                Instruction::ReadReg => {
                    let len = i32::BYTELEN;
                    let index = i32::decode(&program.bytecode[ptr..ptr + len])
                        .filter(|x| *x < self.registers.len() as i32)
                        .ok_or(ExecutionError::InvalidArgument)?;
                    let value = self.registers[index as usize].clone();
                    self.stack.push(value);
                    ptr += len;
                }
                Instruction::Jump => {
                    let label = self
                        .stack
                        .pop()
                        .ok_or_else(|| ExecutionError::InvalidArgument)
                        .and_then(|x| {
                            NodeId::try_from(x).map_err(|_| ExecutionError::InvalidArgument)
                        })?;
                    ptr = program
                        .labels
                        .get(&label)
                        .ok_or(ExecutionError::InvalidLabel)?[0];
                }
                Instruction::Exit => {
                    debug!("Exit called");
                    let code = self.stack.last();
                    if let Some(Scalar::Integer(code)) = code {
                        let code = *code;
                        self.stack.pop();
                        if code != 0 {
                            debug!("Exit code {:?}", code);
                            return Err(ExecutionError::ExitCode(code));
                        } else {
                            return Ok(code);
                        }
                    }
                    return Ok(0);
                }
                Instruction::JumpIfTrue => {
                    if self.stack.len() < 1 {
                        error!(
                            "JumpIfTrue called with missing arguments, stack: {:?}",
                            self.stack
                        );
                        return Err(ExecutionError::InvalidArgument);
                    }
                    let cond = self.stack.pop().unwrap();
                    let len = i32::BYTELEN;
                    let label = i32::decode(&program.bytecode[ptr..ptr + len])
                        .ok_or_else(|| ExecutionError::InvalidLabel)?;
                    if cond.as_bool() {
                        ptr = program
                            .labels
                            .get(&label)
                            .ok_or(ExecutionError::InvalidLabel)?[0];
                    } else {
                        ptr += len;
                    }
                }
                Instruction::CopyLast => {
                    if !self.stack.is_empty() {
                        self.stack.push(self.stack.last().cloned().unwrap());
                    }
                }
                Instruction::Pass => {}
                Instruction::ScalarLabel => {
                    let len = NodeId::BYTELEN;
                    self.stack.push(Scalar::Integer(
                        NodeId::decode(&program.bytecode[ptr..ptr + len])
                            .ok_or(ExecutionError::InvalidArgument)?,
                    ));
                    ptr += len;
                }
                Instruction::ScalarInt => {
                    let len = i32::BYTELEN;
                    self.stack.push(Scalar::Integer(
                        i32::decode(&program.bytecode[ptr..ptr + len])
                            .ok_or(ExecutionError::InvalidArgument)?,
                    ));
                    ptr += len;
                }
                Instruction::ScalarFloat => {
                    let len = f32::BYTELEN;
                    self.stack.push(Scalar::Floating(
                        f32::decode(&program.bytecode[ptr..ptr + len])
                            .ok_or(ExecutionError::InvalidArgument)?,
                    ));
                    ptr += len;
                }
                Instruction::ScalarArray => {
                    let len = self
                        .load_ptr_from_stack()
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
                    self.stack.push(Scalar::Pointer(ptr as i32));
                }
                Instruction::Add => self.binary_op(|a, b| a + b)?,
                Instruction::Sub => self.binary_op(|a, b| a - b)?,
                Instruction::Mul => self.binary_op(|a, b| a * b)?,
                Instruction::Div => self.binary_op(|a, b| a / b)?,
                Instruction::Equals => binary_compare!(self, ==, false),
                Instruction::NotEquals => binary_compare!(self, !=, true),
                Instruction::Less => binary_compare!(self, <, false),
                Instruction::LessOrEq => binary_compare!(self, <=, false),
                Instruction::StringLiteral => {
                    let literal = Self::read_str(&mut ptr, &program.bytecode)
                        .ok_or(ExecutionError::InvalidArgument)?;
                    let obj = self.set_value(literal)?;
                    self.stack.push(Scalar::Pointer(obj.index() as i32));
                }
                Instruction::Call => {
                    let fun_name =
                        Self::read_str(&mut ptr, &program.bytecode).ok_or_else(|| {
                            error!("Could not read function name");
                            ExecutionError::InvalidArgument
                        })?;
                    let mut fun = self.callables.remove(fun_name.as_str()).ok_or_else(|| {
                        ExecutionError::FunctionNotFound(fun_name.as_str().to_owned())
                    })?;

                    let n_inputs = fun.num_params();
                    let mut inputs = Vec::with_capacity(n_inputs as usize);
                    for _ in 0..n_inputs {
                        inputs.push(self.stack.pop().ok_or_else(|| {
                            error!("Missing argument to function call {:?}", fun_name);
                            ExecutionError::MissingArgument
                        })?)
                    }
                    let outptr = self.memory.len() as i32;
                    debug!(
                        "Calling function {} with inputs: {:?} output: {:?}",
                        fun_name, inputs, outptr
                    );
                    let res = fun.call(self, &inputs).map_err(|e| {
                        error!("Calling function {:?} failed with {:?}", fun_name, e);
                        e
                    })?;
                    debug!("Function call returned value: {:?}", res);

                    if res.data_size > 0 {
                        self.stack.push(Scalar::Pointer(outptr));
                    }

                    self.callables.insert(fun_name, fun);
                }
            }
            if self.memory.len() > self.memory_limit {
                return Err(ExecutionError::OutOfMemory);
            }
            debug!("Stack {:?}", self.stack);
            debug!("Top of stack: {:?}", self.stack.last());
        }

        Err(ExecutionError::UnexpectedEndOfInput)
    }

    fn load_ptr_from_stack(&self) -> Option<i32> {
        let val = self.stack.last()?;
        match val {
            Scalar::Pointer(i) => Some(*i),
            _ => None,
        }
    }

    fn binary_op<F>(&mut self, op: F) -> Result<(), ExecutionError>
    where
        F: Fn(Scalar, Scalar) -> Scalar,
    {
        let b = pop_stack!(self);
        let a = pop_stack!(self);
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
    use crate::traits::FunctionWrapper;

    #[test]
    fn test_encode() {
        let value: TPointer = 12342;
        let encoded = value.encode();
        let decoded = TPointer::decode(&encoded).unwrap();

        assert_eq!(value, decoded);
    }

    #[test]
    fn test_binary_operatons() {
        let mut vm = VM::new(());

        vm.stack.push(Scalar::Integer(512));
        vm.stack.push(Scalar::Integer(42));

        vm.binary_op(|a, b| (a + a / b) * b).unwrap();

        let result = vm.stack.last().expect("Expected to read the result");
        match result {
            Scalar::Integer(result) => assert_eq!(*result, (512 + 512 / 42) * 42),
            _ => panic!("Invalid result type"),
        }
    }

    #[test]
    fn test_simple_program() {
        let mut bytecode = Vec::with_capacity(512);
        bytecode.push(Instruction::ScalarInt as u8);
        bytecode.append(&mut 512i32.encode());
        bytecode.push(Instruction::ScalarInt as u8);
        bytecode.append(&mut 42i32.encode());
        bytecode.push(Instruction::Sub as u8);
        bytecode.push(Instruction::ScalarInt as u8);
        bytecode.append(&mut 68i32.encode());
        bytecode.push(Instruction::Add as u8);
        bytecode.push(Instruction::ScalarInt as u8);
        bytecode.append(&mut 0i32.encode());
        bytecode.push(Instruction::Exit as u8);
        let mut program = CompiledProgram::default();
        program.bytecode = bytecode;

        let mut vm = VM::new(());
        vm.run(&program).unwrap();
        assert_eq!(vm.stack.len(), 1);
        let value = vm.stack.last().unwrap();
        match value {
            Scalar::Integer(i) => assert_eq!(*i, 512 - 42 + 68),
            _ => panic!("Invalid value in the stack"),
        }
    }

    #[test]
    fn test_function_call() {
        let mut bytecode = Vec::with_capacity(512);
        bytecode.push(Instruction::ScalarFloat as u8);
        bytecode.append(&mut 42.0f32.encode());
        bytecode.push(Instruction::ScalarInt as u8);
        bytecode.append(&mut 512i32.encode());
        bytecode.push(Instruction::Call as u8);
        bytecode.append(&mut "foo".to_owned().encode());
        bytecode.push(Instruction::Exit as u8);

        let mut program = CompiledProgram::default();
        program.bytecode = bytecode;

        let mut vm = VM::new(());

        fn foo(vm: &mut VM<()>, (a, b): (i32, f32)) -> ExecutionResult {
            let res = a as f32 * b % 13.;
            let res = res as i32;

            vm.set_value(res)
        };

        vm.register_function("foo", FunctionWrapper::new(foo));
        vm.register_function(
            "bar",
            FunctionWrapper::new(|_vm: &mut VM, _a: i32| {
                Err::<Object, _>(ExecutionError::Unimplemented)
            }),
        );

        vm.run(&program).unwrap();

        let ptr = 0;
        let res = vm.get_value::<i32>(ptr).unwrap();

        assert_eq!(res, ((512. as f32) * (42. as f32) % 13.) as i32);
    }
}
