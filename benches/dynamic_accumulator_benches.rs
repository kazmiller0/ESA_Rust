use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use esa_rust::acc::dynamic_accumulator::{DynamicAccumulator, QueryResult};
use rand::Rng;

fn setup_accumulator(size: usize) -> DynamicAccumulator {
    let mut acc = DynamicAccumulator::new();
    let elements: Vec<i64> = (0..size as i64).collect();
    acc.add_batch(&elements).unwrap();
    acc
}

fn bench_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("Dynamic Accumulator Add");
    for size in [10, 20, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("add", size), size, |b, &size| {
            b.iter_with_setup(
                || {
                    let acc = setup_accumulator(size);
                    let new_element = rand::thread_rng().gen::<i64>();
                    (acc, new_element)
                },
                |(mut acc, new_element)| acc.add(&new_element),
            );
        });
    }
    group.finish();
}

fn bench_delete(c: &mut Criterion) {
    let mut group = c.benchmark_group("Dynamic Accumulator Delete");
    for size in [10, 20, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("delete", size), size, |b, &size| {
            b.iter_with_setup(
                || {
                    let acc = setup_accumulator(size);
                    let element_to_delete = (size as i64) / 2;
                    (acc, element_to_delete)
                },
                |(mut acc, element_to_delete)| acc.delete(&element_to_delete),
            );
        });
    }
    group.finish();
}

fn bench_prove_membership(c: &mut Criterion) {
    let mut group = c.benchmark_group("Dynamic Accumulator Prove Membership");
    for size in [10, 20, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("prove_membership", size),
            size,
            |b, &size| {
                let acc = setup_accumulator(size);
                let existing_element = (size as i64) / 2;
                b.iter(|| acc.prove_membership(&existing_element));
            },
        );
    }
    group.finish();
}

fn bench_verify_membership(c: &mut Criterion) {
    let mut group = c.benchmark_group("Dynamic Accumulator Verify Membership");
    for size in [10, 20, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("verify_membership", size),
            size,
            |b, &size| {
                let acc = setup_accumulator(size);
                let existing_element = (size as i64) / 2;
                let proof = acc.prove_membership(&existing_element).unwrap();
                b.iter(|| acc.verify_membership(&proof));
            },
        );
    }
    group.finish();
}

fn bench_prove_non_membership(c: &mut Criterion) {
    let mut group = c.benchmark_group("Dynamic Accumulator Prove Non-Membership");
    for size in [10, 20, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("prove_non_membership", size),
            size,
            |b, &size| {
                let acc = setup_accumulator(size);
                let non_existing_element = size as i64 + 1;
                b.iter(|| acc.prove_non_membership(&non_existing_element));
            },
        );
    }
    group.finish();
}

fn bench_verify_non_membership(c: &mut Criterion) {
    let mut group = c.benchmark_group("Dynamic Accumulator Verify Non-Membership");
    for size in [10, 20, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("verify_non_membership", size),
            size,
            |b, &size| {
                let acc = setup_accumulator(size);
                let non_existing_element = size as i64 + 1;
                let proof = acc.prove_non_membership(&non_existing_element).unwrap();
                b.iter(|| acc.verify_non_membership(&proof));
            },
        );
    }
    group.finish();
}


fn bench_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("Dynamic Accumulator Operations");
    for size in [10, 20, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("query", size), size, |b, &size| {
            let acc = setup_accumulator(size);
            let existing_element = (size as i64) / 2;
            b.iter(|| acc.query(&existing_element));
        });
    }
    group.finish();
}

fn bench_prove_intersection(c: &mut Criterion) {
    let mut group = c.benchmark_group("Dynamic Accumulator Prove Intersection");
    for size in [10, 20, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("prove_intersection", size), size, |b, &size| {
            let acc1 = setup_accumulator(size);
            let mut acc2 = DynamicAccumulator::new();
            let elements: Vec<i64> = (size as i64 / 2..size as i64 + size as i64 / 2).collect();
            acc2.add_batch(&elements).unwrap();
            b.iter(|| acc1.prove_intersection(&acc2));
        });
    }
    group.finish();
}

fn bench_verify_intersection(c: &mut Criterion) {
    let mut group = c.benchmark_group("Dynamic Accumulator Verify Intersection");
    for size in [10, 20, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("verify_intersection", size),
            size,
            |b, &size| {
                let acc1 = setup_accumulator(size);
                let mut acc2 = DynamicAccumulator::new();
                let elements: Vec<i64> =
                    (size as i64 / 2..size as i64 + size as i64 / 2).collect();
                acc2.add_batch(&elements).unwrap();
                let (intersection_acc, proof) = acc1.prove_intersection(&acc2).unwrap();
                b.iter(|| {
                    DynamicAccumulator::verify_intersection(
                        acc1.acc_value,
                        acc2.acc_value,
                        intersection_acc.acc_value,
                        &proof,
                    )
                });
            },
        );
    }
    group.finish();
}

fn bench_prove_union(c: &mut Criterion) {
    let mut group = c.benchmark_group("Dynamic Accumulator Prove Union");
    for size in [10, 20, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("prove_union", size), size, |b, &size| {
            let acc1 = setup_accumulator(size);
            let mut acc2 = DynamicAccumulator::new();
            let elements: Vec<i64> = (size as i64 / 2..size as i64 + size as i64 / 2).collect();
            acc2.add_batch(&elements).unwrap();
            b.iter(|| acc1.prove_union(&acc2));
        });
    }
    group.finish();
}

fn bench_verify_union(c: &mut Criterion) {
    let mut group = c.benchmark_group("Dynamic Accumulator Verify Union");
    for size in [10, 20, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("verify_union", size), size, |b, &size| {
            let acc1 = setup_accumulator(size);
            let mut acc2 = DynamicAccumulator::new();
            let elements: Vec<i64> = (size as i64 / 2..size as i64 + size as i64 / 2).collect();
            acc2.add_batch(&elements).unwrap();
            let (union_acc, proof) = acc1.prove_union(&acc2).unwrap();
            b.iter(|| {
                DynamicAccumulator::verify_union(acc1.acc_value, acc2.acc_value, union_acc.acc_value, &proof)
            });
        });
    }
    group.finish();
}


criterion_group!(
    benches,
    bench_add,
    bench_delete,
    bench_prove_membership,
    bench_verify_membership,
    bench_prove_non_membership,
    bench_verify_non_membership,
    bench_query,
    bench_prove_intersection,
    bench_verify_intersection,
    bench_prove_union,
    bench_verify_union
);
criterion_main!(benches);
