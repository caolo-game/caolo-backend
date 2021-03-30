use super::render_hex;
use caolo_sim::geometry::point::Hexagon;
use caolo_sim::noise::PerlinNoise;
use caolo_sim::prelude::WorldPosition;
use caolo_sim::prelude::*;
use svg::node::element::Path;
use svg::Document;

pub fn generate_room_noise_impl(room: Axial, room_radius: u32, seed: Option<u64>) -> String {
    let mut document = Document::new();
    let mut maxx = 0;
    let mut maxy = 0;
    let mut minx = 0;
    let mut miny = 0;

    // hex properties
    let size = 12.0f32;
    let width = 3.0f32.sqrt() * size;
    let height = 2.0f32 * size;

    let noise = PerlinNoise::new(seed);

    let hex = Hexagon::from_radius(room_radius as i32);
    for (p, noise) in hex.iter_points().map(|pos| {
        let wp = WorldPosition { room, pos };
        (pos, noise.world_perlin(wp, room_radius as f32) + 0.5)
    }) {
        let value = (220.0 * noise) as i32;
        let mut path = Path::new().set("fill", format!("rgba({},{},{},1)", value, value, value));

        let [x, y] = render_hex(&mut path, size, p);
        maxx = maxx.max(x as i32);
        maxy = maxx.max(y as i32);
        minx = minx.min(x as i32);
        miny = minx.min(y as i32);

        document = document.add(path);
    }

    let document = document.set(
        "viewBox",
        (
            minx + width as i32 - 1,
            miny - height as i32 - 1,
            maxx + width as i32 + 1,
            maxy + height as i32 + 1,
        ),
    );

    document.to_string()
}
