use crate::{ExecutionError, TPointer, Value};
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::mem;

pub const MAX_STR_LEN: usize = 128;

pub trait ByteEncodeProperties: Sized {
    const BYTELEN: usize = mem::size_of::<Self>();

    fn encode(self) -> Vec<u8>;
    fn decode(bytes: &[u8]) -> Option<Self>;
}

impl ByteEncodeProperties for String {
    const BYTELEN: usize = MAX_STR_LEN;

    fn encode(self) -> Vec<u8> {
        let mut res: Vec<u8> = self.chars().map(|c| c as u8).collect();
        assert!(res.len() < MAX_STR_LEN);
        let len = res.len() as i32;
        let mut rr = len.encode();
        rr.append(&mut res);
        rr
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        let len = i32::decode(bytes)?;
        let string = bytes
            .iter()
            .skip(i32::BYTELEN)
            .take(len as usize)
            .map(|c| *c as char)
            .collect();
        Some(string)
    }
}

/// Opts in for the default implementation of ByteEncodeProperties
/// Note that using this with pointers, arrays etc. will not work as one might expect!
pub trait AutoByteEncodeProperties {}

impl AutoByteEncodeProperties for i8 {}
impl AutoByteEncodeProperties for i16 {}
impl AutoByteEncodeProperties for i32 {}
impl AutoByteEncodeProperties for u8 {}
impl AutoByteEncodeProperties for u16 {}
impl AutoByteEncodeProperties for u32 {}
impl AutoByteEncodeProperties for f32 {}
impl AutoByteEncodeProperties for TPointer {}
impl AutoByteEncodeProperties for caolo_api::point::Point {}
impl AutoByteEncodeProperties for caolo_api::bots::Bot {}
impl AutoByteEncodeProperties for caolo_api::EntityId {}
impl AutoByteEncodeProperties for caolo_api::OperationResult {}

impl<T: Sized + Clone + Copy + AutoByteEncodeProperties> ByteEncodeProperties for T {
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

pub struct FunctionObject {
    fun: Box<dyn Callable>,
}

impl Callable for FunctionObject {
    fn call(
        &mut self,
        vm: &mut crate::VM,
        params: &[Value],
        output: TPointer,
    ) -> Result<usize, ExecutionError> {
        self.fun.call(vm, params, output)
    }

    fn num_params(&self) -> u8 {
        self.fun.num_params()
    }
}

impl FunctionObject {
    pub fn new<C: Callable + 'static>(f: C) -> Self {
        Self { fun: Box::new(f) }
    }
}

impl std::fmt::Debug for FunctionObject {
    fn fmt(&self, writer: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(writer, "FunctionObject")
    }
}

pub trait Callable {
    /// Take in the VM, parameters and output pointer in parameters and return the length of the
    /// result
    fn call(
        &mut self,
        vm: &mut crate::VM,
        params: &[Value],
        output: TPointer,
    ) -> Result<usize, ExecutionError>;

    fn num_params(&self) -> u8;
}

pub struct FunctionWrapper<F, Args>
where
    F: Fn(&mut crate::VM, Args, TPointer) -> Result<usize, ExecutionError>,
{
    pub f: F,
    _args: PhantomData<Args>,
}

impl<F, Args> FunctionWrapper<F, Args>
where
    F: Fn(&mut crate::VM, Args, TPointer) -> Result<usize, ExecutionError>,
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            _args: Default::default(),
        }
    }
}

impl<F> Callable for FunctionWrapper<F, ()>
where
    F: Fn(&mut crate::VM, (), TPointer) -> Result<usize, ExecutionError>,
{
    fn call(
        &mut self,
        vm: &mut crate::VM,
        _params: &[Value],
        output: TPointer,
    ) -> Result<usize, ExecutionError> {
        (self.f)(vm, (), output)
    }

    fn num_params(&self) -> u8 {
        0
    }
}

impl<F, T> Callable for FunctionWrapper<F, T>
where
    F: Fn(&mut crate::VM, T, TPointer) -> Result<usize, ExecutionError>,
    T: TryFrom<Value>,
{
    fn call(
        &mut self,
        vm: &mut crate::VM,
        params: &[Value],
        output: TPointer,
    ) -> Result<usize, ExecutionError> {
        let val = T::try_from(params[0]).map_err(|_| ExecutionError::InvalidArgument)?;
        (self.f)(vm, val, output)
    }

    fn num_params(&self) -> u8 {
        1
    }
}

impl<F, T1, T2> Callable for FunctionWrapper<F, (T1, T2)>
where
    F: Fn(&mut crate::VM, (T1, T2), TPointer) -> Result<usize, ExecutionError>,
    T1: TryFrom<Value>,
    T2: TryFrom<Value>,
{
    fn call(
        &mut self,
        vm: &mut crate::VM,
        params: &[Value],
        output: TPointer,
    ) -> Result<usize, ExecutionError> {
        let a = T1::try_from(params[0]).map_err(|_| ExecutionError::InvalidArgument)?;
        let b = T2::try_from(params[1]).map_err(|_| ExecutionError::InvalidArgument)?;
        (self.f)(vm, (a, b), output)
    }

    fn num_params(&self) -> u8 {
        2
    }
}
