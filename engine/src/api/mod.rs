//! Methods that are exported to the WASM clients
//!
//! Methods that may fail return an OperationResult or the length of the result in bytes.
//!
mod bots;
mod pathfinding;
mod resources;
mod structures;
pub use self::bots::*;
pub use self::pathfinding::*;
pub use self::resources::*;
pub use self::structures::*;
use crate::intents;
use caolo_api::{self, OperationResult};
use rand::Rng;
