pub mod components;
pub mod indices;

pub use self::components::*;
pub use self::indices::*;

pub use caolo_api::point::{Circle, Point};
pub use caolo_api::point3::{Point3, Sphere};
pub use caolo_api::resources::ResourceType;
pub use caolo_api::user::UserData;

impl crate::tables::SpatialKey2d for Point {
    fn as_array(&self) -> [i32; 2] {
        [self.x, self.y]
    }

    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn get_axis(&self, axis: u8) -> i32 {
        match axis {
            0 => self.x,
            1 => self.y,
            _ => unreachable!(),
        }
    }

    fn dist(&self, other: &Self) -> u32 {
        use std::convert::TryFrom;
        u32::try_from(self.hex_distance(*other)).expect("Distance to fit in 32 bits")
    }
}
