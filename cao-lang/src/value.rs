use crate::traits::AutoByteEncodeProperties;
use crate::TPointer;
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Value {
    Pointer(TPointer),
    IValue(i32),
    FValue(f32),
}

impl Value {
    pub fn as_bool(self) -> bool {
        use Value::*;
        match self {
            Pointer(i) => i != 0,
            IValue(i) => i != 0,
            FValue(i) => i != 0.0,
        }
    }
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
