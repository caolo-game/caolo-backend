use crate::{scalar::Scalar, vm::VM, ExecutionError, TPointer};
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
        assert!(self.len() < Self::BYTELEN);
        let mut rr = (self.len() as i32).encode();
        rr.extend(self.chars().map(|c| c as u8));
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
/// Note that using this with pointers, arrays, strings etc. will not work as one might expect!
pub trait AutoByteEncodeProperties {}

impl AutoByteEncodeProperties for i8 {}
impl AutoByteEncodeProperties for i16 {}
impl AutoByteEncodeProperties for i32 {}
impl AutoByteEncodeProperties for i64 {}
impl AutoByteEncodeProperties for u8 {}
impl AutoByteEncodeProperties for u16 {}
impl AutoByteEncodeProperties for u32 {}
impl AutoByteEncodeProperties for u64 {}
impl AutoByteEncodeProperties for f32 {}
impl AutoByteEncodeProperties for f64 {}
impl AutoByteEncodeProperties for TPointer {}

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

pub struct FunctionObject<Aux> {
    fun: Box<dyn Callable<Aux>>,
}

impl<Aux> Callable<Aux> for FunctionObject<Aux> {
    fn call(
        &mut self,
        vm: &mut VM<Aux>,
        params: &[Scalar],
        output: TPointer,
    ) -> Result<usize, ExecutionError> {
        self.fun.call(vm, params, output)
    }

    fn num_params(&self) -> u8 {
        self.fun.num_params()
    }
}

impl<Aux> FunctionObject<Aux> {
    pub fn new<C: Callable<Aux> + 'static>(f: C) -> Self {
        Self { fun: Box::new(f) }
    }
}

impl<Aux> std::fmt::Debug for FunctionObject<Aux> {
    fn fmt(&self, writer: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(writer, "FunctionObject")
    }
}

pub trait Callable<Aux> {
    /// Take in the VM, parameters and output pointer in parameters and return the length of the
    /// result
    fn call(
        &mut self,
        vm: &mut VM<Aux>,
        params: &[Scalar],
        output: TPointer,
    ) -> Result<usize, ExecutionError>;

    fn num_params(&self) -> u8;
}

pub struct FunctionWrapper<Aux, F, Args>
where
    F: Fn(&mut VM<Aux>, Args, TPointer) -> Result<usize, ExecutionError>,
{
    pub f: F,
    _args: PhantomData<(Args, Aux)>,
}

impl<Aux, F, Args> FunctionWrapper<Aux, F, Args>
where
    F: Fn(&mut VM<Aux>, Args, TPointer) -> Result<usize, ExecutionError>,
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            _args: Default::default(),
        }
    }
}

impl<Aux, F> Callable<Aux> for FunctionWrapper<Aux, F, ()>
where
    F: Fn(&mut VM<Aux>, (), TPointer) -> Result<usize, ExecutionError>,
{
    fn call(
        &mut self,
        vm: &mut VM<Aux>,
        _params: &[Scalar],
        output: TPointer,
    ) -> Result<usize, ExecutionError> {
        (self.f)(vm, (), output)
    }

    fn num_params(&self) -> u8 {
        0
    }
}

impl<Aux, F, T> Callable<Aux> for FunctionWrapper<Aux, F, T>
where
    F: Fn(&mut VM<Aux>, T, TPointer) -> Result<usize, ExecutionError>,
    T: TryFrom<Scalar>,
{
    fn call(
        &mut self,
        vm: &mut VM<Aux>,
        params: &[Scalar],
        output: TPointer,
    ) -> Result<usize, ExecutionError> {
        let val = T::try_from(params[0]).map_err(|_| ExecutionError::InvalidArgument)?;
        (self.f)(vm, val, output)
    }

    fn num_params(&self) -> u8 {
        1
    }
}

impl<Aux, F, T1, T2> Callable<Aux> for FunctionWrapper<Aux, F, (T1, T2)>
where
    F: Fn(&mut VM<Aux>, (T1, T2), TPointer) -> Result<usize, ExecutionError>,
    T1: TryFrom<Scalar>,
    T2: TryFrom<Scalar>,
{
    fn call(
        &mut self,
        vm: &mut VM<Aux>,
        params: &[Scalar],
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

impl<Aux, F, T1, T2, T3> Callable<Aux> for FunctionWrapper<Aux, F, (T1, T2, T3)>
where
    F: Fn(&mut VM<Aux>, (T1, T2, T3), TPointer) -> Result<usize, ExecutionError>,
    T1: TryFrom<Scalar>,
    T2: TryFrom<Scalar>,
    T3: TryFrom<Scalar>,
{
    fn call(
        &mut self,
        vm: &mut VM<Aux>,
        params: &[Scalar],
        output: TPointer,
    ) -> Result<usize, ExecutionError> {
        let a = T1::try_from(params[0]).map_err(|_| ExecutionError::InvalidArgument)?;
        let b = T2::try_from(params[1]).map_err(|_| ExecutionError::InvalidArgument)?;
        let c = T3::try_from(params[1]).map_err(|_| ExecutionError::InvalidArgument)?;
        (self.f)(vm, (a, b, c), output)
    }

    fn num_params(&self) -> u8 {
        3
    }
}
