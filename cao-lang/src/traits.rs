use crate::{ExecutionError, TPointer};
use std::mem;

pub const MAX_STR_LEN: usize = 512;

pub trait ByteEncodeProperties: Sized {
    const BYTELEN: usize = mem::size_of::<Self>();

    fn encode(self) -> Vec<u8>;
    fn decode(bytes: &[u8]) -> Option<Self>;
}

impl ByteEncodeProperties for String {
    const BYTELEN: usize = MAX_STR_LEN;

    fn encode(self) -> Vec<u8> {
        self.chars().map(|c| c as u8).collect()
    }
    fn decode(bytes: &[u8]) -> Option<Self> {
        let string = bytes.iter().map(|c| *c as char).collect();
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

pub trait Callable {
    /// Take in the VM, parameters and output pointer in parameters and return the length of the
    /// result
    fn call(
        &mut self,
        vm: &mut crate::VM,
        params: &[TPointer],
        output: TPointer,
    ) -> Result<usize, ExecutionError>;

    fn num_params(&self) -> u8;
    fn name(&self) -> &'static str;
}

impl std::fmt::Debug for Box<dyn Callable> {
    fn fmt(&self, writer: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(writer, "{}", self.name())
    }
}
