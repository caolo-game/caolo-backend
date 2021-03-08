use caolo_sim::indices::EntityId;
use caolo_sim::tables::{dense::DenseVecTable, Table};
use criterion::{black_box, criterion_group, BenchmarkId, Criterion};
use rand::seq::SliceRandom;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use rayon::iter::ParallelIterator;
use std::convert::TryFrom;

fn get_rand() -> impl rand::Rng {
    SmallRng::seed_from_u64(0xdeadbeef)
}

fn insert_at_random(c: &mut Criterion) {
    c.bench_function("vec_table insert_at_random", |b| {
        let mut rng = get_rand();
        let mut table = DenseVecTable::<EntityId, i32>::new();
        b.iter(|| {
            let id = rng.gen_range(0, 1 << 20);
            let id = EntityId(id);
            let res = table.insert_or_update(id, rng.gen_range(0, 200));
            debug_assert!(res);
            res
        });
    });
}

fn insert_at_random_w_reserve(c: &mut Criterion) {
    c.bench_function("vec_table insert_at_random_w_reserve", |b| {
        let mut rng = get_rand();
        let mut table = DenseVecTable::<EntityId, i32>::with_capacity(1 << 20);
        b.iter(|| {
            let id = rng.gen_range(0, 1 << 20);
            let id = EntityId(id);
            let res = table.insert_or_update(id, rng.gen_range(0, 200));
            debug_assert!(res);
            res
        });
    });
}

fn update_all_iter_2pow14_sparse(c: &mut Criterion) {
    c.bench_function("vec_table update_all_iter_2pow14_sparse", |b| {
        // The Id domain is 1.2 * LEN

        const LEN: usize = 1 << 14;
        let mut rng = get_rand();
        let mut table = DenseVecTable::<EntityId, usize>::with_capacity(LEN);
        for i in 0..LEN {
            let mut id = Default::default();
            while table.contains_id(id) {
                id = EntityId(rng.gen_range(
                    0,
                    u32::try_from(LEN * 6 / 5).expect("max len to fit into u32"),
                ));
            }
            table.insert_or_update(id, i);
        }
        b.iter(|| {
            table.iter_mut().for_each(|(_, val)| {
                *val += 8;
            });
        });
        black_box(table);
    });
}

fn update_all_iter_2pow14_dense(c: &mut Criterion) {
    c.bench_function("vec_table update_all_iter_2pow14_dense", |b| {
        // The whole table is filled

        const LEN: usize = 1 << 14;
        let mut table = DenseVecTable::<EntityId, usize>::with_capacity(LEN);
        for i in 0..LEN {
            let id = EntityId(i as u32);
            table.insert_or_update(id, i);
        }
        b.iter(|| {
            table.iter_mut().for_each(|(_, val)| {
                *val += 8;
            });
        });
        black_box(table);
    });
}

fn get_by_id_random_2_pow_16(c: &mut Criterion) {
    c.bench_function("vec_table get_by_id_random_2_pow_16", |b| {
        const LEN: usize = 1 << 16;
        let mut rng = get_rand();
        let mut table = DenseVecTable::<EntityId, usize>::with_capacity(LEN);
        let mut ids = Vec::with_capacity(LEN);
        for i in 0..LEN {
            let mut id = Default::default();
            while table.contains_id(id) {
                id = EntityId(
                    rng.gen_range(0, u32::try_from(LEN * 2).expect("max len to fit into u32")),
                );
            }
            table.insert_or_update(id, i);
            ids.push((id, i));
        }
        b.iter(|| {
            let ind = rng.gen_range(0, LEN);
            let (id, x) = ids[ind];
            let res = table.get_by_id(id);
            debug_assert_eq!(*res.expect("result to be found"), x);
            res
        });
    });
}

fn override_update_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec_table override_update_random");
    for size in (8..20).step_by(2) {
        let size = 1 << size;

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let mut rng = get_rand();
            let mut table = DenseVecTable::<EntityId, usize>::with_capacity(size);
            let mut ids = Vec::with_capacity(size);
            for i in 0..size {
                let mut id = Default::default();
                while table.contains_id(id) {
                    id = EntityId(
                        rng.gen_range(0, u32::try_from(size * 2).expect("max len to fit into u32")),
                    );
                }
                table.insert_or_update(id, i);
                ids.push((id, i));
            }
            b.iter(|| {
                let ind = rng.gen_range(0, size);
                let (id, x) = ids[ind];
                let res = table.insert_or_update(id, x * 2);
                debug_assert!(res);
                res
            });
        });
    }
    group.finish();
}

fn override_update_all_serial(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec_table override_update_all_serial");
    for size in (8..20).step_by(2) {
        let size = 1 << size;

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let mut rng = get_rand();
            let mut table = DenseVecTable::<EntityId, usize>::with_capacity(size);
            let mut ids = Vec::with_capacity(size);
            for i in 0..size {
                let mut id = Default::default();
                while table.contains_id(id) {
                    id = EntityId(
                        rng.gen_range(0, u32::try_from(size * 2).expect("max len to fit into u32")),
                    );
                }
                table.insert_or_update(id, i);
                ids.push((id, i));
            }
            b.iter(|| {
                table.iter_mut().for_each(|(_, v)| *v += 1);
            });
        });
    }
    group.finish();
}


fn override_update_all_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec_table override_update_all_parallel");
    for size in (8..20).step_by(2) {
        let size = 1 << size;

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let mut rng = get_rand();
            let mut table = DenseVecTable::<EntityId, usize>::with_capacity(size);
            let mut ids = Vec::with_capacity(size);
            for i in 0..size {
                let mut id = Default::default();
                while table.contains_id(id) {
                    id = EntityId(
                        rng.gen_range(0, u32::try_from(size * 2).expect("max len to fit into u32")),
                    );
                }
                table.insert_or_update(id, i);
                ids.push((id, i));
            }
            b.iter(|| {
                table.par_iter_mut().for_each(|(_, v)| *v += 1);
            });
        });
    }
    group.finish();
}

fn delete_by_id_random(c: &mut Criterion) {
    c.bench_function("vec_table delete_by_id_random", |b| {
        let mut rng = get_rand();
        let mut table = DenseVecTable::<EntityId, i32>::new();
        let mut ids = Vec::with_capacity(1 << 15);
        for i in 0..1 << 15 {
            let mut res = false;
            let mut id = Default::default();
            while !res {
                id = EntityId(rng.gen_range(0, 1 << 25));
                res = table.insert_or_update(id, i);
            }
            ids.push(id);
        }
        ids.as_mut_slice().shuffle(&mut rng);
        let mut i = 0;
        let mask = (1 << 15) - 1;
        b.iter(|| {
            i = (i + 1) & mask;
            let id = ids[i];
            let res = table.delete(id);
            debug_assert!(res.is_some());
            table.insert_or_update(id, 123);
            res
        });
    });
}

criterion_group!(
    vec_benches,
    insert_at_random,
    update_all_iter_2pow14_sparse,
    update_all_iter_2pow14_dense,
    get_by_id_random_2_pow_16,
    override_update_random,
    override_update_all_serial,
    override_update_all_parallel,
    delete_by_id_random,
    insert_at_random_w_reserve
);
