# ADR-0024: ALFRED point-in-time & vintage observations

- **Status:** Accepted
- **Date:** 2026-07-06
- **Deciders:** Project owner

## Context

FRED serves the **latest** value of every observation; ALFRED (ArchivaL FRED)
serves the data **as it was known at a point in time** — the headline
"serious analyst" capability: reproducing a series as it looked on a past date
(no look-ahead bias in a backtest), and retrieving specific revisions of a
revised series. The `series/observations` endpoint exposes this through four
request parameters we don't model today:

- `realtime_start` / `realtime_end` (ISO dates) — the real-time period a value
  was current for.
- `vintage_dates` — a comma-separated list of specific revision dates.
- `output_type` (1–4) — the response *layout*.

The library currently models only the latest data: `ObservationsRequest` has no
realtime/vintage parameters, and `Observation` is `{date, value}` — it silently
drops the `realtime_start`/`realtime_end` FRED already sends on every row
(`crates/ferric-fred/src/series.rs` notes the deferral for v1).

**A live probe settled the response shapes, and they are not uniform:**

- **`output_type=1` (default, "by real-time period") — a *tall* shape.** Every
  row is `{realtime_start, realtime_end, date, value}`. A point-in-time query
  (`realtime_start == realtime_end == <past date>`) returns the series as known
  then; a realtime *window* returns one row per date per sub-period the value
  held; `vintage_dates` selects specific revisions — all in this same tall shape.
- **`output_type=2`/`3` ("by vintage date") — a *wide, pivoted* shape.** Each row
  is `{date, "GNPCA_20150101": "…", "GNPCA_20150327": "…", …}`: one **dynamically
  named column per vintage** (`{SERIES_ID}_{YYYYMMDD}`). This is a fundamentally
  different structure from `Vec<Observation>`.
- **`output_type=4` ("initial release only")** is a niche mode that did not
  return the tall shape cleanly in probing.

## Decision

**We will add the point-in-time and specific-vintage capability now — the tall
`output_type=1` shape — and defer the wide vintage-matrix modes.** This delivers
the headline value (as-of-date and specific-vintage queries) while keeping the
slice bounded to a single, already-owned return type.

**Library.**

- `Observation` gains `realtime_start: NaiveDate` and `realtime_end: NaiveDate`,
  as **non-optional** fields. FRED sends them on *every* observation row
  (defaulting to today for a latest query), so per [ADR-0005](0005-domain-modelling-and-strong-typing.md)
  they are modelled as always-present, not `Option`. `Observation` stays
  `PartialEq`-only (it already is, for the `f64` value).
- `ObservationsRequest` gains a paired `.realtime(start, end)` setter (storing
  `Option<(NaiveDate, NaiveDate)>`, following [ADR-0019](0019-series-updates-time-window.md)'s
  time-window precedent — a real-time *period*; pass the same date twice for a
  point-in-time query) and `.vintage_dates(dates)` (a comma-joined ISO list).
- We deliberately **do not** add an `output_type` setter in this slice. The
  default (`1`) is the tall shape these parameters produce; exposing a setter
  would invite `output_type=2`/`3`, whose wide payload `Vec<Observation>` cannot
  represent.

**CLI.** `fred observations` / `fred chart` gain `--realtime-start` /
`--realtime-end` (required together) and `--vintage-dates`; the text renderer
shows each row's real-time period when an ALFRED query was made (the latest view
stays `date  value`).

**MCP.** `get_observations` gains `realtime_start` / `realtime_end` (required
together) and `vintage_dates`; the realtime fields flow into the structured
output, and — via [ADR-0023](0023-mcp-output-schemas.md) — the tool's output
schema advertises them automatically.

**Scope boundary — defer the vintage matrix.** `output_type=2`/`3` (all vintages
/ new-and-revised, the dynamically-keyed wide format) and `output_type=4`
(initial-release-only) are a distinct, value-shaped dimension needing their own
return type (a per-date map of vintage → value). Folding them in is a documented
follow-up, once we design that type — exactly as ADR-0017 deferred, then #9
delivered, release-table observation values.

## Consequences

- Full point-in-time and specific-vintage support lands for the highest-traffic
  endpoint, `series/observations`.
- `Observation` gains two always-present fields. This is additive for
  deserialization but a breaking change for any consumer that constructs the
  struct by literal or matches it exhaustively — acceptable pre-1.0, and
  release-plz will version it. Every observation now carries its real-time
  period; for a plain latest query that is `today/today` (mild redundancy, but
  it is exactly what FRED returns and it is essential for ALFRED reads).
- The realtime window is modelled as a **required pair**, slightly narrower than
  FRED (which defaults each end independently). This matches ADR-0019 and reads
  as "specify a real-time period"; the point-in-time case passes one date twice.
- Deferring `output_type=2`/`3` means "all vintages at once" still isn't
  available; a caller who wants every revision must, for now, query specific
  `vintage_dates` or a realtime window. We accept a temporarily-incomplete
  surface over bolting the wide matrix onto `Vec<Observation>`.

## Alternatives considered

- **Model `realtime_start`/`realtime_end` as `Option`.** Truly additive (old
  `{date, value}` JSON still deserializes) and lower-churn on fixtures, but it
  models always-present data as maybe-absent — the opposite of ADR-0005. Rejected
  in favour of honest non-optional fields.
- **A separate `VintageObservation` type** carrying the realtime fields, leaving
  `Observation` untouched. Rejected: the real-time period is intrinsic to a FRED
  observation, not a distinct entity; two near-identical types would duplicate
  the value/`"."` handling and split every consumer.
- **Model `output_type=2` (the wide vintage matrix) now.** Rejected for this
  slice: its dynamically-named `{SERIES_ID}_{VINTAGE}` columns need a bespoke
  return type and their own live-verified shape — a follow-up, per the scope
  boundary.
- **Expose an `output_type` enum (1–4) now anyway.** Rejected: setters 2/3 would
  produce a payload the `Vec<Observation>` return type can't hold, a foot-gun.
  The realtime/vintage parameters cover the headline use cases under the default.
