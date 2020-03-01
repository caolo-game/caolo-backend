use super::*;
use rand::prelude::*;
use std::collections::HashMap;
use test::Bencher;

#[test]
fn insertions() {
    let mut table = MortonTable::new();

    let r = table.insert(Point::new(16, 32), 123i32);
    assert!(r);
}

#[test]
fn test_range_query_all() {
    let mut rng = rand::thread_rng();

    let mut table = MortonTable::new();

    for i in 0..256 {
        let p = Point {
            x: rng.gen_range(0, 128),
            y: rng.gen_range(0, 128),
        };
        let inserted = table.insert(p, i);
        assert!(inserted);
    }

    let mut res = Vec::new();
    table.find_by_range(&Point::default(), 256, &mut res);

    assert_eq!(res.len(), 256);
}
#[test]
fn get_by_id_bugged_() {
    let points = [
        Point { x: 3, y: 10 },
        Point { x: 5, y: 11 },
        Point { x: 63, y: 5 },
        Point { x: 50, y: 8 },
        Point { x: 63, y: 9 },
        Point { x: 39, y: 25 },
        Point { x: 53, y: 27 },
        Point { x: 14, y: 37 },
        Point { x: 0, y: 46 },
        Point { x: 1, y: 61 },
        Point { x: 30, y: 53 },
        Point { x: 36, y: 39 },
        Point { x: 46, y: 32 },
        Point { x: 58, y: 38 },
        Point { x: 38, y: 59 },
        Point { x: 54, y: 49 },
        Point { x: 82, y: 4 },
        Point { x: 84, y: 14 },
        Point { x: 74, y: 20 },
        Point { x: 77, y: 30 },
        Point { x: 83, y: 23 },
        Point { x: 112, y: 11 },
        Point { x: 99, y: 18 },
        Point { x: 115, y: 29 },
        Point { x: 70, y: 37 },
        Point { x: 64, y: 40 },
        Point { x: 82, y: 32 },
        Point { x: 86, y: 36 },
        Point { x: 70, y: 53 },
        Point { x: 99, y: 35 },
        Point { x: 97, y: 43 },
        Point { x: 108, y: 42 },
        Point { x: 107, y: 62 },
        Point { x: 122, y: 63 },
        Point { x: 17, y: 67 },
        Point { x: 29, y: 66 },
        Point { x: 10, y: 89 },
        Point { x: 31, y: 94 },
        Point { x: 42, y: 75 },
        Point { x: 49, y: 64 },
        Point { x: 62, y: 66 },
        Point { x: 33, y: 90 },
        Point { x: 59, y: 82 },
        Point { x: 60, y: 85 },
        Point { x: 53, y: 93 },
        Point { x: 16, y: 105 },
        Point { x: 55, y: 109 },
        Point { x: 38, y: 121 },
        Point { x: 41, y: 127 },
        Point { x: 73, y: 70 },
        Point { x: 75, y: 70 }, // this is the ficked fucked fuckery
        Point { x: 65, y: 78 },
        Point { x: 76, y: 73 },
        Point { x: 95, y: 65 },
        Point { x: 92, y: 69 },
        Point { x: 87, y: 75 },
        Point { x: 117, y: 69 },
        Point { x: 98, y: 84 },
        Point { x: 120, y: 83 },
        Point { x: 88, y: 97 },
        Point { x: 99, y: 118 },
        Point { x: 110, y: 126 },
        Point { x: 126, y: 122 },
    ];
    let points: Vec<(_, _)> = points
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, p)| (p, i))
        .collect();

    let mut table = MortonTable::<Point, usize>::new();
    for (p, e) in points.iter() {
        let inserted = table.insert(p.clone(), *e);
        assert!(inserted);
    }

    for p in points {
        let found = table.get_by_id(&p.0);
        let key = MortonKey::new(p.0.x as u16, p.0.y as u16);
        assert_eq!(found, Some(&p.1), "{:?} {:?}", p.0, key);
    }
}

