//! divan microbenches for FRED response deserialization (perf-tooling pilot,
//! issue #42 / ADR-0026).
//!
//! Target: the `series/observations` parse hot path — `serde_json::from_slice`
//! into a `Vec<Observation>`, where every row runs the custom
//! [`Observation`] value deserializer (`"."` → `None`, else parse `f64`) and two
//! `NaiveDate` parses. We bench both the latest-query shape (realtime = today)
//! and the heavier ALFRED vintage shape (distinct realtime windows), across
//! three sizes, with an items counter so divan reports rows/sec throughput.
//!
//! Run with `cargo bench -p ferric-fred --bench deserialization`.

use divan::{black_box, counter::ItemsCount, Bencher};
use ferric_fred::Observation;

#[path = "fixtures/mod.rs"]
mod fixtures;

fn main() {
    divan::main();
}

/// Mirror of the crate-internal `series/observations` envelope: the only field
/// this slice reads is the observations array (serde drops the rest).
#[derive(serde::Deserialize)]
struct ObsEnvelope {
    observations: Vec<Observation>,
}

/// Sizes spanning a small series, a long daily series, and a stress size.
const ROW_COUNTS: [usize; 3] = [1_000, 10_000, 100_000];

/// Parse a latest-query envelope (realtime = today for every row).
#[divan::bench(args = ROW_COUNTS)]
fn observations_latest(bencher: Bencher, rows: usize) {
    bencher
        .counter(ItemsCount::new(rows))
        .with_inputs(|| fixtures::observations_body(rows, 1))
        .bench_values(|body| {
            let env: ObsEnvelope =
                serde_json::from_slice(black_box(&body)).expect("fixture parses");
            black_box(env.observations.len())
        });
}

/// Parse an ALFRED vintage-query envelope (distinct realtime windows per row) —
/// the heavier real-world shape a point-in-time / `vintage_dates` query returns.
#[divan::bench(args = ROW_COUNTS)]
fn observations_alfred_vintages(bencher: Bencher, rows: usize) {
    bencher
        .counter(ItemsCount::new(rows))
        .with_inputs(|| fixtures::observations_body(rows, 8))
        .bench_values(|body| {
            let env: ObsEnvelope =
                serde_json::from_slice(black_box(&body)).expect("fixture parses");
            black_box(env.observations.len())
        });
}
