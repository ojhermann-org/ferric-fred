# ADR-0027: Types in, types out — make illegal states unrepresentable

- **Status:** Accepted
- **Date:** 2026-07-14
- **Deciders:** Otto Hermann

> **Canonical for the sibling Rust MCP repos** (`rustrolabe`, `time-value`) for closed-vs-open vocabularies and the open-set anti-ceremony boundary — see [ADR-0029](0029-shared-disciplines-across-the-sibling-rust-mcp-repos.md).

## Context

[ADR-0005](0005-domain-modelling-and-strong-typing.md) committed the library to a
strongly-typed domain model: newtype identifiers, closed-vocabulary enums,
`Option<f64>` for the `"."` sentinel, `NaiveDate` for dates. [ADR-0023](0023-mcp-output-schemas.md)
carried that discipline to the MCP **output** surface — every tool advertises a
schema derived from the real return type, so the emitted value and its schema
share one definition and cannot drift.

Two sibling projects by the same owner have since named this stance explicitly and
found it worth recording as a first-class principle rather than a collection of
local habits:

- `rustrolabe` **ADR-0101** ("types are a first-class design tool") holds every
  design to three questions: *can the wrong state be made unrepresentable? can the
  invariant live in a type at the chokepoint every path funnels through? does the
  type communicate the behaviour to a reader and the compiler at once?* It also
  draws an explicit **boundary**: a type that adds ceremony without removing a real
  failure mode is not an improvement — "genuinely open sets stay strings."
- `time-value` **ADR-0005** + **ADR-0039** ("types in, types out") reach the same
  conclusion from the other side: an untyped layer sitting next to a typed one is a
  recurring source of consistency defects, so type both edges the same way.

`ferric-fred` is already well down this road — and, thanks to ADR-0023, *ahead* of
where the siblings started on the output edge. What it has never written down is the
principle itself, nor the two places the discipline currently stops short:

1. **Boundary validation is deferred, not designed in.** Newtype constructors do no
   validation (`ids.rs`: "Construction does no validation for now"), and
   `Error::InvalidInput` exists but is used only for the missing-`FRED_API_KEY`
   case. This is a deliberate, defensible choice for *open* identifiers — but it has
   never been distinguished, in writing, from the cases where a value really is
   constrained and a mistake really is possible.
2. **Some invariants live in runtime checks, not types.** The MCP layer enforces
   FRED's "give both or neither" pairing rules with hand-written `if`s
   (`crates/ferric-fred-mcp/src/main.rs`: `realtime_start`/`realtime_end` must come
   together; `start_time`/`end_time` must come together). Each check sits on one
   call site and a new surface (a second MCP tool, the CLI, a future consumer) can
   forget it.

This ADR records the principle and its boundary. It is a **stance, not a
refactor**: it does not itself change any code or the published API. The concrete
typed-invariant work it licenses is catalogued below and sequenced as its own
future ADRs/PRs.

## Decision

**We adopt "make illegal states unrepresentable" as the standing design test for
`ferric-fred`, on both the input and output edges — types in, types out.** When a
decision could encode an invariant in a type the compiler (or serde, at the wire
boundary) checks, we prefer that over a comment, a convention, or a runtime check,
holding it to the three questions above: *unrepresentable? at the chokepoint?
documents and enforces at once?*

**The boundary is explicit, and it is the same one the [faithful-mirror scope
principle](0005-domain-modelling-and-strong-typing.md) already implies.** A type
must remove a mistake a reasonable contributor could actually make; ceremony that
removes no failure mode is a regression, not an improvement. In FRED's domain that
means:

- **Open sets stay strings.** FRED **series identifiers** are an open set of
  thousands of arbitrary-looking codes; free-text search is free text; the
  `Frequency::Other` / `SeasonalAdjustment::Other` catch-alls (ADR-0005) exist
  precisely to *not* reject values we haven't modelled. FRED is the authority on
  whether an id or a search string is valid, and it rejects malformed ones cleanly.
  Wrapping these in *fallible* constructors would add ceremony while removing no
  real mistake — so we don't. Their newtype constructors stay **infallible**.
- **Constrained values are candidates for typed invariants.** Where a value has a
  real, checkable constraint that a contributor could get wrong — a bounded
  numeric, a "both or neither" pairing, an ordering — the invariant belongs in a
  type at the library chokepoint every surface funnels through, not in a per-surface
  runtime check. `Error::InvalidInput` is the reserved channel for the boundary
  checks this produces.

**This pass records the principle only.** ID constructors remain infallible and the
published 0.3.6 API is untouched; the work below is deferred so each piece can land
as a reviewed, appropriately-versioned change rather than a breaking churn bundled
under a docs commit.

### Deferred candidates (each its own future ADR/PR)

- **Bounded `Limit` / `Offset` newtypes.** FRED caps `limit` (endpoint-specific,
  e.g. 1–1000) and requires `offset >= 0`. A validated newtype makes an
  out-of-range page request unrepresentable at construction instead of a 400 from
  FRED. Fits the currently-underused `Error::InvalidInput`.
- **A pairing/ordering type for realtime and update windows.** Lift the
  `main.rs` `realtime_start`/`realtime_end` and `start_time`/`end_time` "both or
  neither" checks (and `start <= end` ordering) into a single library type, so the
  invariant is enforced once at the chokepoint and no surface can forget it —
  mirroring `rustrolabe`'s `Zodiac::Sidereal(Ayanamsha)` (an illegal combination
  made unrepresentable) and its `FrameRequirement` (an invariant made a library
  property).

## Consequences

- The project has a written, citable stance — new endpoints and tools are held to
  the same test, and reviewers have a shared vocabulary for "should this be a type?"
- The boundary is equally written down, so the principle does **not** become a
  mandate to wrap everything: open sets are protected from ceremony on purpose.
- No code or API change lands with this ADR; the typed-invariant work is real but
  paced, each piece choosing its own version impact (a bounded newtype on a builder
  setter can be additive; changing an id constructor to fallible would be breaking
  and is explicitly out of scope).
- A small ongoing cost: each new constrained value now prompts the "type or runtime
  check?" question rather than defaulting to a runtime check — which is the point.

## Alternatives considered

- **Do the code changes now (fallible constructors, bounded newtypes, pairing
  types) in one pass.** Rejected for this change: it conflates the *decision* with a
  multi-part, partly-breaking implementation, and the owner chose to record the
  stance first and sequence the code behind it.
- **Fallible constructors on identifiers too (validate id formats).** Rejected as
  the standing rule: series ids are an open set, FRED is the validator, and a
  format check removes no mistake a contributor could realistically make while
  breaking every `SeriesId::new` call site. This is exactly the "ceremony without a
  removed failure mode" the boundary excludes.
- **Leave the principle implicit in ADR-0005.** Rejected: ADR-0005 describes the
  *shape* of the model (newtypes, enums) but never states the test or its boundary,
  and it predates the typed-output work (ADR-0023) and the pairing-check debt. The
  siblings found the explicit statement worth its own ADR; so do we.
