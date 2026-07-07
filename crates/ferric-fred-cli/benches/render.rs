//! divan microbench for the `fred chart` TUI render work (perf pilot — ADR-0026).
//!
//! `fred chart` startup can't be timed with hyperfine — its event loop blocks on
//! `event::read()` from the tty, so it never exits on its own, and crossterm
//! reads keys from `/dev/tty` not stdin (so no piped `q`). Instead we measure the
//! only part that's ours and could regress: `build_chart_data` (the once-per-
//! startup reduction) and `render_chart` into a headless ratatui `TestBackend`
//! (the per-draw cost). No tty, no network — deterministic and CI-runnable.
//!
//! Run with `cargo bench -p ferric-fred-cli --bench render`. Local only: divan
//! has no Bencher adapter (ADR-0026), so this isn't uploaded to bencher.dev.

use chrono::NaiveDate;
use divan::{black_box, Bencher};
use ferric_fred::Observation;
use ferric_fred_cli::chart::{build_chart_data, render_chart};
use ratatui::{backend::TestBackend, Terminal};

fn main() {
    divan::main();
}

/// Point counts spanning a short series to a dense one.
const POINTS: [usize; 3] = [16, 256, 4096];

/// Deterministic observations: a parabola (real vertical spread for the plot),
/// dated by day from 1900-01-01. The values are what gets plotted; the dates
/// only need to be valid and ordered.
fn observations(n: usize) -> Vec<Observation> {
    let base = NaiveDate::from_ymd_opt(1900, 1, 1).expect("valid date");
    (0..n)
        .map(|i| {
            let d = base + chrono::Duration::days(i as i64);
            let x = i as f64 - n as f64 / 2.0;
            Observation {
                realtime_start: d,
                realtime_end: d,
                date: d,
                value: Some(x * x),
            }
        })
        .collect()
}

/// The once-per-startup reduction: drop missing values, sort, compute axis
/// bounds, tick labels, and the stats line.
#[divan::bench(args = POINTS)]
fn build_data(bencher: Bencher, points: usize) {
    bencher
        .with_inputs(|| observations(points))
        .bench_values(|obs| black_box(build_chart_data(black_box(&obs))));
}

/// The per-draw cost: render one frame into a headless `TestBackend`. Chart data
/// is built once, outside the timed region.
#[divan::bench(args = POINTS)]
fn render_frame(bencher: Bencher, points: usize) {
    let data = build_chart_data(&observations(points)).expect("enough points to plot");
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).expect("test backend");
    bencher.bench_local(|| {
        terminal
            .draw(|frame| render_chart(frame, black_box(&data), "bench"))
            .expect("frame renders");
    });
}
