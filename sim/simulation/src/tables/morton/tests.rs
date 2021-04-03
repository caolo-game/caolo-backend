use super::*;
use crate::geometry::Axial;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};

#[test]
fn merge_simple() {
    let mut lhs = MortonTable::from_iterator((0..8).map(|i| (Axial::new(42, i), 1))).unwrap();
    let rhs = MortonTable::from_iterator((0..16).map(|i| (Axial::new(42, i), 2))).unwrap();

    lhs.merge(&rhs, |_, l, r| l + r).unwrap();

    for i in 0..8 {
        let j = lhs.at(Axial::new(42, i)).unwrap();
        assert_eq!(*j, 3);
    }
    for i in 8..16 {
        let j = lhs.at(Axial::new(42, i)).unwrap();
        assert_eq!(*j, 2);
    }
}

#[test]
fn aabb_simple() {
    let points = [
        Axial::new(12, 50),
        Axial::new(8, 1),
        Axial::new(20, 32),
        Axial::new(23, 12),
    ];

    let table = MortonTable::from_iterator(points.iter().cloned().map(|p| (p, 1))).unwrap();
    let [min, max] = table.aabb().unwrap();

    let min = min.as_array();
    let max = max.as_array();

    let [q, r] = min;
    assert!(q <= 8);
    assert!(r <= 1);

    let [q, r] = max;
    assert!(23 <= q);
    assert!(50 <= r);
}

#[test]
fn simple_from_iterator() {
    let mut rng = rand::thread_rng();
    let mut points = [
        Axial::new(1, 23),
        Axial::new(2, 42),
        Axial::new(1 << 15 - 1, 23),
        Axial::new(1, 1 << 14 - 2),
        Axial::new(rng.gen_range(0, 1 << 15), rng.gen_range(0, 1 << 15)),
        Axial::new(rng.gen_range(0, 1 << 15), rng.gen_range(0, 1 << 15)),
        Axial::new(rng.gen_range(0, 1 << 15), rng.gen_range(0, 1 << 15)),
        Axial::new(rng.gen_range(0, 1 << 15), rng.gen_range(0, 1 << 15)),
        Axial::new(rng.gen_range(0, 1 << 15), rng.gen_range(0, 1 << 15)),
        Axial::new(rng.gen_range(0, 1 << 15), rng.gen_range(0, 1 << 15)),
        Axial::new(rng.gen_range(0, 1 << 15), rng.gen_range(0, 1 << 15)),
        Axial::new(rng.gen_range(0, 1 << 15), rng.gen_range(0, 1 << 15)),
        Axial::new(rng.gen_range(0, 1 << 15), rng.gen_range(0, 1 << 15)),
        Axial::new(rng.gen_range(0, 1 << 15), rng.gen_range(0, 1 << 15)),
        Axial::new(rng.gen_range(0, 1 << 15), rng.gen_range(0, 1 << 15)),
        Axial::new(rng.gen_range(0, 1 << 15), rng.gen_range(0, 1 << 15)),
    ];
    points.shuffle(&mut rng);
    MortonTable::from_iterator(points.iter().enumerate().map(|(i, p)| (*p, i))).unwrap();
}

#[test]
fn insertions() {
    let mut table = MortonTable::new();

    table.insert(Axial::new(16, 32), 123i32).unwrap();
}

fn test_range_query_all_by_rng(rng: &mut impl rand::Rng) {
    let center = Axial::new(64, 64);

    let points = (0..256)
        .map(|i| {
            let p = Axial {
                q: rng.gen_range(-64, 64),
                r: rng.gen_range(-64, 64),
            } + center;
            (p, i)
        })
        .collect::<HashSet<_>>();

    let table = MortonTable::from_iterator(points.iter().cloned()).unwrap();

    let mut res = Vec::new();
    table.find_by_range(
        center,
        Axial::new(0, 0).hex_distance(center) as u32 + 1,
        &mut res,
    );

    let res = res
        .into_iter()
        .map(|(p, i)| (p, *i))
        .collect::<HashSet<_>>();
    let exp = points;
    let reslen = res.len();

    let diff = res.symmetric_difference(&exp).collect::<Vec<_>>();

    assert!(
        diff.is_empty(),
        "Result did not contain the expected values. Res len: {} Exp len: {} Difference:\n{:#?}",
        reslen,
        exp.len(),
        diff
    );
}

