//! The public API of the game.
//! Wraps the Engine functions in a more managable API
//!
//! (De)Serialization between WASM and the Engine is done via [MessagePack](https://msgpack.org/index.html)
//! When using the API for writing bots this does not effect you.
//!
#[macro_use]
extern crate serde_derive;

pub mod bots;
pub mod pathfinding;
pub mod point;
pub mod resources;
pub mod structures;
pub mod terrain;
pub mod user;

use rmp_serde as rmps;

pub type EntityId = u64;
pub type UserId = uuid::Uuid;

mod external {
    extern "C" {
        pub fn _print(ptr: *const u8, len: i32);
        pub fn _rand_range(from: i32, to: i32) -> i32;
        pub fn _get_max_path_length() -> i32;
        pub fn _get_my_bots_len() -> i32;
        pub fn _get_my_bots(ptr: *mut u8) -> i32;
        pub fn _send_move_intent(ptr: *const u8, len: i32) -> i32;
        pub fn _send_mine_intent(ptr: *const u8, len: i32) -> i32;
        pub fn _send_dropoff_intent(ptr: *const u8, len: i32) -> i32;
        pub fn _find_resources_in_range(q: i32, r: i32, radius: i32, outptr: *mut u8) -> i32;
        pub fn _find_path(fromx: i32, fromy: i32, tox: i32, toy: i32, outptr: *mut u8) -> i32;
    }
}

/// Print to console
#[no_mangle]
pub fn print(s: &str) {
    unsafe {
        external::_print(s.as_ptr(), s.len() as i32);
    }
}

/// Get a random number in the interval [from, to)
#[no_mangle]
pub fn rand_range(from: i32, to: i32) -> i32 {
    unsafe { external::_rand_range(from, to) }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(i32)]
pub enum OperationResult {
    Ok = 0,
    NotOwner = -1,
    InvalidInput = -2,
    OperationFailed = -3,
    NotInRange = -4,
    InvalidTarget = -5,
    Empty = -6,
    Full = -7,
}

impl From<i32> for OperationResult {
    fn from(i: i32) -> OperationResult {
        match i {
            0 => OperationResult::Ok,
            -1 => OperationResult::NotOwner,
            -2 => OperationResult::InvalidInput,
            -3 => OperationResult::OperationFailed,
            -4 => OperationResult::NotInRange,
            -5 => OperationResult::InvalidTarget,
            -6 => OperationResult::Empty,
            -7 => OperationResult::Full,
            _ => panic!("Got an unexpected return code {}", i),
        }
    }
}

#[cfg(test)]
// To be able to link the tests
mod _external {
    #[no_mangle]
    fn _print() {}

    #[no_mangle]
    fn _rand_range() {}
}