#[test]
fn get_by_id() {
    let mut rng = rand::thread_rng();

    let mut table = MortonTable::<Point, usize>::new();

    let mut points = Vec::with_capacity(64);

    for i in 0..64 {
        let p = Point {
            x: rng.gen_range(0, 128),
            y: rng.gen_range(0, 128),
        };
        points.push((p, i));
    }

    for (p, e) in points.iter() {
        let inserted = table.insert(p.clone(), *e);
        assert!(inserted);
    }

    println!("{:?}", table);

    for p in points {
        let found = table.get_by_id(&p.0);
        let key = MortonKey::new(p.0.x as u16, p.0.y as u16);
        assert_eq!(found, Some(&p.1), "{:?} {:?}", p.0, key);
    }
}

#[bench]
fn bench_contains_rand_at_2pow16(b: &mut Bencher) {
    let mut rng = rand::thread_rng();

    let table = MortonTable::from_iterator((0..(1 << 16)).map(|i| {
        let p = Point {
            x: rng.gen_range(0, 8000),
            y: rng.gen_range(0, 8000),
        };
        (p, i)
    }));

    b.iter(|| {
        let p = Point {
            x: rng.gen_range(0, 8000),
            y: rng.gen_range(0, 8000),
        };
        table.contains_key(&p)
    });
}

#[bench]
fn bench_range_query(b: &mut Bencher) {
    let mut rng = rand::thread_rng();

    let mut table = MortonTable::new();

    for i in 0..(1 << 15) {
        let p = Point {
            x: rng.gen_range(0, 3900 * 2),
            y: rng.gen_range(0, 3900 * 2),
        };
        let inserted = table.insert(p, i);
        assert!(inserted);
    }

    let mut res = Vec::with_capacity(512);
    let radius = 512;

    b.iter(|| {
        let table = &table;
        res.clear();
        let p = Point {
            x: rng.gen_range(0, 3900 * 2),
            y: rng.gen_range(0, 3900 * 2),
        };
        table.find_by_range(&p, radius, &mut res);
        res.len()
    });
}

#[bench]
fn bench_get_entities_in_range_sparse(b: &mut Bencher) {
    let mut rng = rand::thread_rng();

    let table = MortonTable::from_iterator((0..1 << 12).map(|_| {
        let p = Point {
            x: rng.gen_range(0, 3900 * 2),
            y: rng.gen_range(0, 3900 * 2),
        };
        (p, EntityComponent(EntityId(rng.gen())))
    }));

    let radius = 512;

    b.iter(|| {
        let table = &table;
        let p = Point {
            x: rng.gen_range(0, 3900 * 2),
            y: rng.gen_range(0, 3900 * 2),
        };
        table.get_entities_in_range(&Circle { center: p, radius })
    });
}

#[bench]
fn bench_get_entities_in_range_dense(b: &mut Bencher) {
    use crate::tables::traits::PositionTable;
    let mut rng = rand::thread_rng();

    let table = MortonTable::from_iterator((0..1 << 12).map(|_| {
        let p = Point {
            x: rng.gen_range(0, 200 * 2),
            y: rng.gen_range(0, 200 * 2),
        };
        (p, EntityComponent(EntityId(rng.gen())))
    }));

    let radius = 50;

    b.iter(|| {
        let table = &table;
        let p = Point {
            x: rng.gen_range(0, 200 * 2),
            y: rng.gen_range(0, 200 * 2),
        };
        table.get_entities_in_range(&Circle { center: p, radius })
    });
}

#[bench]
fn make_morton_table(b: &mut Bencher) {
    let mut rng = rand::thread_rng();

    b.iter(|| {
        let table = MortonTable::from_iterator((0..(1 << 15)).map(|_| {
            (
                Point {
                    x: rng.gen_range(0, 3900 * 2),
                    y: rng.gen_range(0, 3900 * 2),
                },
                rng.next_u32(),
            )
        }));
        table
    });
}

