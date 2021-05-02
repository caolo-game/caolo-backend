use super::Axial;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Copy, Eq, PartialEq, Serialize, Deserialize, Ord, PartialOrd)]
pub struct Hexagon {
    pub center: Axial,
    pub radius: i32,
}

impl Hexagon {
    pub fn new(center: Axial, radius: i32) -> Self {
        Self { center, radius }
    }

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

    pub fn iter_edge(self) -> impl Iterator<Item = Axial> {
        debug_assert!(
            self.radius >= 0,
            "negative radius will not work as expected"
        );
        let radius = self.radius;
        let starts = [
            self.center + Axial::new(0, -radius),
            self.center + Axial::new(radius, -radius),
            self.center + Axial::new(radius, 0),
            self.center + Axial::new(0, radius),
            self.center + Axial::new(-radius, radius),
            self.center + Axial::new(-radius, 0),
        ];
        let deltas = [
            Axial::new(1, 0),
            Axial::new(0, 1),
            Axial::new(-1, 1),
            Axial::new(-1, 0),
            Axial::new(0, -1),
            Axial::new(1, -1),
        ];
        (0..6).flat_map(move |di| {
            // iterating over `deltas` is a compile error because they're freed at the end of this
            // funciton...
            let delta = deltas[di];
            let pos = starts[di];
            (0..radius).map(move |j| pos + delta * j)
        })
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

    #[test]
    fn test_iter_edge() {
        let pos = Axial::new(0, 0);
        let radius = 4;
        let hex = Hexagon::new(pos, radius);

        let edge: Vec<_> = hex.iter_edge().collect();

        dbg!(hex, &edge);

        assert_eq!(edge.len(), 6 * radius as usize);

        for (i, p) in edge.iter().copied().enumerate() {
            assert_eq!(
                p.hex_distance(pos),
                radius as u32,
                "Hex #{} {:?} is out of range",
                i,
                p
            );
        }
    }
}
