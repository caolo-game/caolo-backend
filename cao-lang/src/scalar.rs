use crate::compiler::NodeId;
use crate::traits::AutoByteEncodeProperties;
use crate::TPointer;
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Scalar {
    Pointer(TPointer),
    IScalar(i32),
    FScalar(f32),
    Label(NodeId),
}

impl Scalar {
    pub fn as_bool(self) -> bool {
        use Scalar::*;
        match self {
            Pointer(i) => i != 0,
            IScalar(i) => i != 0,
            FScalar(i) => i != 0.0,
            Label(_) => true,
        }
    }
}

impl AutoByteEncodeProperties for Scalar {}

impl TryFrom<Scalar> for i32 {
    type Error = Scalar;

    fn try_from(v: Scalar) -> Result<Self, Scalar> {
        match v {
            Scalar::Label(i) | Scalar::IScalar(i) => Ok(i),
            _ => Err(v),
        }
    }
}

impl TryFrom<Scalar> for f32 {
    type Error = Scalar;

    fn try_from(v: Scalar) -> Result<Self, Scalar> {
        match v {
            Scalar::FScalar(i) => Ok(i),
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
