use crate::traits::AutoByteEncodeProperties;
use crate::TPointer;
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Pointer(TPointer),
    IValue(i32),
    FValue(f32),
}

impl AutoByteEncodeProperties for Value {}

impl TryFrom<Value> for i32 {
    type Error = Value;

    fn try_from(v: Value) -> Result<Self, Value> {
        match v {
            Value::IValue(i) => Ok(i),
            _ => Err(v),
        }
    }
}
impl TryFrom<Value> for f32 {
    type Error = Value;

    fn try_from(v: Value) -> Result<Self, Value> {
        match v {
            Value::FValue(i) => Ok(i),
            _ => Err(v),
        }
    }
}
impl TryFrom<Value> for TPointer {
    type Error = Value;

    fn try_from(v: Value) -> Result<Self, Value> {
        match v {
            Value::Pointer(i) => Ok(i),
            _ => Err(v),
        }
    }
}
