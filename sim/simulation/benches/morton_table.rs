use caolo_sim::components::EntityComponent;
use caolo_sim::geometry::Axial;
use caolo_sim::indices::EntityId;
use caolo_sim::tables::morton::MortonTable;
use criterion::{black_box, criterion_group, BenchmarkId, Criterion};
use rand::RngCore;
use rand::{rngs::SmallRng, Rng, SeedableRng};

fn get_rand() -> impl rand::Rng {
    SmallRng::seed_from_u64(0xdeadbeef)
}

fn contains_rand(c: &mut Criterion) {
    let mut group = c.benchmark_group("morton table contains_rand");
    for size in 8..14 {
        let size = 1 << size;
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, move |b, &size| {
            let mut rng = get_rand();

            let table = MortonTable::from_iterator((0..size).map(|i| {
                let p = Axial {
                    q: rng.gen_range(0, 8000),
                    r: rng.gen_range(0, 8000),
                };
                (p, i)
            }))
            .unwrap();

            b.iter(|| {
                let p = Axial {
                    q: rng.gen_range(0, 8000),
                    r: rng.gen_range(0, 8000),
                };
                table.contains_key(p)
            })
        });
    }
    group.finish();
}

fn get_entities_in_range_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("morton table get_entities_in_range sparse");
    for size in 8..14 {
        let size = 1 << size;
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let mut rng = get_rand();

            let table = MortonTable::from_iterator((0..size).map(|_| {
                let p = Axial {
                    q: rng.gen_range(0, 3900 * 2),
                    r: rng.gen_range(0, 3900 * 2),
                };
                (p, EntityComponent(EntityId(rng.gen())))
            }))
            .unwrap();

            let radius = 512;
            b.iter(|| {
                let table = &table;
                let p = Axial {
                    q: rng.gen_range(0, 3900 * 2),
                    r: rng.gen_range(0, 3900 * 2),
                };
                let mut entities = Vec::with_capacity(512 * 512);
                table.find_by_range(p, radius, &mut entities);
                entities
            });
        });
    }
    group.finish();
}

fn get_entities_in_range_dense(c: &mut Criterion) {
    let mut group = c.benchmark_group("morton table get_entities_in_range dense");
    for size in 8..14 {
        let size = 1 << size;
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let mut rng = get_rand();

            let table = MortonTable::from_iterator((0..size).map(|_| {
                let p = Axial {
                    q: rng.gen_range(0, 200 * 2),
                    r: rng.gen_range(0, 200 * 2),
                };
                (p, EntityComponent(EntityId(rng.gen())))
            }))
            .unwrap();

            let radius = 50;
            b.iter(|| {
                let table = &table;
                let p = Axial {
                    q: rng.gen_range(0, 200 * 2),
                    r: rng.gen_range(0, 200 * 2),
                };
                let mut entities = Vec::with_capacity(50 * 50);
                table.find_by_range(p, radius, &mut entities);
                entities
            });
        });
    }
    group.finish();
}

fn make_morton_table(c: &mut Criterion) {
    let mut group = c.benchmark_group("morton table make_morton_table");
    for size in 8..14 {
        let size = 1 << size;
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let mut rng = get_rand();

            b.iter(|| {
                let table = MortonTable::from_iterator((0..size).map(|_| {
                    (
                        Axial {
                            q: rng.gen_range(0, 3900 * 2),
                            r: rng.gen_range(0, 3900 * 2),
                        },
                        rng.next_u32(),
                    )
                }))
                .unwrap();
                table
            });
        });
    }
    group.finish();
}

fn rebuild_morton_table(c: &mut Criterion) {
    let mut group = c.benchmark_group("morton table rebuild_morton_table");
    for size in 8..14 {
        let size = 1 << size;

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let mut rng = get_rand();

            let mut table = MortonTable::with_capacity(size);

            b.iter(|| {
                table.clear();

                table
                    .extend((0..size).map(|_| {
                        (
                            Axial {
                                q: rng.gen_range(0, 3900 * 2),
                                r: rng.gen_range(0, 3900 * 2),
                            },
                            rng.next_u32(),
                        )
                    }))
                    .unwrap();
            });
        });
    }
    group.finish();
}

fn at_rand(c: &mut Criterion) {
    let mut group = c.benchmark_group("morton table at random");
    for size in 8..14 {
        let size = 1 << size;
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &len| {
            let mut rng = get_rand();

            let table = MortonTable::from_iterator((0..len).map(|_| {
                let pos = Axial {
                    q: rng.gen_range(0, 3900 * 2),
                    r: rng.gen_range(0, 3900 * 2),
                };
                (pos, rng.next_u32())
            }))
            .unwrap();

            b.iter(|| {
                let pos = Axial {
                    q: rng.gen_range(0, 3900 * 2),
                    r: rng.gen_range(0, 3900 * 2),
                };
                table.at(pos)
            });
        });
    }
    group.finish();
}

fn at_in_table_rand(c: &mut Criterion) {
    let mut group = c.benchmark_group("morton table at, all queried elements are in the table");
    for size in 8..14 {
        let size = 1 << size;
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &len| {
            let mut rng = get_rand();

            let mut points = Vec::with_capacity(len);
            let table = MortonTable::from_iterator((0..len).map(|_| {
                let pos = Axial {
                    q: rng.gen_range(0, 3900 * 2),
                    r: rng.gen_range(0, 3900 * 2),
                };
                points.push(pos.clone());
                (pos, rng.next_u32())
            }))
            .unwrap();

            b.iter(|| {
                let i = rng.gen_range(0, points.len());
                let pos = &points[i];
                table.at(*pos)
            });
        });
    }
    group.finish();
}

fn random_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("morton table random_insert");
    for size in 8..12 {
        let size = 1 << size;
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let mut rng = get_rand();
            let mut table = MortonTable::<usize>::new();

            for _ in 0..size {
                let q = rng.gen_range(0, 29000);
                let r = rng.gen_range(0, 29000);
                let p = Axial::new(q, r);

                table.insert(p, 420).unwrap();
            }

            b.iter(|| {
                let q = rng.gen_range(0, 29000);
                let r = rng.gen_range(0, 29000);
                let p = Axial::new(q, r);

                table.insert(p, 420).unwrap()
            });
        });
    }
    group.finish();
}

fn random_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("morton table random_update");
    for size in 8..13 {
        let size = 1 << size;
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let mut rng = get_rand();
            let mut table = MortonTable::<u32>::new();
            let mut memory = Vec::new();

            for _ in 0..size {
                let q = rng.gen_range(0, 29000);
                let r = rng.gen_range(0, 29000);
                let p = Axial::new(q, r);
                memory.push(p);

                table.insert(p, 420).unwrap();
            }

            let memory = &memory;
            let table = &mut table;

            b.iter(move || {
                let i = rng.gen_range(0, memory.len());
                let p = memory[i].clone();
                let mut updated = table.update(p, rng.next_u32());
                black_box(&mut updated);
                debug_assert!(updated.is_some());
            });
        });
    }
    group.finish();
}

criterion_group!(
    morton_benches,
    contains_rand,
    get_entities_in_range_sparse,
    get_entities_in_range_dense,
    make_morton_table,
    random_insert,
    rebuild_morton_table,
    at_in_table_rand,
    at_rand,
    random_update,
);
