//! Generate rooms
//!
mod params;

pub use params::*;

use crate::geometry::{Axial, Hexagon};
use crate::indices::WorldPosition;
use crate::storage::views::{UnsafeView, View};
use crate::tables::morton_hierarchy::SpacialStorage;
use crate::tables::{morton::msb_de_bruijn, square_grid::HexGrid};
use crate::terrain::TileTerrainType;
use crate::{
    components::{RoomConnection, TerrainComponent},
    noise::PerlinNoise,
};
use rand::{prelude::SmallRng, Rng, SeedableRng};
use slog::{debug, error, trace, Logger};
use std::cmp::Ordering;
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone, thiserror::Error)]
pub enum RoomGenerationError {
    #[error("Can not generate room with the given parameters: {radius}")]
    BadArguments { radius: u32 },
    #[error("A room may only have up to 6 neihgbours, got: {0}")]
    TooManyNeighbours(usize),
    #[error("Got an invlid neighbour {0:?}")]
    InvalidNeighbour(Axial),
    #[error("Internal error: Failed to connect chunks, remaining: {0:?}")]
    ExpectedSingleChunk(usize),
    #[error("Bad edge offsets at edge {edge:?} with a radius of {radius}. Start is {offset_start} and End is {offset_end}")]
    BadEdgeOffset {
        edge: Axial,
        offset_start: i32,
        offset_end: i32,
        radius: i32,
    },
}

type MapTables = (UnsafeView<Axial, TerrainComponent>,);

type GradientMap = HexGrid<f32>;

/// find the smallest power of two that can hold `size`
fn pot(size: u32) -> u32 {
    if size & (size - 1) == 0 {
        size
    } else {
        let msb = msb_de_bruijn(size);
        1 << (msb + 1)
    }
}

#[derive(Debug, Clone)]
pub struct HeightMapProperties {
    pub radius: i32,
    /// standard deviation of the height map
    pub std: f32,
    /// mean height of the map
    pub mean: f32,
    /// standard deviation of the normalized height map
    pub normal_std: f32,
    /// mean of normalized heights
    pub normal_mean: f32,
    pub min: f32,
    pub max: f32,
    /// max - min
    pub depth: f32,
    pub width: i32,
    pub height: i32,
}

