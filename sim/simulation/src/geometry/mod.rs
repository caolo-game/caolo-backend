mod axial;
mod hexagon;

pub use axial::*;
pub use hexagon::*;

pub fn aabb_over_circle(center: Axial, radius: u32) -> (Axial, Axial) {
    let [x, y] = center.as_array();
    let radius = radius as i32;
    let from = Axial::new(x - radius, y - radius);
    let to = Axial::new(x + radius, y + radius);

    (from, to)
}
