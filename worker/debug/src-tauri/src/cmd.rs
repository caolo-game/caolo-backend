use caolo_sim::prelude::*;
use serde::Deserialize;
use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::Document;

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
}

fn render_room(it: impl Iterator<Item = (Axial, TerrainComponent)>) -> Document {
    let mut document = Document::new().set("viewBox", (0, 0, 2000, 2000));
    for (p, t) in it {
        let path = match t.0 {
            caolo_sim::terrain::TileTerrainType::Empty => continue,
            caolo_sim::terrain::TileTerrainType::Plain => Path::new().set("fill", "yellow"),
            caolo_sim::terrain::TileTerrainType::Bridge => Path::new().set("fill", "green"),
            caolo_sim::terrain::TileTerrainType::Wall => Path::new().set("fill", "red"),
        };
        let size = 12.0f32;
        let [x, y] = p.to_pixel_pointy(size);
        let width = 3.0f32.sqrt() * size;
        let height = 2.0f32 * size;
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

        // TODO: fill by terrain type
        let path = path.set("d", data);

        document = document.add(path);
    }

    document
}

pub fn generate_world(logger: slog::Logger, world_radius: u32, room_radius: u32) -> Vec<String> {
    let mut exc = SimpleExecutor;
    let world = exc
        .initialize(
            Some(logger.clone()),
            GameConfig {
                world_radius,
                room_radius,
                ..Default::default()
            },
        )
        .unwrap();

    View::<WorldPosition, TerrainComponent>::new(&world)
        .iter_rooms()
        .map(|(Room(room_id), room)| {
            let doc = render_room(room.iter().map(|(p, t)| (p, *t)));
            let res = WorldResponse {
                room_id,
                payload: doc.to_string(),
            };
            serde_json::to_string(&res).unwrap()
        })
        .collect()
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct WorldResponse {
    pub room_id: Axial,
    pub payload: String,
}
