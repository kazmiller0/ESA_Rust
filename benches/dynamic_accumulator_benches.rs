//! Benchmarks for the dynamic accumulator operations.
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use esa_rust::acc::dynamic_accumulator::DynamicAccumulator;

fn setup_accumulator(size: u64) -> DynamicAccumulator {
    let mut acc = DynamicAccumulator::new();
    for i in 0..size {
        acc.add(&(i as i64));
    }
    acc
}

fn bench_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("DynamicAccumulator: Add");

    for size in [10, 100, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut acc = setup_accumulator(size as u64);
            let new_element = (size + 1) as i64;
            b.iter(|| {
                // We clone the accumulator for each iteration to avoid timing the setup
                // and to ensure each 'add' is independent.
                let mut acc_clone = acc.clone();
                acc_clone.add(black_box(&new_element));
            });
        });
    }
    group.finish();
}

fn bench_delete(c: &mut Criterion) {
    let mut group = c.benchmark_group("DynamicAccumulator: Delete");

    for size in [10, 100, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let acc = setup_accumulator(size as u64);
            let element_to_delete = (size / 2) as i64;
            b.iter(|| {
                let mut acc_clone = acc.clone();
                acc_clone.delete(black_box(&element_to_delete)).unwrap();
            });
        });
    }
    group.finish();
}

fn bench_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("DynamicAccumulator: Update");

    for size in [10, 100, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let acc = setup_accumulator(size as u64);
            let old_element = (size / 2) as i64;
            let new_element = (size + 1) as i64;
            b.iter(|| {
                let mut acc_clone = acc.clone();
                acc_clone
                    .update(black_box(&old_element), black_box(&new_element))
                    .unwrap();
            });
        });
    }
    group.finish();
}

fn bench_prove_membership(c: &mut Criterion) {
    let mut group = c.benchmark_group("DynamicAccumulator: Prove Membership");

    for size in [10, 100, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let acc = setup_accumulator(size as u64);
            let element_to_prove = (size / 2) as i64;
            b.iter(|| {
                acc.prove_membership(black_box(&element_to_prove)).unwrap();
            });
        });
    }
    group.finish();
}

fn bench_prove_non_membership(c: &mut Criterion) {
    let mut group = c.benchmark_group("DynamicAccumulator: Prove Non-Membership");

    // Using smaller sizes because this operation is expensive.
    for size in [10, 50, 100, 150, 200].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let acc = setup_accumulator(size as u64);
            let element_to_prove = (size + 1) as i64; // An element not in the set
            b.iter(|| {
                acc.prove_non_membership(black_box(&element_to_prove))
                    .unwrap();
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_add,
    bench_delete,
    bench_update,
    bench_prove_membership,
    bench_prove_non_membership
);
criterion_main!(benches);
