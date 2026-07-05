# ADR-0019: series/updates time window (non-ISO timestamps)

- **Status:** Accepted
- **Date:** 2026-07-05
- **Deciders:** Project owner

## Context

`series/updates` (the "what changed recently" feed) accepts an optional
`start_time` / `end_time` pair that narrows results to series updated within a
time range, down to the minute. Two things make it unlike every other date
parameter we model:

- **The format is non-ISO.** FRED wants `yyyymmddhhmm` (24-hour, e.g.
  `201803021420` for 2018-03-02 14:20), not the ISO `yyyy-mm-dd` we use for
  `observation_start` and friends. FRED documents these as wall-clock times in
  its own zone, with no offset supplied.
- **The two are a required pair.** FRED rejects `start_time` without `end_time`
  and vice versa — they only work together.

[ADR-0013](0013-endpoint-addition-pattern.md) added `series/updates` (the
`UpdatesFilter` enum + builder) but explicitly deferred this window as a "non-ISO
timestamp" case needing its own decision. This is that decision.

## Decision

We will model the pair as **one builder method** rather than two setters:

```rust
pub fn time_window(self, start: NaiveDateTime, end: NaiveDateTime) -> Self
```

- **A single method takes both bounds**, so "one set, the other missing" is
  unrepresentable — the required-pair rule is enforced by the type system, not by
  a runtime check. Internally the builder holds `Option<(NaiveDateTime,
  NaiveDateTime)>`.
- **`NaiveDateTime`, passed through without timezone conversion.** FRED's times
  are wall-clock in its zone and carry no offset, so a timezone-aware type would
  imply precision we don't have. We serialize each bound with `format("%Y%m%d%H%M")`
  (minute granularity; any seconds are dropped, matching FRED).
- **CLI:** `fred updates --start-time <T> --end-time <T>`, each `requires` the
  other (clap), accepting `YYYY-MM-DDTHH:MM` or `YYYY-MM-DD HH:MM` (seconds
  optional).
- **MCP:** `get_series_updates` gains string `start_time` / `end_time` params;
  the handler validates both-or-neither and parses them the same way.

## Consequences

- The invalid half-set state cannot be constructed in the library; the CLI and
  MCP re-impose the pairing at their own boundaries (clap `requires`; a handler
  check) because their inputs arrive separately.
- No timezone handling to get wrong — but callers must know FRED's times are in
  FRED's zone; we document it rather than convert.
- Minute granularity only, which is all FRED offers here.
- A second timestamp shape now lives in the codebase (`%Y%m%d%H%M` alongside ISO
  dates); it is localized to this one builder, so the surface stays small.

## Alternatives considered

- **Two independent setters** (`.start_time(dt)` / `.end_time(dt)`) — mirrors the
  `observation_start` / `observation_end` shape, but lets a caller set one and
  not the other, which FRED rejects only at request time. Rejected: the paired
  method makes the illegal state unrepresentable.
- **A dedicated `FredTimestamp` newtype** wrapping the format — more ceremony than
  a `NaiveDateTime` + one `format(...)` call earns for a single endpoint.
  Rejected as premature; revisit if a second endpoint needs the same shape.
- **A timezone-aware `DateTime<Tz>`** — FRED supplies no offset for these times,
  so converting would fabricate precision. Rejected in favour of an honest naive
  time plus documentation.