#[test]
fn regression_query_all_duplicated_items_bug() {
    let seed = [
        26, 84, 47, 127, 109, 136, 2, 23, 141, 90, 81, 183, 80, 116, 43, 39,
    ];

    let mut rng = rand::rngs::SmallRng::from_seed(seed);

    test_range_query_all_by_rng(&mut rng);
}

#[test]
fn test_range_query_all() {
    for _ in 0..16 {
        let mut rng = rand::thread_rng();
        let mut seed = [0; 16];
        rng.fill_bytes(&mut seed);

        dbg!(&seed);
        let mut rng = rand::rngs::SmallRng::from_seed(seed);

        test_range_query_all_by_rng(&mut rng);
    }
}
#[test]
fn regression_at_bug1() {
    let points = [
        Axial { q: 3, r: 10 },
        Axial { q: 5, r: 11 },
        Axial { q: 63, r: 5 },
        Axial { q: 50, r: 8 },
        Axial { q: 63, r: 9 },
        Axial { q: 39, r: 25 },
        Axial { q: 53, r: 27 },
        Axial { q: 14, r: 37 },
        Axial { q: 0, r: 46 },
        Axial { q: 1, r: 61 },
        Axial { q: 30, r: 53 },
        Axial { q: 36, r: 39 },
        Axial { q: 46, r: 32 },
        Axial { q: 58, r: 38 },
        Axial { q: 38, r: 59 },
        Axial { q: 54, r: 49 },
        Axial { q: 82, r: 4 },
        Axial { q: 84, r: 14 },
        Axial { q: 74, r: 20 },
        Axial { q: 77, r: 30 },
        Axial { q: 83, r: 23 },
        Axial { q: 112, r: 11 },
        Axial { q: 99, r: 18 },
        Axial { q: 115, r: 29 },
        Axial { q: 70, r: 37 },
        Axial { q: 64, r: 40 },
        Axial { q: 82, r: 32 },
        Axial { q: 86, r: 36 },
        Axial { q: 70, r: 53 },
        Axial { q: 99, r: 35 },
        Axial { q: 97, r: 43 },
        Axial { q: 108, r: 42 },
        Axial { q: 107, r: 62 },
        Axial { q: 122, r: 63 },
        Axial { q: 17, r: 67 },
        Axial { q: 29, r: 66 },
        Axial { q: 10, r: 89 },
        Axial { q: 31, r: 94 },
        Axial { q: 42, r: 75 },
        Axial { q: 49, r: 64 },
        Axial { q: 62, r: 66 },
        Axial { q: 33, r: 90 },
        Axial { q: 59, r: 82 },
        Axial { q: 60, r: 85 },
        Axial { q: 53, r: 93 },
        Axial { q: 16, r: 105 },
        Axial { q: 55, r: 109 },
        Axial { q: 38, r: 121 },
        Axial { q: 41, r: 127 },
        Axial { q: 73, r: 70 },
        Axial { q: 75, r: 70 }, // this is the ficked fucked fuckery
        Axial { q: 65, r: 78 },
        Axial { q: 76, r: 73 },
        Axial { q: 95, r: 65 },
        Axial { q: 92, r: 69 },
        Axial { q: 87, r: 75 },
        Axial { q: 117, r: 69 },
        Axial { q: 98, r: 84 },
        Axial { q: 120, r: 83 },
        Axial { q: 88, r: 97 },
        Axial { q: 99, r: 118 },
        Axial { q: 110, r: 126 },
        Axial { q: 126, r: 122 },
    ];
    let points: Vec<(_, _)> = points
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, p)| (p, i))
        .collect();

    let table = MortonTable::<usize>::from_iterator(points.iter().cloned()).unwrap();

    dbg!(&table);

    for p in points {
        let found = table.at(p.0);
        let key = MortonKey::new(p.0.q as u16, p.0.r as u16);
        assert_eq!(found, Some(&p.1), "{:?} {:?}", p.0, key);
    }
}

