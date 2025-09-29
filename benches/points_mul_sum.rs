use ark_bls12_381::{Fr, G1Projective};
use ark_ff::{PrimeField, UniformRand};
use ark_ec::ProjectiveCurve;
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use rand::thread_rng;

fn points_mul_sum(points: &[G1Projective], scalars: &[Fr]) -> G1Projective {
    points
        .iter()
        .zip(scalars.iter())
        .map(|(p, s)| p.mul(s.into_repr()))
        .sum()
}

fn bench_points_mul_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("Points Multiplication Sum");
    let mut rng = thread_rng();

    for &size in [10, 50, 100].iter() {
        let points: Vec<G1Projective> = (0..size).map(|_| G1Projective::rand(&mut rng)).collect();
        let scalars: Vec<Fr> = (0..size).map(|_| Fr::rand(&mut rng)).collect();

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| points_mul_sum(&points, &scalars));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_points_mul_sum);
criterion_main!(benches);

