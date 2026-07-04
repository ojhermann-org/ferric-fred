# ADR-0005: Domain modelling & strong typing

- **Status:** Accepted
- **Date:** 2026-07-04
- **Deciders:** Project owner

## Context

The whole premise of `ferric-fred` is a *strongly-typed* client: the FRED API
returns many fields that are really enumerations or identifiers wearing string
clothes (`frequency`, `units`, `seasonal_adjustment`, `aggregation_method`,
`filter_variable`, `order_by`, `sort_order`), plus date-stamped observation
values where missing data is the literal string `"."`. We want the type system
to encode this, while staying resilient to the API adding values we haven't seen.

## Decision

- **Identifiers are newtypes**, not bare `String`s: `SeriesId`, `CategoryId`,
  `ReleaseId`, `SourceId`, `TagName`, etc. Cheap to construct, hard to mix up.
- **Closed vocabularies are enums**: `Frequency`, `Units`, `SeasonalAdjustment`,
  `AggregationMethod`, `SortOrder`, `OrderBy`, …. Each enum carries a catch-all
  **`Other(String)`** variant (or is `#[non_exhaustive]`) so an unrecognised
  API value deserializes rather than errors — forward-compatibility over
  strictness.
- **Observation values are `Option<f64>`**: the FRED sentinel `"."` maps to
  `None`; everything else parses to `Some(f64)`. (Per
  [ADR-0004](0004-error-handling.md), a genuinely unparseable non-`"."` value is
  a `Deserialize` error, not a silent `None`.)
- **Dates are `chrono::NaiveDate`** — FRED dates are calendar dates without
  time/zone, which `NaiveDate` models exactly.
- **serde** drives (de)serialization; wire-format quirks (dotted missing values,
  string-encoded numbers, date formats) are handled with custom
  `Deserialize`/`serde(with = ...)` adapters, not pushed onto callers.
- **ALFRED vintages** (`realtime_start` / `realtime_end`) are **deferred** for
  v1, but request/response types are designed so these fields can be added
  later without a breaking change (optional, defaulted).

## Consequences

- Callers get autocomplete-friendly, mistake-resistant types instead of stringly
  data, and charts/analysis can consume `f64` directly.
- `f64` accepts negligible floating-point imprecision — acceptable for
  economic-series display and analysis (see the numeric-type decision).
- The `Other(String)` / `#[non_exhaustive]` escape hatches mean the client keeps
  working when FRED introduces a new frequency or unit before we model it.
- A modest amount of custom serde code to own the wire quirks.

## Alternatives considered

- **`Option<Decimal>` (`rust_decimal`)** — exact, but heavier and slower for
  charting, and unnecessary for FRED's precision. Rejected for v1.
- **Raw `String` fields** — maximally faithful, but abandons the project's
  entire reason to exist. Rejected.
- **Strict enums without a catch-all** — cleaner types, but a new API value
  becomes a hard deserialization failure. Rejected in favour of forward-compat.
- **`time` crate instead of `chrono`** — viable; we choose `chrono::NaiveDate`
  for its ubiquity and exact date-only fit. Revisitable if a dependency pulls us
  the other way.
