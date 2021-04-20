use super::render_hex;
use super::WorldResponse;
use caolo_sim::prelude::*;
use caolo_sim::terrain::TileTerrainType;
use svg::node::element::Path;
use svg::Document;

fn render_room(it: impl Iterator<Item = (Axial, TileTerrainType)>) -> Document {
    let mut document = Document::new();
    let mut maxx = 0;
    let mut maxy = 0;
    let mut minx = 0;
    let mut miny = 0;

    // hex properties
    let size = 12.0f32;
    let width = 3.0f32.sqrt() * size;
    let height = 2.0f32 * size;
    for (p, t) in it {
        let mut path = match t {
            TileTerrainType::Empty => Path::new().set("fill", "lightblue"),
            TileTerrainType::Plain => Path::new().set("fill", "yellow"),
            TileTerrainType::Bridge => Path::new().set("fill", "green"),
            TileTerrainType::Wall => Path::new().set("fill", "red"),
        };
        let [x, y] = render_hex(&mut path, size, p);
        maxx = maxx.max(x as i32);
        maxy = maxx.max(y as i32);
        minx = minx.min(x as i32);
        miny = minx.min(y as i32);

        document = document.add(path);
    }

    document.set(
        "viewBox",
        (
            minx + width as i32 - 1,
            miny - height as i32 - 1,
            maxx + width as i32 + 1,
            maxy + height as i32 + 1,
        ),
    )
}

#[derive(serde::Serialize)]
pub struct TerrainRoom {
    room_id: Axial,
    terrain: Vec<(Axial, TileTerrainType)>,
}

pub fn get_terrain(world: &World) -> Vec<TerrainRoom> {
    View::<WorldPosition, TerrainComponent>::new(&world)
        .iter_rooms()
        .map(|(Room(room_id), room)| {
            let res = TerrainRoom {
                room_id,
                terrain: room.iter().map(|(p, t)| (p, t.0)).collect(),
            };
            res
        })
        .collect()
}

pub fn render_terrain(world: &World) -> Vec<String> {
    let terrain = get_terrain(&world);
    terrain
        .into_iter()
        .map(|TerrainRoom { room_id, terrain }| {
            let doc = render_room(terrain.iter().map(|(p, t)| (*p, *t)));
            let res = WorldResponse {
                room_id,
                payload: doc.to_string(),
            };
            serde_json::to_string(&res).unwrap()
        })
        .collect()
}
