use caolo_sim::model::components::EntityComponent;
use caolo_sim::model::geometry::{Circle, Point};
use caolo_sim::model::EntityId;
use caolo_sim::tables::{MortonTable, PositionTable};
use criterion::{criterion_group, Criterion};
use rand::Rng;
use rand::RngCore;

fn contains_rand_at_2pow16(c: &mut Criterion) {
    c.bench_function("morton contains rand 2^16", move |b| {
        let mut rng = rand::thread_rng();

        let table = MortonTable::from_iterator((0..(1 << 16)).map(|i| {
            let p = Point {
                x: rng.gen_range(0, 8000),
                y: rng.gen_range(0, 8000),
            };
            (p, i)
        }))
        .unwrap();
        b.iter(|| {
            let p = Point {
                x: rng.gen_range(0, 8000),
                y: rng.gen_range(0, 8000),
            };
            table.contains_key(&p)
        })
    });
}

fn get_entities_in_range_sparse(c: &mut Criterion) {
    c.bench_function("get_entities_in_range sparse", |b| {
        let mut rng = rand::thread_rng();

        let table = MortonTable::from_iterator((0..1 << 12).map(|_| {
            let p = Point {
                x: rng.gen_range(0, 3900 * 2),
                y: rng.gen_range(0, 3900 * 2),
            };
            (p, EntityComponent(EntityId(rng.gen())))
        }))
        .unwrap();

        let radius = 512;
        b.iter(|| {
            let table = &table;
            let p = Point {
                x: rng.gen_range(0, 3900 * 2),
                y: rng.gen_range(0, 3900 * 2),
            };
            table.get_entities_in_range(&Circle { center: p, radius })
        });
    });
}

fn get_entities_in_range_dense(c: &mut Criterion) {
    c.bench_function("get_entities_in_range dense", |b| {
        let mut rng = rand::thread_rng();

        let table = MortonTable::from_iterator((0..1 << 12).map(|_| {
            let p = Point {
                x: rng.gen_range(0, 200 * 2),
                y: rng.gen_range(0, 200 * 2),
            };
            (p, EntityComponent(EntityId(rng.gen())))
        }))
        .unwrap();

        let radius = 50;
        b.iter(|| {
            let table = &table;
            let p = Point {
                x: rng.gen_range(0, 200 * 2),
                y: rng.gen_range(0, 200 * 2),
            };
            table.get_entities_in_range(&Circle { center: p, radius })
        });
    });
}

fn make_morton_table(c: &mut Criterion) {
    c.bench_function("make morton table", |b| {
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
            }))
            .unwrap();
            table
        });
    });
}

fn rebuild_morton_table(c: &mut Criterion) {
    c.bench_function("rebuild_morton_table", |b| {
        let mut rng = rand::thread_rng();

        let mut table = MortonTable::with_capacity(1 << 15);

        b.iter(|| {
            table.clear();

            table
                .extend((0..(1 << 15)).map(|_| {
                    (
                        Point {
                            x: rng.gen_range(0, 3900 * 2),
                            y: rng.gen_range(0, 3900 * 2),
                        },
                        rng.next_u32(),
                    )
                }))
                .unwrap();
        });
    });
}

fn get_by_id_rand_2pow16(c: &mut Criterion) {
    c.bench_function("get_by_id random in 2^16", |b| {
        let mut rng = rand::thread_rng();

        let len = 1 << 16;
        let table = MortonTable::from_iterator((0..len).map(|_| {
            let pos = Point {
                x: rng.gen_range(0, 3900 * 2),
                y: rng.gen_range(0, 3900 * 2),
            };
            (pos, rng.next_u32())
        }))
        .unwrap();

        b.iter(|| {
            let pos = Point {
                x: rng.gen_range(0, 3900 * 2),
                y: rng.gen_range(0, 3900 * 2),
            };
            table.get_by_id(&pos)
        });
    });
}

fn get_by_id_in_table_rand_2pow16(c: &mut Criterion) {
    c.bench_function(
        "get_by_id 2^16 elements, all queried elements are in the table",
        |b| {
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
            }))
            .unwrap();

            b.iter(|| {
                let i = rng.gen_range(0, points.len());
                let pos = &points[i];
                table.get_by_id(pos)
            });
        },
    );
}

fn random_insert(c: &mut Criterion) {
    c.bench_function("random_insert", |b| {
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
        });
    });
}

criterion_group!(
    morton_benches,
    contains_rand_at_2pow16,
    get_entities_in_range_sparse,
    get_entities_in_range_dense,
    make_morton_table,
    random_insert,
    rebuild_morton_table,
    get_by_id_in_table_rand_2pow16,
    get_by_id_rand_2pow16,
    rebuild_morton_table,
);
