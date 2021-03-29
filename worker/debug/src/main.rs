use caolo_sim::prelude::*;
use slog::{o, Drain};
use svg::{
    node::element::{path::Data, Path},
    Document,
};

fn render_room(it: impl Iterator<Item = (Axial, TerrainComponent)>) -> svg::Document {
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

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_envlogger::new(drain).fuse();
    let drain = slog_async::Async::new(drain)
        .overflow_strategy(slog_async::OverflowStrategy::Drop)
        .build()
        .fuse();
    let logger = slog::Logger::root(drain, o!());

    let mut exc = SimpleExecutor;
    let world = exc
        .initialize(
            Some(logger.clone()),
            GameConfig {
                world_radius: 1,
                room_radius: 16,
                ..Default::default()
            },
        )
        .unwrap();

    for (room_id, room) in View::<WorldPosition, TerrainComponent>::new(&world).iter_rooms() {
        let doc = render_room(room.iter().map(|(p, t)| (p, *t)));
        let name = format!("out/room-{}-{}.svg", room_id.0.q, room_id.0.r);
        svg::save(name, &doc).unwrap();
    }
}
