use super::Axial;
use cao_lang::traits::AutoByteEncodeProperties;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Copy, Eq, PartialEq, Serialize, Deserialize, Ord, PartialOrd)]
pub struct Hexagon {
    pub center: Axial,
    pub radius: i32,
}

impl Hexagon {
    pub fn from_radius(radius: i32) -> Self {
        debug_assert!(radius >= 0);
        Self {
            radius,
            center: Axial::new(radius, radius),
        }
    }

    pub fn contains(self, point: Axial) -> bool {
        let point = point - self.center;
        let [x, y, z] = point.hex_axial_to_cube();
        let r = self.radius;
        debug_assert!(r >= 0);
        x.abs() <= r && y.abs() <= r && z.abs() <= r
    }

    pub fn iter_points(self) -> impl Iterator<Item = Axial> {
        let radius = self.radius;
        let center = self.center;
        (-radius..=radius).flat_map(move |x| {
            let fromy = (-radius).max(-x - radius);
            let toy = radius.min(-x + radius);
            (fromy..=toy).map(move |y| {
                let p = Axial::new(x, -x - y);
                p + center
            })
        })
    }

    pub fn with_center(mut self, center: Axial) -> Self {
        self.center = center;
        self
    }

    pub fn with_offset(mut self, offset: Axial) -> Self {
        self.center += offset;
        self
    }

    pub fn with_radius(mut self, radius: i32) -> Self {
        self.radius = radius;
        self
    }
}
impl AutoByteEncodeProperties for Hexagon {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_iter_points_are_inside_itself() {
        let hex = Hexagon::from_radius(12);

        for p in hex.iter_points() {
            assert!(hex.contains(p));
        }
    }
}