/// Generate a random terrain in hexagon
/// `edges` is a list of neighbours to connect to, meaning these edges are
/// reachable via land.
///
/// Returns property description of the generated height map.
pub fn generate_room(
    logger: Logger,
    params: &RoomGenerationParams,
    edges: &[RoomConnection],
    (mut terrain,): MapTables,
) -> Result<HeightMapProperties, RoomGenerationError> {
    debug!(logger, "Generating Room {:#?}\nedges:\n{:?}", params, edges);
    let RoomGenerationParams { room, radius, .. } = params;
    if edges.len() > 6 {
        return Err(RoomGenerationError::TooManyNeighbours(edges.len()));
    }
    let mut rng = SmallRng::seed_from_u64(params.seed);

    let radius = *radius as i32;
    let dsides = pot(radius as u32 * 2) as i32;

    terrain.clear();
    terrain.resize(dsides);
    let mut gradient = GradientMap::new(dsides as usize);

    let center = Axial::new(radius, radius);

    let mut min_grad = 1e15f32;
    let mut max_grad = -1e15f32;

    trace!(logger, "Generating heightmap");
    let noise = PerlinNoise::new(params.seed);
    for pos in Hexagon::from_radius(dsides).iter_points() {
        let grad = noise.world_perlin(WorldPosition { pos, room: room.0 }, radius as f32);
        gradient.insert(pos, grad).unwrap();
        min_grad = min_grad.min(grad);
        max_grad = max_grad.max(grad);
    }
    trace!(logger, "Generating heightmap done");

    let heightmap_props = transform_heightmap_into_terrain(
        &logger,
        HeightMapTransformParams {
            max_grad,
            min_grad,
            dsides,
            radius: radius - 1,
            chance_plain: params.chance_plain,
            chance_wall: params.chance_wall,
        },
        &gradient,
        terrain,
    )?;

    {
        // ensure at least 1 plain at this point
        let r2 = radius / 2;
        let minq = center.q - r2;
        let minr = center.r - r2;
        let maxq = center.q + r2;
        let maxr = center.r + r2;

        let q = rng.gen_range(minq, maxq);
        let r = rng.gen_range(minr, maxr);
        terrain
            .insert(Axial::new(q, r), TerrainComponent(TileTerrainType::Plain))
            .map_err(|e| {
                error!(logger, "Failed to update the center point {:?}", e);
            })
            .expect("Failed to update center");
    }

    if params.plain_dilation > 0 {
        // to make dilation unbiased we clone the terrain and inject that as separate input
        let terrain_in: <TerrainComponent as crate::tables::Component<Axial>>::Table =
            (*terrain).clone();
        dilate(
            logger.clone(),
            center,
            radius,
            params.plain_dilation,
            View::from_table(&terrain_in),
            terrain,
        );
    }

    coastline(&logger, radius - 1, terrain);

    let chunk_metadata = calculate_plain_chunks(&logger, View::from_table(&*terrain));
    if chunk_metadata.chunks.len() > 1 {
        connect_chunks(
            &logger,
            radius - 1,
            &mut rng,
            &chunk_metadata.chunks,
            terrain,
        );
    }

    fill_edges(logger.clone(), edges, terrain, &mut rng)?;

    trace!(logger, "Cutting outliers");
    // cut the edges, because generation might insert invalid Plains on the edge
    let bounds = Hexagon { center, radius };
    let delegates: Vec<Axial> = terrain
        .iter()
        .filter_map(|(p, t)| {
            (!bounds.contains(p) && !matches!(t, TerrainComponent(TileTerrainType::Empty)))
                .then(|| p)
        })
        .collect();
    trace!(logger, "Deleting {} items from the room", delegates.len());
    for p in delegates {
        terrain[p] = TerrainComponent(TileTerrainType::Empty);
    }
    trace!(logger, "Cutting outliers done");

    debug!(logger, "Map generation done {:#?}", heightmap_props);
    Ok(heightmap_props)
}

fn fill_edges(
    logger: Logger,
    edges: &[RoomConnection],
    terrain: UnsafeView<Axial, TerrainComponent>,
    rng: &mut impl Rng,
) -> Result<(), RoomGenerationError> {
    trace!(logger, "Filling edges");
    let Hexagon { center, radius } = terrain.bounds();
    let mut chunk_metadata = calculate_plain_chunks(&logger, View::from_table(&*terrain));
    if chunk_metadata.chunks.len() != 1 {
        error!(
            logger,
            "Expected 1 single chunk when applying edges, intead got {}",
            chunk_metadata.chunks.len()
        );
        return Err(RoomGenerationError::ExpectedSingleChunk(
            chunk_metadata.chunks.len(),
        ));
    }
    for mut edge in edges.iter().cloned() {
        // offset - 1 but at least 0
        edge.offset_start = 1.max(edge.offset_start) - 1;
        edge.offset_end = 1.max(edge.offset_end) - 1;
        chunk_metadata
            .chunks
            .push(HashSet::with_capacity(radius as usize));
        fill_edge(
            &logger,
            center,
            radius - 1,
            TileTerrainType::Plain,
            &edge,
            terrain,
            chunk_metadata.chunks.last_mut().unwrap(),
        )?;
    }
    trace!(logger, "Connecting edges to the mainland");
    connect_chunks(&logger, radius - 2, rng, &chunk_metadata.chunks, terrain);
    trace!(logger, "Filling edges done");
    for edge in edges.iter() {
        chunk_metadata
            .chunks
            .push(HashSet::with_capacity(radius as usize));
        fill_edge(
            &logger,
            center,
            radius,
            TileTerrainType::Bridge,
            edge,
            terrain,
            chunk_metadata.chunks.last_mut().unwrap(),
        )?;
    }
    Ok(())
}