#[bench]
fn rebuild_morton_table(b: &mut Bencher) {
    let mut rng = rand::thread_rng();

    let mut table = MortonTable::with_capacity(1 << 15);

    b.iter(|| {
        table.clear();

        table.extend((0..(1 << 15)).map(|_| {
            (
                Point {
                    x: rng.gen_range(0, 3900 * 2),
                    y: rng.gen_range(0, 3900 * 2),
                },
                rng.next_u32(),
            )
        }));
    })
}

#[bench]
fn bench_get_by_id_rand_2pow16(b: &mut Bencher) {
    let mut rng = rand::thread_rng();

    let len = 1 << 16;
    let table = MortonTable::from_iterator((0..len).map(|_| {
        let pos = Point {
            x: rng.gen_range(0, 3900 * 2),
            y: rng.gen_range(0, 3900 * 2),
        };
        (pos, rng.next_u32())
    }));

    b.iter(|| {
        let pos = Point {
            x: rng.gen_range(0, 3900 * 2),
            y: rng.gen_range(0, 3900 * 2),
        };
        table.get_by_id(&pos)
    });
}

#[test]
fn from_iterator_inserts_correctly() {
    let mut rng = rand::thread_rng();

    let len = 1 << 12;
    let mut points = HashMap::with_capacity(len);
    let table = MortonTable::from_iterator((0..len).filter_map(|_| {
        let pos = Point {
            x: rng.gen_range(0, 3900 * 2),
            y: rng.gen_range(0, 3900 * 2),
        };
        if !points.contains_key(&pos) {
            return None;
        }
        let val = rng.next_u32();
        points.insert(pos.clone(), val);
        Some((pos, val))
    }));

    for (pos, val) in points {
        let v = *table.get_by_id(&pos).expect("to find the value");
        assert_eq!(val, v);
    }
}

#[bench]
fn bench_get_by_id_in_table_rand_2pow16(b: &mut Bencher) {
    let mut rng = rand::thread_rng();

    let len = 1 << 16;
    let mut points = Vec::with_capacity(len);
    let table = MortonTable::from_iterator((0..len).map(|_| {
        let pos = Point {
            x: rng.gen_range(0, 3900 * 2),
            y: rng.gen_range(0, 3900 * 2),
        };
        points.push(pos.clone());
        (pos, rng.next_u32())
    }));

    b.iter(|| {
        let i = rng.gen_range(0, points.len());
        let pos = &points[i];
        table.get_by_id(pos)
    });
}

#[bench]
fn bench_get_entities_in_range_dense_in_hashmap(b: &mut Bencher) {
    let mut rng = rand::thread_rng();

    let mut table = std::collections::HashMap::new();

    for _ in 0..(1 << 15) {
        let p = Point {
            x: rng.gen_range(0, 200 * 2),
            y: rng.gen_range(0, 200 * 2),
        };
        table.insert(p, EntityComponent(EntityId(rng.gen())));
    }

    let radius = 50;

    let mut v = Vec::with_capacity(512);
    b.iter(|| {
        let table = &table;
        let x = rng.gen_range(0, 200 * 2);
        let y = rng.gen_range(0, 200 * 2);
        v.clear();
        for x in x - radius..x + radius {
            for y in y - radius..y + radius {
                let p = Point { x, y };
                if let Some(val) = table.get(&p) {
                    v.push((p, val));
                }
            }
        }
        v.len()
    });
}

#[test]
fn morton_key_reconstruction_rand() {
    let mut rng = rand::thread_rng();

    for _ in 0..(1 << 12) {
        let x = rng.gen_range(0, 2000);
        let y = rng.gen_range(0, 2000);

        let morton = MortonKey::new(x, y);

        let res = morton.as_point();

        assert_eq!([x, y], res);
    }
}

#[bench]
fn bench_random_insert(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let mut table = MortonTable::<Point, usize>::new();

    for _ in 0..10_000 {
        let x = rng.gen_range(0, 29000);
        let y = rng.gen_range(0, 29000);
        let p = Point::new(x, y);

        table.insert(p, 420);
    }

    b.iter(|| {
        let x = rng.gen_range(0, 29000);
        let y = rng.gen_range(0, 29000);
        let p = Point::new(x, y);

        table.insert(p, 420)
    })
}
