use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tetramem_v12::universe::cognitive::topology::TopologyEngine;
use tetramem_v12::universe::coord::Coord7D;
use tetramem_v12::universe::hebbian::HebbianMemory;
use tetramem_v12::universe::memory::MemoryCodec;
use tetramem_v12::universe::node::DarkUniverse;
use tetramem_v12::universe::pulse::{PulseEngine, PulseType};

fn bench_materialize(c: &mut Criterion) {
    let mut u = DarkUniverse::new(100_000_000.0);
    c.bench_function("materialize_biased", |b| {
        let mut i = 0i32;
        b.iter(|| {
            let c = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
            u.materialize_biased(black_box(c), 100.0, 0.6).ok();
            i += 1;
        })
    });
}

fn bench_encode_decode(c: &mut Criterion) {
    let mut u = DarkUniverse::new(100_000_000.0);
    let data: Vec<f64> = (0..14).map(|i| (i as f64 + 1.0) * 0.1).collect();
    let anchor = Coord7D::new_even([100, 100, 100, 0, 0, 0, 0]);
    let mem = MemoryCodec::encode(&mut u, &anchor, &data).unwrap();

    c.bench_function("encode_14d", |b| {
        let mut j = 200i32;
        b.iter(|| {
            let c = Coord7D::new_even([j, 100, 100, 0, 0, 0, 0]);
            MemoryCodec::encode(&mut u, black_box(&c), black_box(&data)).ok();
            j += 1;
        })
    });

    c.bench_function("decode_14d", |b| {
        b.iter(|| MemoryCodec::decode(black_box(&u), black_box(&mem)).unwrap())
    });
}

fn bench_pulse(c: &mut Criterion) {
    let mut u = DarkUniverse::new(100_000_000.0);
    let mut h = HebbianMemory::new();
    for x in 0..20i32 {
        for y in 0..20 {
            for z in 0..20 {
                u.materialize_biased(Coord7D::new_even([x, y, z, 0, 0, 0, 0]), 50.0, 0.6)
                    .ok();
            }
        }
    }
    let engine = PulseEngine::new();
    c.bench_function("pulse_propagate", |b| {
        let mut i = 0i32;
        b.iter(|| {
            let src = Coord7D::new_even([i % 20, 0, 0, 0, 0, 0, 0]);
            engine.propagate(black_box(&src), PulseType::Exploratory, &u, &mut h);
            i += 1;
        })
    });
}

fn bench_topology(c: &mut Criterion) {
    let mut u = DarkUniverse::new(100_000_000.0);
    for x in 0..15i32 {
        for y in 0..15 {
            for z in 0..15 {
                u.materialize_biased(Coord7D::new_even([x, y, z, 0, 0, 0, 0]), 50.0, 0.6)
                    .ok();
            }
        }
    }
    c.bench_function("topology_analyze_3375nodes", |b| {
        b.iter(|| TopologyEngine::analyze(black_box(&u)))
    });
}

fn bench_conservation(c: &mut Criterion) {
    let mut u = DarkUniverse::new(10_000_000.0);
    for i in 0..1000i32 {
        u.materialize_biased(Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]), 100.0, 0.6)
            .ok();
    }
    c.bench_function("verify_conservation_1000nodes", |b| {
        b.iter(|| u.verify_conservation())
    });
}

criterion_group!(
    benches,
    bench_materialize,
    bench_encode_decode,
    bench_pulse,
    bench_topology,
    bench_conservation,
);
criterion_main!(benches);