fn dilate(
    logger: Logger,
    center: Axial,
    radius: i32,
    kernel_width: u32,
    terrain_in: View<Axial, TerrainComponent>,
    mut terrain_out: UnsafeView<Axial, TerrainComponent>,
) {
    trace!(
        logger,
        "Dilating terrain center: {:?} radius: {} kernel: {}",
        center,
        radius,
        kernel_width
    );
    if radius < kernel_width as i32 + 1 || kernel_width == 0 {
        trace!(logger, "Skipping dilating");
        return;
    }

    let threshold = (kernel_width * kernel_width / 3).max(1);

    let points = Hexagon {
        center,
        radius: radius - 1,
    }
    .iter_points();
    for (p, _) in points
        .filter_map(|p| terrain_in.at(p).map(|t| (p, t)))
        .filter(|(_, t)| !t.0.is_walkable())
    {
        let mut neighbours_on = -1; // account for p
        terrain_in.query_range(p, kernel_width, &mut |_, TerrainComponent(t)| {
            neighbours_on += t.is_walkable() as i32;
        });

        if neighbours_on > threshold as i32 {
            terrain_out[p] = TerrainComponent(TileTerrainType::Plain);
        }
    }
    trace!(logger, "Dilate done");
}

fn connect_chunks(
    logger: &Logger,
    radius: i32,
    rng: &mut impl Rng,
    chunks: &[HashSet<Axial>],
    mut terrain: UnsafeView<Axial, TerrainComponent>,
) {
    trace!(logger, "Connecting {} chunks", chunks.len());
    debug_assert!(radius > 0);
    let mut bounds = Hexagon::from_radius(radius - 1);
    bounds.center += Axial::new(1, 1);

    'chunks: for chunk in chunks[1..].iter() {
        let avg: Axial =
            chunk.iter().cloned().fold(Axial::default(), |a, b| a + b) / chunk.len() as i32;
        let closest = *chunks[0]
            .iter()
            .min_by_key(|p| p.hex_distance(avg))
            .unwrap();
        let mut current = *chunk
            .iter()
            .min_by_key(|p| p.hex_distance(closest))
            .unwrap();

        let get_next_step = |current| {
            let vel = closest - current;
            debug_assert!(vel.q != 0 || vel.r != 0);
            match vel.q.abs().cmp(&vel.r.abs()) {
                Ordering::Equal => {
                    if (vel.q + vel.r) % 2 == 0 {
                        Axial::new(vel.q / vel.q.abs(), 0)
                    } else {
                        Axial::new(0, vel.r / vel.r.abs())
                    }
                }
                Ordering::Less => Axial::new(0, vel.r / vel.r.abs()),
                Ordering::Greater => Axial::new(vel.q / vel.q.abs(), 0),
            }
        };

        if current.hex_distance(closest) <= 1 {
            continue 'chunks;
        }
        'connecting: loop {
            let vel = get_next_step(current);
            current += vel;
            terrain[current] = TerrainComponent(TileTerrainType::Plain);
            if current.hex_distance(closest) == 0 {
                break 'connecting;
            }
            for _ in 0..4 {
                let vel = if rng.gen_bool(0.5) {
                    vel.rotate_left()
                } else {
                    vel.rotate_right()
                };
                let c = current + vel;
                if !bounds.contains(c) {
                    continue;
                }
                current = c;
                terrain[current] = TerrainComponent(TileTerrainType::Plain);
                if current.hex_distance(closest) < 1 {
                    break 'connecting;
                }
            }
        }
    }
    trace!(logger, "Connecting chunks done");
}

