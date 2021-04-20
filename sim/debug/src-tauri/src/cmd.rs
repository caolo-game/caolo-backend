pub mod map_gen;
pub mod room_noise;

use caolo_sim::prelude::*;
use serde::Deserialize;
use svg::node::element::path::Data;
use svg::node::element::Path;
use tracing::error;

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
    // your custom commands
    // multiple arguments are allowed
    GenerateWorld {
        room_radius: u32,
        world_radius: u32,
        callback: String,
        error: String,
    },
    MapNoise {
        room_radius: u32,
        seed: Option<u64>,
        callback: String,
        error: String,
    },
    FindPath {
        from: WorldPosition,
        to: WorldPosition,
        callback: String,
        error: String,
    },
    GetWorld {
        callback: String,
        error: String,
    },
}

pub fn generate_room_noise(room_radius: u32, seed: Option<u64>) -> String {
    room_noise::generate_room_noise_impl(room_radius, seed)
}

pub fn find_path_world(
    from: WorldPosition,
    to: WorldPosition,
    world: &World,
) -> Vec<WorldPosition> {
    use caolo_sim::pathfinding::find_path;
    use caolo_sim::prelude::*;

    let mut path = Vec::with_capacity(512);
    let mut rooms_to_visit = Vec::with_capacity(512);
    if let Err(err) = find_path(
        from,
        to,
        1,
        FromWorld::new(world),
        25000,
        &mut path,
        &mut rooms_to_visit,
    ) {
        error!("Failed to find path {:?}", err);
        return vec![];
    }

    let room = from.room;

    path.into_iter()
        .map(|pos| WorldPosition { room, pos: pos.0 })
        .collect()
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct WorldResponse {
    pub room_id: Axial,
    pub payload: String,
}

/// return the pixel coordinates
pub fn render_hex(path: &mut Path, size: f32, p: Axial) -> [f32; 2] {
    let width = 3.0f32.sqrt() * size;
    let height = 2.0f32 * size;
    let [x, y] = p.to_pixel_pointy(size);

    let pp = [
        [width / 2.0, 0.0],
        [width, height / 4.0],
        [width, height * 3.0 / 4.0],
        [width / 2.0, height],
        [0.0, height * 3.0 / 4.0],
        [0.0, height / 4.0],
    ];

    let mut data = Data::new().move_to((x + pp[0][0], y + pp[0][1]));
    for [px, py] in pp.iter().copied() {
        data = data.line_to((px + x, py + y));
    }

    let p = std::mem::take(path);
    *path = p.set("d", data);

    [x, y]
}
