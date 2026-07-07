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
- **criterion → retained as the CI-tracked harness (not just a baseline).** It
  began as a minimal mirror of one divan workload
  (`deserialization_criterion.rs`) to ground the vs-criterion comparison — but the
  pilot surfaced that **Bencher has no divan adapter** (it parses `rust_criterion`
  and `shell_hyperfine` natively; divan would need hand-formatted Bencher Metric
  Format JSON). So criterion earns an ongoing role: it is the harness Bencher
  ingests for deserialization tracking in CI. It stays one workload, not a second
  suite.
- **hyperfine → Adopt.** `scripts/bench-cli.sh` times `fred` startup (offline,
  deterministic — the reliable regression signal) and a live fetch-and-render
  (guarded on `FRED_API_KEY`, network-noisy — informative only). hyperfine ships
  in the dev flake, exports JSON, and Bencher parses it via `shell_hyperfine`.
- **bencher.dev → Trial, wired (hosted).** `.github/workflows/bench.yml` +
  `scripts/bench-ci.sh` upload results to the hosted project `ferric-fred`:
  deserialization via the criterion mirror (`rust_criterion`, gated with `--err`
  since the workload is deterministic) and CLI startup via hyperfine
  (`shell_hyperfine`, tracked but not gated — shared-runner wall-clock is
  noisier). Auth is a Bencher **project API key** (`bencher_run_…`, `--key` —
  bencher's JWT `--token` form is deprecated), stored in Infisical at
  `dev:/shared` and injected via the existing `ferric-fred-ci` machine identity
  (ADR-0016/0018) — never a GitHub secret; the workflow is a gated no-op until
  that identity is configured.
  Hosted (not self-hosted) was chosen: a token and a project slug are the whole
  setup, versus standing up and operating an instance for a pilot. In parallel,
  CI still guards every bench with `cargo bench --no-run` so they can't rot.

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
  vs. a saved baseline, rich reports — and, decisively for the tooling here, a
  native Bencher adapter (divan has none). For *this* workload — a deterministic
  parse where we want a fast, readable *local* signal — divan is the better fit;
  but criterion remains the harness we feed to Bencher in CI. The two are
  complementary, not redundant: divan local, criterion for tracked regressions.

## Consequences

- The deserialization hot path now has a committed, runnable benchmark: any PR
  that regresses parse throughput can be measured locally with one command, and
  CI keeps the benches compiling.
- A concrete, first-hand radar data point exists: divan and hyperfine move to
  Adopt with rationale; bencher is a scoped, well-understood Trial rather than an
  open question.
- We carry a small, deliberate cost: a second bench harness (criterion) and its
  heavier dependency tree remain as dev-dependencies. This is no longer just for
  the comparison — criterion is the harness Bencher ingests, so it stays as long
  as we track deserialization in CI (divan has no Bencher adapter). If a divan
  adapter later lands (or we hand-format Bencher Metric Format JSON from divan),
  the criterion mirror could be revisited.
- One follow-up remains: give the TUI a headless render-and-exit path so
  `fred chart` startup can be timed by hyperfine without a fragile stdin hack
  (crossterm reads the tty, not stdin). Bencher CI wiring — previously the other
  open follow-up — is done as of this ADR.

## Alternatives considered

- **criterion as the primary harness** — the incumbent, but heavier to compile
  and more ceremonious to parametrize; its statistical/reporting strengths are
  better delegated to bencher for this workload. Kept only as the baseline.
- **A synthetic pure-algorithm benchmark** — would exercise miri/kani too, but
  ferric-fred has no such surface; inventing one would measure a fiction, not the
  code we ship. miri/kani stay parked (out of scope, above).
- **Self-hosted Bencher** — evaluating the self-hosted operational cost was in
  scope, but for a Trial it buys nothing the hosted tier doesn't: the point is to
  learn whether Bencher's tracking/gating fits our workflow, not to run the
  service. Hosted needs only a token + slug. Self-hosting stays an option if a
  data-residency or cost reason ever appears.
- **A GitHub Actions secret for the Bencher token** — simplest, but it would put
  secret material in the repo's settings, against the org's single-front-door
  policy (secrets live in Infisical; CI reads them via the machine identity).
  Routed through `dev:/shared` instead, exactly like `FRED_API_KEY` and
  `CARGO_REGISTRY_TOKEN`.
- **Timing the TUI via piped stdin** — crossterm reads key events from the tty,
  not stdin, so a pipe would measure the harness, not the app. Deferred to a real
  headless-exit affordance.