/// Turn every `Wall` into `Plain` if it has empty neighbour(s).
/// This should result in a nice coastline where the `Walls` were neighbours with the ocean.
fn coastline(logger: &Logger, radius: i32, mut terrain: UnsafeView<Axial, TerrainComponent>) {
    trace!(logger, "Building coastline");
    let mut changeset = vec![];
    'walle: for wall_pos in Hexagon::from_radius(radius).iter_points().filter(|p| {
        matches!(
            terrain.at(*p),
            Some(TerrainComponent(TileTerrainType::Wall))
        )
    }) {
        for n in wall_pos.hex_neighbours().iter().cloned() {
            if matches!(
                terrain.at(n),
                Some(TerrainComponent(TileTerrainType::Empty)) | None
            ) {
                changeset.push(wall_pos);
                continue 'walle;
            }
        }
    }
    trace!(logger, "Changing walls to plains {:#?}", changeset);
    for p in changeset.iter() {
        terrain[*p] = TerrainComponent(TileTerrainType::Plain);
    }
    trace!(logger, "Building coastline done");
}

struct HeightMapTransformParams {
    max_grad: f32,
    min_grad: f32,
    dsides: i32,
    radius: i32,
    chance_plain: f32,
    chance_wall: f32,
}

fn transform_heightmap_into_terrain(
    logger: &Logger,
    HeightMapTransformParams {
        max_grad,
        min_grad,
        dsides,
        radius,
        chance_plain,
        chance_wall,
    }: HeightMapTransformParams,
    gradient: &HexGrid<f32>,
    mut terrain: UnsafeView<Axial, TerrainComponent>,
) -> Result<HeightMapProperties, RoomGenerationError> {
    trace!(logger, "Building terrain from height-map");
    let mut mean = 0.0;
    let mut std = 0.0;
    let mut normal_mean = 0.0;
    let mut normal_std = 0.0;
    let mut i = 1.0;
    let depth = max_grad - min_grad;

    terrain.clear();
    terrain.resize(radius);

    let terrain_bounds: Hexagon = terrain.bounds();
    let tg_vec = gradient.bounds().center - terrain_bounds.center; // from terrain to gradient center displacement
    debug_assert!(terrain_bounds.radius <= gradient.bounds().radius);
    trace!(logger, "Calculating points of a hexagon in the height map",);
    trace!(
        logger,
        "Terrain bounds: {:?} Gradient bounds: {:?}",
        terrain.bounds(),
        gradient.bounds(),
    );
    terrain
        .extend(terrain_bounds.iter_points().map(|p| {
            trace!(logger, "Computing terrain of gradient point: {:?}", p);
            let mut grad = match gradient.at(tg_vec + p).copied() {
                Some(g) => g,
                None => {
                    error!(logger, "{:?} has no gradient", p);
                    debug_assert!(false);
                    return (p, TerrainComponent(TileTerrainType::Empty));
                }
            };
            trace!(logger, "p: {:?} grad: {}", p, grad);

            {
                // let's do some stats
                let tmp = grad - mean;
                mean += tmp / i;
                std += tmp * (grad - mean);
            }

            // normalize grad to [0-1]
            grad -= min_grad;
            grad /= depth;

            {
                // let's do some stats on the normal
                let tmp = grad - normal_mean;
                normal_mean += tmp / i;
                normal_std += tmp * (grad - normal_mean);
                i += 1.0;
            }

            trace!(logger, "Normalized grad: {}", grad);

            if !grad.is_finite() {
                return (p, TerrainComponent(TileTerrainType::Empty));
            }
            let terrain = if grad <= chance_plain {
                TileTerrainType::Plain
            } else if grad <= chance_plain + chance_wall {
                TileTerrainType::Wall
            } else {
                TileTerrainType::Empty
            };
            (p, TerrainComponent(terrain))
        }))
        .expect("Terrain building failed");

    trace!(logger, "Building terrain from height-map done");
    std = (std / i).sqrt();
    normal_std = (normal_std / i).sqrt();

    let props = HeightMapProperties {
        radius,
        normal_mean,
        normal_std,
        std,
        mean,
        min: min_grad,
        max: max_grad,
        depth,
        width: dsides,
        height: dsides,
    };

    Ok(props)
}

