use ark_bls12_381::{Fr, G1Projective};
use ark_ff::{PrimeField, UniformRand};
use ark_ec::ProjectiveCurve;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::thread_rng;

fn bench_fixed_base_pow(c: &mut Criterion) {
    let mut rng = thread_rng();
    let base = G1Projective::rand(&mut rng);
    let scalar = Fr::rand(&mut rng);

    c.bench_function("fixed_base_pow", |b| {
        b.iter(|| {
            base.mul(scalar.into_repr())
        });
    });
}

criterion_group!(benches, bench_fixed_base_pow);
criterion_main!(benches);