#[test]
fn at_few_items() {
    let mut rng = rand::thread_rng();

    let mut points = HashSet::with_capacity(64);

    for i in 0..16 {
        points.clear();

        for _ in 0..i {
            let p = Axial {
                q: rng.gen_range(0, 128),
                r: rng.gen_range(0, 128),
            };
            let i = 1000 * p.q + p.r;
            points.insert((p, i as usize));
        }
        let table =
            MortonTable::<usize>::from_iterator(points.iter().cloned()).expect("table build");

        println!("{:?}\n{:?}", table.skiplist, table.keys);

        for p in points.iter() {
            let found = table.at(p.0);
            let key = MortonKey::new(p.0.q as u16, p.0.r as u16);
            assert_eq!(found, Some(&p.1), "{:?} {:?}", p.0, key);
        }
    }
}

#[test]
fn at() {
    let mut rng = rand::thread_rng();

    let mut points = HashSet::with_capacity(64);

    for _ in 0..64 {
        let p = Axial {
            q: rng.gen_range(0, 128),
            r: rng.gen_range(0, 128),
        };
        let i = 1000 * p.q + p.r;
        points.insert((p, i as usize));
    }

    let table = MortonTable::<usize>::from_iterator(points.iter().cloned()).expect("table build");

    println!("{:?}\n{:?}", table.skiplist, table.keys);

    for p in points {
        let found = table.at(p.0);
        let key = MortonKey::new(p.0.q as u16, p.0.r as u16);
        assert_eq!(found, Some(&p.1), "{:?} {:?}", p.0, key);
    }
}

#[test]
fn morton_key_reconstruction_rand() {
    let mut rng = rand::thread_rng();

    for _ in 0..(1 << 12) {
        let q = rng.gen_range(0, 2000);
        let r = rng.gen_range(0, 2000);

        let morton = MortonKey::new(q, r);

        let res = morton.as_point();

        assert_eq!([q, r], res);
    }
}

#[test]
fn from_iterator_inserts_correctly() {
    let mut rng = rand::thread_rng();

    let len = 1 << 12;
    let mut points = HashMap::with_capacity(len);
    let table = MortonTable::from_iterator((0..len).filter_map(|_| {
        let pos = Axial {
            q: rng.gen_range(0, 3900 * 2),
            r: rng.gen_range(0, 3900 * 2),
        };
        if !points.contains_key(&pos) {
            return None;
        }
        let val = rng.next_u32();
        points.insert(pos.clone(), val);
        Some((pos, val))
    }))
    .unwrap();

    for (pos, val) in points {
        let v = *table.at(pos).expect("to find the value");
        assert_eq!(val, v);
    }
}

#[test]
fn dedupe_simple() {
    let mut rng = rand::thread_rng();

    let mut table = MortonTable::from_iterator((0..128).flat_map(|_| {
        let pos = Axial {
            q: rng.gen_range(0, 1 << 15),
            r: rng.gen_range(0, 1 << 15),
        };
        vec![(pos, 0), (pos, 1), (pos, 3)]
    }))
    .unwrap();
    table.dedupe();

    let mut cnt = 0;

    let positions = table
        .iter()
        .map(|(p, _)| {
            cnt += 1;
            p
        })
        .collect::<HashSet<_>>();
    assert_eq!(positions.len(), 128);
    assert_eq!(cnt, 128);
}
