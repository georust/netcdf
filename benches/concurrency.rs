use criterion::{criterion_group, criterion_main, Criterion};
use std::path::Path;
use std::sync::Arc;

#[path = "../tests/common/mod.rs"]
mod common;
use common::test_location;

const OVERSUBSCRIBE_THREADS: usize = 50;
const ITERATIONS: usize = 300;
const REPETITIONS: usize = 100;

fn conc_sequential_reads_same_fd(path: &Path) {
    let f = netcdf::open(path).unwrap();
    for _ in 0..ITERATIONS {
        let c = [2_usize, 30, 30];
        let n = c.iter().product::<usize>();

        for _ in 0..REPETITIONS {
            let v = f.variable("SST").unwrap().unwrap();
            let mut val: Vec<f32> = vec![0.0; n];
            v.values_to(&mut val, Some(&[0, 0, 0]), Some(&c)).unwrap();
        }
    }
}

fn threaded_concurrent_reads_same_fd(path: &Path) {
    let f = Arc::new(netcdf::open(path).unwrap());

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(8)
        .build()
        .unwrap();

    let f = Arc::clone(&f);
    pool.scope(move |s| {
        for _ in 0..ITERATIONS {
            let f = Arc::clone(&f);

            s.spawn(move |_| {
                let c = [2_usize, 30, 30];
                let n = c.iter().product::<usize>();

                let v = f.variable("SST").unwrap().unwrap();
                let mut val: Vec<f32> = vec![0.0; n];
                for _ in 0..REPETITIONS {
                    v.values_to(&mut val, Some(&[0, 0, 0]), Some(&c)).unwrap();
                }
            });
        }
    })
}

fn threaded_concurrent_reads_same_fd_many_threads(path: &Path) {
    let f = Arc::new(netcdf::open(path).unwrap());

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(OVERSUBSCRIBE_THREADS)
        .build()
        .unwrap();
    let f = Arc::clone(&f);
    pool.scope(move |s| {
        for _i in 0..ITERATIONS {
            let f = Arc::clone(&f);

            s.spawn(move |_| {
                let c = [2_usize, 30, 30];
                let n = c.iter().product::<usize>();

                let v = f.variable("SST").unwrap().unwrap();
                let mut val: Vec<f32> = vec![0.0; n];
                for _ in 0..REPETITIONS {
                    v.values_to(&mut val, Some(&[0, 0, 0]), Some(&c)).unwrap();
                }
            });
        }
    })
}

fn threaded_concurrent_pool_fds(path: &Path) {
    let mut fdpool = Vec::new();
    for _ in 0..8 {
        let f = netcdf::open(path).unwrap();
        fdpool.push(f);
    }

    let fdpool = Arc::new(fdpool);

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(8)
        .build()
        .unwrap();
    let fdpool = Arc::clone(&fdpool);
    pool.scope(move |s| {
        for _i in 0..ITERATIONS {
            let fdpool = Arc::clone(&fdpool);

            s.spawn(move |_| {
                let n = rayon::current_thread_index().unwrap();
                let f = &fdpool[n];

                let c = [2_usize, 30, 30];
                let n = c.iter().product::<usize>();

                let v = f.variable("SST").unwrap().unwrap();
                let mut val: Vec<f32> = vec![0.0; n];
                for _ in 0..REPETITIONS {
                    v.values_to(&mut val, Some(&[0, 0, 0]), Some(&c)).unwrap();
                }
            });
        }
    })
}

fn threaded_concurrent_pool_fds_many_threads(path: &Path) {
    let mut fdpool = Vec::new();
    for _ in 0..8 {
        let f = netcdf::open(path).unwrap();
        fdpool.push(f);
    }

    let fdpool = Arc::new(fdpool);

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(OVERSUBSCRIBE_THREADS)
        .build()
        .unwrap();
    let fdpool = Arc::clone(&fdpool);
    pool.scope(move |s| {
        for _i in 0..ITERATIONS {
            let fdpool = Arc::clone(&fdpool);

            s.spawn(move |_| {
                let n = rayon::current_thread_index().unwrap() % fdpool.len();
                let f = &fdpool[n];

                let c = [2_usize, 30, 30];
                let n = c.iter().product::<usize>();

                let v = f.variable("SST").unwrap().unwrap();
                let mut val: Vec<f32> = vec![0.0; n];
                for _ in 0..REPETITIONS {
                    v.values_to(&mut val, Some(&[0, 0, 0]), Some(&c)).unwrap();
                }
            });
        }
    })
}

/// The goal of these tests is to measure the performance of concurrently accessing a `netCDF` file
/// from several threads simultaneously.
fn concurrency_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrency");
    group.sample_size(10);

    let path = test_location().join("coads_climatology.nc");

    group.bench_function("conc_sequential_reads_same_fd", |b| {
        b.iter(|| {
            conc_sequential_reads_same_fd(&path);
        })
    });
    group.bench_function("threaded_concurrent_reads_same_fd", |b| {
        b.iter(|| {
            threaded_concurrent_reads_same_fd(&path);
        })
    });
    group.bench_function("threaded_concurrent_reads_same_fd_many_threads", |b| {
        b.iter(|| {
            threaded_concurrent_reads_same_fd_many_threads(&path);
        })
    });
    group.bench_function("threaded_concurrent_pool_fds", |b| {
        b.iter(|| {
            threaded_concurrent_pool_fds(&path);
        })
    });
    group.bench_function("threaded_concurrent_pool_fds_many_threads", |b| {
        b.iter(|| {
            threaded_concurrent_pool_fds_many_threads(&path);
        })
    });

    group.finish();
}

criterion_group!(concurrency, concurrency_benchmark);
criterion_main!(concurrency);
