//! criterion mirror of one observations-parse workload — the vs-divan baseline
//! for the perf-tooling pilot (issue #42 / ADR-0026).
//!
//! Deliberately minimal: it benches only the latest-query observations parse at
//! the same three sizes as the divan `observations_latest` group, so the two
//! harnesses can be compared on identical work (ergonomics, output, and the
//! number they report). It is *not* a second full suite — divan
//! (`deserialization.rs`) is the primary one.
//!
//! Run with `cargo bench -p ferric-fred --bench deserialization_criterion`.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ferric_fred::Observation;

#[path = "fixtures/mod.rs"]
mod fixtures;

/// Mirror of the crate-internal `series/observations` envelope (see the divan
/// bench for the rationale).
#[derive(serde::Deserialize)]
struct ObsEnvelope {
    observations: Vec<Observation>,
}

fn bench_observations(c: &mut Criterion) {
    let mut group = c.benchmark_group("observations_parse");
    for rows in [1_000usize, 10_000, 100_000] {
        let body = fixtures::observations_body(rows, 1);
        group.throughput(Throughput::Elements(rows as u64));
        group.bench_with_input(BenchmarkId::from_parameter(rows), &body, |b, body| {
            b.iter(|| {
                let env: ObsEnvelope =
                    serde_json::from_slice(black_box(body)).expect("fixture parses");
                black_box(env.observations.len())
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_observations);
criterion_main!(benches);
