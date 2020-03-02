use crate::{
    scalar::Scalar,
    vm::{Object, VM},
    ExecutionError,
};
use std::any::type_name;
use std::convert::TryFrom;
use std::fmt::Write;
use std::marker::PhantomData;
use std::mem;

pub const MAX_STR_LEN: usize = 128;

pub trait ObjectProperties: std::fmt::Debug {
    fn write_debug(&self, output: &mut String) {
        write!(output, "{:?}", self).unwrap();
    }
}

pub type ExecutionResult = Result<Object, ExecutionError>;

pub trait ByteEncodeProperties: Sized + ObjectProperties {
    const BYTELEN: usize = mem::size_of::<Self>();

    fn displayname() -> &'static str {
        type_name::<Self>()
    }
    fn encode(self) -> Vec<u8>;
    fn decode(bytes: &[u8]) -> Option<Self>;
}

impl ByteEncodeProperties for String {
    const BYTELEN: usize = MAX_STR_LEN;

    fn displayname() -> &'static str {
        "Text"
    }

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

impl ByteEncodeProperties for () {
    const BYTELEN: usize = 0;
    fn displayname() -> &'static str {
        "Void"
    }

    fn encode(self) -> Vec<u8> {
        vec![]
    }

    fn decode(_bytes: &[u8]) -> Option<Self> {
        None
    }
}

/// Opts in for the default implementation of ByteEncodeProperties
/// Note that using this with pointers, arrays, strings etc. will not work as one might expect!
pub trait AutoByteEncodeProperties {
    fn displayname() -> &'static str {
        type_name::<Self>()
    }
}

impl AutoByteEncodeProperties for i8 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for i16 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for i32 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for i64 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for u8 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for u16 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for u32 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for u64 {
    fn displayname() -> &'static str {
        "Integer"
    }
}
impl AutoByteEncodeProperties for f32 {
    fn displayname() -> &'static str {
        "Floating point"
    }
}
impl AutoByteEncodeProperties for f64 {
    fn displayname() -> &'static str {
        "Floating point"
    }
}

impl<T1: AutoByteEncodeProperties> AutoByteEncodeProperties for (T1,) {}

impl<T1: AutoByteEncodeProperties, T2: AutoByteEncodeProperties> AutoByteEncodeProperties
    for (T1, T2)
{
}

impl<T1: AutoByteEncodeProperties, T2: AutoByteEncodeProperties, T3: AutoByteEncodeProperties>
    AutoByteEncodeProperties for (T1, T2, T3)
{
}

impl<
        T1: AutoByteEncodeProperties,
        T2: AutoByteEncodeProperties,
        T3: AutoByteEncodeProperties,
        T4: AutoByteEncodeProperties,
    > AutoByteEncodeProperties for (T1, T2, T3, T4)
{
}

impl<T: std::fmt::Debug> ObjectProperties for T {
    fn write_debug(&self, output: &mut String) {
        write!(output, "{:?}", self).unwrap();
    }
}

impl<T: Sized + Clone + Copy + AutoByteEncodeProperties + std::fmt::Debug> ByteEncodeProperties
    for T
{
    fn encode(self) -> Vec<u8> {
        let size: usize = Self::BYTELEN;

        let mut result = vec![0; size];
        unsafe {
            let dayum = mem::transmute::<*const Self, *const u8>(&self as *const Self);
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
    fn call(&mut self, vm: &mut VM<Aux>, params: &[Scalar]) -> ExecutionResult {
        self.fun.call(vm, params)
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
    fn call(&mut self, vm: &mut VM<Aux>, params: &[Scalar]) -> ExecutionResult;

    fn num_params(&self) -> u8;
}

pub struct FunctionWrapper<Aux, F, Args>
where
    F: Fn(&mut VM<Aux>, Args) -> ExecutionResult,
{
    pub f: F,
    _args: PhantomData<(Args, Aux)>,
}

impl<Aux, F, Args> FunctionWrapper<Aux, F, Args>
where
    F: Fn(&mut VM<Aux>, Args) -> ExecutionResult,
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
    F: Fn(&mut VM<Aux>, ()) -> ExecutionResult,
{
    fn call(&mut self, vm: &mut VM<Aux>, _params: &[Scalar]) -> ExecutionResult {
        (self.f)(vm, ())
    }

    fn num_params(&self) -> u8 {
        0
    }
}

impl<Aux, F, T> Callable<Aux> for FunctionWrapper<Aux, F, T>
where
    F: Fn(&mut VM<Aux>, T) -> ExecutionResult,
    T: TryFrom<Scalar>,
{
    fn call(&mut self, vm: &mut VM<Aux>, params: &[Scalar]) -> ExecutionResult {
        let val = T::try_from(params[0]).map_err(convert_error(0))?;
        (self.f)(vm, val)
    }

    fn num_params(&self) -> u8 {
        1
    }
}

impl<Aux, F, T1, T2> Callable<Aux> for FunctionWrapper<Aux, F, (T1, T2)>
where
    F: Fn(&mut VM<Aux>, (T1, T2)) -> ExecutionResult,
    T1: TryFrom<Scalar>,
    T2: TryFrom<Scalar>,
{
    fn call(&mut self, vm: &mut VM<Aux>, params: &[Scalar]) -> ExecutionResult {
        let a = T1::try_from(params[0]).map_err(convert_error(0))?;
        let b = T2::try_from(params[1]).map_err(convert_error(1))?;
        (self.f)(vm, (a, b))
    }

    fn num_params(&self) -> u8 {
        2
    }
}

impl<Aux, F, T1, T2, T3> Callable<Aux> for FunctionWrapper<Aux, F, (T1, T2, T3)>
where
    F: Fn(&mut VM<Aux>, (T1, T2, T3)) -> ExecutionResult,
    T1: TryFrom<Scalar>,
    T2: TryFrom<Scalar>,
    T3: TryFrom<Scalar>,
{
    fn call(&mut self, vm: &mut VM<Aux>, params: &[Scalar]) -> ExecutionResult {
        let a = T1::try_from(params[0]).map_err(convert_error(0))?;
        let b = T2::try_from(params[1]).map_err(convert_error(1))?;
        let c = T3::try_from(params[1]).map_err(convert_error(2))?;
        (self.f)(vm, (a, b, c))
    }

    fn num_params(&self) -> u8 {
        3
    }
}

fn convert_error<'a, T: 'a>(i: i32) -> impl Fn(T) -> ExecutionError + 'a {
    return move |_| {
        log::debug!("Failed to convert arugment #{}", i);
        ExecutionError::InvalidArgument
    };
}
