//! Key-Value storages
//!
mod btree;
mod morton;

pub use btree::*;
pub use morton::*;

use super::*;
use crate::storage::TableId;
use std::ops::Add;

pub trait SpatialKey2d: TableId + Add<Output = Self> {
    fn as_array(&self) -> [i32; 2];

    /// Get axis 0 or 1
    fn get_axis(&self, axis: u8) -> i32;

    /// Construct a new key with given coordinates
    fn new(x: i32, y: i32) -> Self;

    /// Distance between two keys
    fn dist(&self, other: &Self) -> u32;

    /// Distance among given axis. Used for separating axis tests to reduce query times when only
    /// one axis is considered..
    fn axis_dist(&self, other: &Self, axis: u8) -> u32 {
        (self.get_axis(axis) - other.get_axis(axis)).abs() as u32
    }
}
