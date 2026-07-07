//! Library surface of the `fred` CLI.
//!
//! The command-line binary (`src/main.rs`) is the real entry point; this small
//! library exists only so the reusable UI logic can be exercised directly by
//! benchmarks and unit tests. Currently that is the [`chart`] module — the
//! `fred chart` TUI rendering, measured headlessly in `benches/render.rs`
//! (perf pilot — ADR-0026). The binary uses it via `ferric_fred_cli::chart`.

pub mod chart;
