#![feature(test)]
extern crate test;
use test::Bencher;

use std::sync::Arc;

mod common;
use common::test_location;

// The goal of these tests is to measure the performance of concurrently accessing a NetCDF file
// from several threads simultaneously.

#[bench]
fn conc_sequential_reads_same_fd(b: &mut Bencher) {
    let f = test_location().join("coads_climatology.nc");
    let f = netcdf::open(&f).unwrap();

    b.iter(|| {
        for _i in 0..500 {
            let c = vec![2usize, 30, 30];
            let n = c.iter().product::<usize>();

            let v = f.variable("SST").unwrap().unwrap();
            let mut val: Vec<f32> = vec![0.0; n];
            v.values_to(&mut val, None, Some(&c)).unwrap();
        }
    })
}

#[bench]
fn threaded_concurrent_reads_same_fd(b: &mut Bencher) {
    let f = test_location().join("coads_climatology.nc");
    let f = Arc::new(netcdf::open(&f).unwrap());

    let pool = rayon::ThreadPoolBuilder::new().num_threads(8).build().unwrap();
    b.iter(|| {
        let f = Arc::clone(&f);
        pool.scope(move |s| {

            for _i in 0..500 {
                let f = Arc::clone(&f);

                s.spawn(move |_| {
                    let c = vec![2usize, 30, 30];
                    let n = c.iter().product::<usize>();

                    let v = f.variable("SST").unwrap().unwrap();
                    let mut val: Vec<f32> = vec![0.0; n];
                    v.values_to(&mut val, None, Some(&c)).unwrap();
                });
            }
        })
    })
}

#[bench]
fn threaded_concurrent_reads_same_fd_many_threads(b: &mut Bencher) {
    let f = test_location().join("coads_climatology.nc");
    let f = Arc::new(netcdf::open(&f).unwrap());

    let pool = rayon::ThreadPoolBuilder::new().num_threads(200).build().unwrap();
    b.iter(|| {
        let f = Arc::clone(&f);
        pool.scope(move |s| {

            for _i in 0..500 {
                let f = Arc::clone(&f);

                s.spawn(move |_| {
                    let c = vec![2usize, 30, 30];
                    let n = c.iter().product::<usize>();

                    let v = f.variable("SST").unwrap().unwrap();
                    let mut val: Vec<f32> = vec![0.0; n];
                    v.values_to(&mut val, None, Some(&c)).unwrap();
                });
            }
        })
    })
}

#[bench]
fn threaded_concurrent_pool_fds(b: &mut Bencher) {
    let mut fdpool = Vec::new();
    for _j in 0..8 {
        let f = test_location().join("coads_climatology.nc");
        let f = netcdf::open(&f).unwrap();
        fdpool.push(f);
    }

    let fdpool = Arc::new(fdpool);

    let pool = rayon::ThreadPoolBuilder::new().num_threads(8).build().unwrap();
    b.iter(|| {
        let fdpool = Arc::clone(&fdpool);
        pool.scope(move |s| {

            for _i in 0..500 {
                let fdpool = Arc::clone(&fdpool);

                s.spawn(move |_| {
                    let n = rayon::current_thread_index().unwrap();
                    let f = &fdpool[n];

                    let c = vec![2usize, 30, 30];
                    let n = c.iter().product::<usize>();

                    let v = f.variable("SST").unwrap().unwrap();
                    let mut val: Vec<f32> = vec![0.0; n];
                    v.values_to(&mut val, None, Some(&c)).unwrap();
                });
            }
        })
    })
}

#[bench]
fn threaded_concurrent_pool_fds_many_threads(b: &mut Bencher) {
    let mut fdpool = Vec::new();
    for _j in 0..8 {
        let f = test_location().join("coads_climatology.nc");
        let f = netcdf::open(&f).unwrap();
        fdpool.push(f);
    }

    let fdpool = Arc::new(fdpool);

    let pool = rayon::ThreadPoolBuilder::new().num_threads(200).build().unwrap();
    b.iter(|| {
        let fdpool = Arc::clone(&fdpool);
        pool.scope(move |s| {

            for _i in 0..500 {
                let fdpool = Arc::clone(&fdpool);

                s.spawn(move |_| {
                    let n = rayon::current_thread_index().unwrap() % fdpool.len();
                    let f = &fdpool[n];

                    let c = vec![2usize, 30, 30];
                    let n = c.iter().product::<usize>();

                    let v = f.variable("SST").unwrap().unwrap();
                    let mut val: Vec<f32> = vec![0.0; n];
                    v.values_to(&mut val, None, Some(&c)).unwrap();
                });
            }
        })
    })
}
