use super::pathfinding_room::find_path_in_room;
use super::*;
use crate::{
    prelude::Hexagon,
    tables::{hex_grid::HexGrid, morton_table::MortonTable, morton_hierarchy::SpacialStorage},
    terrain::TileTerrainType,
};

#[test]
fn test_simple_wall() {
    let from = Axial::new(2, 1);
    let to = Axial::new(5, 2);

    let positions = MortonTable::new();
    let mut terrain = HexGrid::new(3);
    terrain
        .extend(
            Hexagon::from_radius(3)
                .iter_points()
                .map(|Axial { q: x, r: y }| {
                    let ty = if x == 3 && y <= 4 {
                        TileTerrainType::Wall
                    } else {
                        TileTerrainType::Plain
                    };

                    (Axial::new(x, y), TerrainComponent(ty))
                }),
        )
        .unwrap();

    let mut path = vec![];
    find_path_in_room(
        from,
        to,
        0,
        (View::from_table(&positions), View::from_table(&terrain)),
        512,
        &mut path,
    )
    .expect("Path finding failed");
    path.reverse();

    let mut current = from;
    for point in path.iter() {
        let point = point.0;
        assert_eq!(point.hex_distance(current), 1);
        if point.q == 3 {
            assert!(point.r > 4, "{:?}", point);
        }
        current = point;
    }
    assert_eq!(current, to);
}

#[test]
fn test_path_is_continous() {
    let from = Axial::new(17, 6);
    let to = Axial::new(7, 16);

    let positions = MortonTable::new();
    let mut terrain = HexGrid::new(12);

    terrain.iter_mut().for_each(|(_, t)| {
        *t = TerrainComponent(TileTerrainType::Plain);
    });

    let mut path = vec![];
    find_path_in_room(
        from,
        to,
        0,
        (View::from_table(&positions), View::from_table(&terrain)),
        512,
        &mut path,
    )
    .expect("Path finding failed");
    path.reverse();

    let mut current = from;
    for point in path.iter() {
        let point = point.0;
        assert_eq!(point.hex_distance(current), 1);
        if point.q == 2 {
            assert!(point.r.abs() > 5, "{:?}", point);
        }
        current = point;
    }
    assert_eq!(current, to);
}

#[test]
fn test_pathfinding_at_distance() {
    let from = Axial::new(17, 6);
    let to = Axial::new(7, 16);

    let positions = MortonTable::new();
    let mut terrain = HexGrid::new(12);

    terrain.iter_mut().for_each(|(_, t)| {
        *t = TerrainComponent(TileTerrainType::Plain);
    });

    let mut path = vec![];
    find_path_in_room(
        from,
        to,
        2,
        (View::from_table(&positions), View::from_table(&terrain)),
        512,
        &mut path,
    )
    .expect("Path finding failed");
    path.reverse();

    let mut current = from;
    for point in path.iter() {
        let point = point.0;
        assert_eq!(point.hex_distance(current), 1);
        if point.q == 2 {
            assert!(point.r.abs() > 5, "{:?}", point);
        }
        current = point;
    }
    assert_eq!(current.hex_distance(to), 2);
}
