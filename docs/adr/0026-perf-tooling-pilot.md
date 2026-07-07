# ADR-0026: Perf-tooling pilot — divan, hyperfine, bencher

- **Status:** Accepted
- **Date:** 2026-07-07
- **Deciders:** Otto Hermann (with Claude Code)

## Context

The [org Tech Radar](https://github.com/orgs/ojhermann-org/projects/7) vets tools
before adoption, and three performance tools sat in **Trial/Assess** with no
first-hand data: **divan** (microbenchmarking), **hyperfine** (CLI wall-clock
timing), and **bencher.dev** (CI benchmark tracking + regression gating). The
radar needed a genuine workload to produce a Trial→Adopt data point rather than a
paper evaluation (issue #42).

ferric-fred is a fitting testbed. It owns its deserialization (ADR-0004/0005): a
`series/observations` response is read as raw bytes and parsed with
`serde_json::from_slice` into a `Vec<Observation>`, where every row runs a custom
value deserializer (`"."` → `None`, else parse `f64`) plus two `NaiveDate`
parses. Long daily series and ALFRED vintage queries (ADR-0024) make that a real
hot path over tens of thousands of rows. The CLI adds a second, coarser surface:
process startup and a fetch-and-render round trip.

Two tools the radar also tracks — **miri** and **kani** — are explicitly *out of
scope*: the crate is a strongly-typed I/O client with `unsafe_code = "forbid"` and
little pure-algorithmic surface, so neither would find anything here. They stay
parked until a suitable testbed exists.

## Decision

We will **trial all three tools against the deserialization + CLI-timing
workload**, and record the following radar verdicts:

- **divan → Adopt (for microbenchmarks).** It is the primary bench harness. Two
  suites live in `crates/ferric-fred/benches/deserialization.rs`
  (`observations_latest` and `observations_alfred_vintages`, each across
  1k/10k/100k rows), fed by a deterministic, RNG-free fixture builder
  (`benches/fixtures/mod.rs`) whose JSON mirrors FRED's wire envelope exactly.
- **criterion → kept as the baseline, not adopted.** A minimal mirror of one
  divan workload (`deserialization_criterion.rs`) exists solely to ground the
  vs-criterion comparison below. It is not a second suite and should not grow
  into one.
- **hyperfine → Adopt.** `scripts/bench-cli.sh` times `fred` startup (offline,
  deterministic — the reliable regression signal) and a live fetch-and-render
  (guarded on `FRED_API_KEY`, network-noisy — informative only). hyperfine ships
  in the dev flake and can export JSON for downstream tooling.
- **bencher.dev → Trial, deferred (not wired yet).** CI *tracking* of these
  benchmarks over time — the point of bencher — needs an account and an API
  token, which per the org's secrets policy must be routed through `~/infisical`
  rather than stashed ad hoc. It lands as its own follow-up PR, at which point the
  hosted-vs-self-hosted call is made. Until then, CI guards the benches with
  `cargo bench --no-run` (they compile, they don't rot) and full timed runs are a
  local, on-demand activity.

### divan vs. criterion — the verdict

Same workload, same three sizes, both harnesses agreed on the number
(~1.5 M rows/s at 1k rows), so this is an ergonomics call, not an accuracy one:

- **Ergonomics.** divan expresses "parametrize by size" as
  `#[divan::bench(args = ROW_COUNTS)]` on a plain function; criterion needs a
  `benchmark_group` + `BenchmarkId::from_parameter` loop. divan's
  setup-outside-timing (`with_inputs(...).bench_values(...)`) reads more directly
  than criterion's `iter_batched`.
- **Output.** divan prints a compact per-size tree with a built-in items/sec
  counter inline; criterion prints per-parameter blocks and, by default, writes
  HTML reports and pulls in a heavier dependency tree (plotters, rayon).
- **Compile cost.** divan is the lighter dependency; criterion's default features
  add meaningful build time we don't need for a parse microbench.
- **What criterion still wins.** Mature statistical analysis, change-detection
  vs. a saved baseline, and rich reports. For *this* workload — a deterministic
  parse where we want a fast, readable local signal and will delegate regression
  gating to bencher — divan is the better fit.

## Consequences

- The deserialization hot path now has a committed, runnable benchmark: any PR
  that regresses parse throughput can be measured locally with one command, and
  CI keeps the benches compiling.
- A concrete, first-hand radar data point exists: divan and hyperfine move to
  Adopt with rationale; bencher is a scoped, well-understood Trial rather than an
  open question.
- We carry a small, deliberate cost: a second bench harness (criterion) and its
  heavier dependency tree remain as dev-dependencies purely for the comparison.
  If the vs-criterion question is considered settled, a future PR may drop the
  criterion mirror and its dependency.
- Two follow-ups are now explicit: **(a)** wire bencher.dev into CI (needs the
  Infisical-routed token; picks hosted vs self-hosted), and **(b)** give the TUI
  a headless render-and-exit path so `fred chart` startup can be timed by
  hyperfine without a fragile stdin hack.

## Alternatives considered

- **criterion as the primary harness** — the incumbent, but heavier to compile
  and more ceremonious to parametrize; its statistical/reporting strengths are
  better delegated to bencher for this workload. Kept only as the baseline.
- **A synthetic pure-algorithm benchmark** — would exercise miri/kani too, but
  ferric-fred has no such surface; inventing one would measure a fiction, not the
  code we ship. miri/kani stay parked (out of scope, above).
- **Wiring bencher.dev in this same PR** — blocked on an account and a secret
  token that must go through `~/infisical`; batching it here would either stall
  the whole pilot or invite an ad-hoc secret. Split out instead.
- **Timing the TUI via piped stdin** — crossterm reads key events from the tty,
  not stdin, so a pipe would measure the harness, not the app. Deferred to a real
  headless-exit affordance.