fn fill_edge(
    logger: &Logger,
    center: Axial,
    radius: i32,
    ty: TileTerrainType,
    edge: &RoomConnection,
    mut terrain: UnsafeView<Axial, TerrainComponent>,
    chunk: &mut HashSet<Axial>,
) -> Result<(), RoomGenerationError> {
    trace!(logger, "Filling edge {:?}", edge);
    terrain
        .extend(iter_edge(center, radius as u32, edge)?.map(move |vertex| {
            chunk.insert(vertex);
            (vertex, TerrainComponent(ty))
        }))
        .expect("Failed to expand terrain with edge");

    Ok(())
}

pub fn iter_edge(
    center: Axial,
    radius: u32,
    RoomConnection {
        offset_start,
        offset_end,
        direction: edge,
    }: &RoomConnection,
) -> Result<impl Iterator<Item = Axial>, RoomGenerationError> {
    let radius = radius as i32;
    if edge.q.abs() > 1 || edge.r.abs() > 1 || edge.r == edge.q {
        return Err(RoomGenerationError::InvalidNeighbour(*edge));
    }
    let end = edge.rotate_right();
    let vel = end - *edge;

    let vertex = (*edge * radius) + center;

    let offset_start = *offset_start as i32;
    let offset_end = *offset_end as i32;
    if radius - offset_start - offset_end <= 0 {
        return Err(RoomGenerationError::BadEdgeOffset {
            radius,
            edge: *edge,
            offset_start,
            offset_end,
        });
    }

    let it = (offset_start..(radius - offset_end)).map(move |i| vertex + (vel * i));
    Ok(it)
}

struct ChunkMeta {
    pub chungus_mass: usize,
    pub chunks: Vec<HashSet<Axial>>,
}

/// Find the connecting `Plain` chunks.
/// The first one will be the largest chunk
fn calculate_plain_chunks(logger: &Logger, terrain: View<Axial, TerrainComponent>) -> ChunkMeta {
    trace!(logger, "calculate_plain_chunks");
    let mut visited = HashSet::new();
    let mut todo = VecDeque::new();
    let mut startind = 0;
    let mut chunk_id = 0;

    let mut chungus_id = 0;
    let mut chungus_mass = 0;
    let mut chunks = Vec::with_capacity(4);
    'a: loop {
        let current = terrain
            .iter()
            .enumerate()
            .skip(startind)
            .find_map(|(i, (p, t))| (t.0.is_walkable() && !visited.contains(&p)).then(|| (i, p)));
        if current.is_none() {
            break 'a;
        }
        let (i, current) = current.unwrap();
        startind = i;
        todo.clear();
        todo.push_back(current);
        let mut chunk = HashSet::new();

        while let Some(current) = todo.pop_front() {
            if !visited.insert(current) {
                continue;
            }
            chunk.insert(current);
            terrain.query_range(current, 1, &mut |p, t| {
                let TerrainComponent(t) = t;
                if t.is_walkable() && !visited.contains(&p) {
                    todo.push_back(p);
                }
            });
        }
        let mass = chunk.len();
        if mass > chungus_mass {
            chungus_mass = mass;
            chungus_id = chunk_id;
        }
        chunks.push(chunk);
        chunk_id += 1;
    }
    if chunks.len() >= 2 {
        chunks.swap(0, chungus_id);
    }
    trace!(
        logger,
        "calculate_plain_chunks done, found {} chunks",
        chunks.len()
    );
    debug_assert!(
        chunks
            .iter()
            .zip(chunks.iter().skip(1))
            .find(|(a, b)| !a.is_disjoint(b))
            .is_none(),
        "Internal error: chunks must be disjoint!"
    );
    ChunkMeta {
        chungus_mass,
        chunks,
    }
}

