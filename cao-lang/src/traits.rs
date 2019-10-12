use crate::TPointer;
use std::mem;

pub trait ByteEncodeProperties: Sized {
    const BYTELEN: usize = mem::size_of::<Self>();

    fn encode(self) -> Vec<u8>;
    fn decode(bytes: &[u8]) -> Option<Self>;
}

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
impl AutoByteEncodeProperties for caolo_api::EntityId {}

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
