use crate::compiler::NodeId;
use crate::traits::AutoByteEncodeProperties;
use crate::TPointer;
use serde_derive::{Deserialize, Serialize};
use std::convert::{From, TryFrom};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Scalar {
    Pointer(TPointer),
    Integer(i32),
    Floating(f32),
}

impl Scalar {
    pub fn as_bool(self) -> bool {
        use Scalar::*;
        match self {
            Pointer(i) => i != 0,
            Integer(i) => i != 0,
            Floating(i) => i != 0.0,
        }
    }
}

impl AutoByteEncodeProperties for Scalar {}

impl From<Scalar> for bool {
    fn from(s: Scalar) -> Self {
        s.as_bool()
    }
}

impl TryFrom<Scalar> for i32 {
    type Error = Scalar;

    fn try_from(v: Scalar) -> Result<Self, Scalar> {
        match v {
            Scalar::Integer(i) => Ok(i),
            _ => Err(v),
        }
    }
}

impl TryFrom<Scalar> for f32 {
    type Error = Scalar;

    fn try_from(v: Scalar) -> Result<Self, Scalar> {
        match v {
            Scalar::Floating(i) => Ok(i),
            _ => Err(v),
        }
    }
}

impl TryFrom<Scalar> for TPointer {
    type Error = Scalar;

    fn try_from(v: Scalar) -> Result<Self, Scalar> {
        match v {
            Scalar::Pointer(i) => Ok(i),
            _ => Err(v),
        }
    }
}