/// Print a 2D TerrainComponent map to the console, intended for debugging small maps.
#[allow(unused)]
fn print_terrain(from: Axial, to: Axial, terrain: View<Axial, TerrainComponent>) {
    assert!(from.q < to.q);
    assert!(from.r < to.r);

    for y in (from.r..=to.r) {
        for x in (from.q..=to.q) {
            match terrain.at(Axial::new(x, y)) {
                Some(TerrainComponent(TileTerrainType::Wall)) => print!("#"),
                Some(TerrainComponent(TileTerrainType::Plain)) => print!("."),
                Some(TerrainComponent(TileTerrainType::Bridge)) => print!("x"),
                Some(TerrainComponent(TileTerrainType::Empty)) | None => print!(" "),
            }
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pathfinding::find_path_in_room;
    use crate::storage::views::View;
    use crate::utils::*;
    use crate::{components::EntityComponent, tables::morton::MortonTable};
    use slog::{o, Drain};

    #[test]
    fn maps_are_not_homogeneous() {
        setup_testing();
        let logger = test_logger();

        let mut terrain = HexGrid::new(8);

        let params = RoomGenerationParams::builder()
            .with_radius(8)
            .with_plain_dilation(1)
            .build()
            .unwrap();

        let props = generate_room(
            logger,
            &params,
            &[],
            (UnsafeView::from_table(&mut terrain),),
        )
        .unwrap();

        dbg!(props);

        let from = Axial::new(0, 0);
        let to = Axial::new(16, 16);
        print_terrain(from, to, View::from_table(&terrain));

        let mut seen_empty = false;
        let mut seen_wall = false;
        let mut seen_plain = false;

        // assert that the terrain is not homogeneous
        // check points in radius-1 to account for the bridges
        let center = Axial::new(8, 8);
        let points = Hexagon { center, radius: 7 }.iter_points();
        for point in points {
            match terrain.at(point) {
                Some(TerrainComponent(TileTerrainType::Empty)) | None => seen_empty = true,
                Some(TerrainComponent(TileTerrainType::Plain))
                | Some(TerrainComponent(TileTerrainType::Bridge)) => seen_plain = true,
                Some(TerrainComponent(TileTerrainType::Wall)) => seen_wall = true,
            }
        }

        assert!(seen_plain);
        assert!(seen_wall || seen_empty);
    }

    #[test]
    fn all_plain_are_reachable() {
        let logger = test_logger();

        const RADIUS: i32 = 8;

        // doesn't work all the time...
        let mut plains = Vec::with_capacity(512);
        let mut terrain = HexGrid::new(0);

        let params = RoomGenerationParams::builder()
            .with_radius(RADIUS as u32)
            .with_plain_dilation(1)
            .build()
            .unwrap();
        let props = generate_room(
            logger,
            &params,
            &[],
            (UnsafeView::from_table(&mut terrain),),
        )
        .unwrap();

        dbg!(props);

        for p in terrain.bounds().iter_points() {
            let TerrainComponent(tile) = terrain[p];
            if tile.is_walkable() {
                plains.push(p);
            }
        }

        let from = Axial::new(0, 0);
        let to = terrain.bounds().center + Axial::new(RADIUS, RADIUS);

        print_terrain(from, to, View::from_table(&terrain));

        let positions = MortonTable::<EntityComponent>::new();
        let mut path = Vec::with_capacity(1024);

        let first = plains.iter().next().expect("at least 1 plain");
        let logger = slog::Logger::root(slog_stdlog::StdLog.fuse(), o!());
        for b in plains.iter().skip(1) {
            path.clear();
            if let Err(err) = find_path_in_room(
                &logger,
                *first,
                *b,
                (View::from_table(&positions), View::from_table(&terrain)),
                10240,
                &mut path,
            ) {
                panic!("Failed to find path from {:?} to {:?}: {:?}", first, b, err);
            }
        }
    }
}
