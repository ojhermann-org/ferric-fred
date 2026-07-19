# ADR-0030: The L2 testing stance — targeted class-tests, broad `proptest` declined

- **Status:** Accepted <!-- Proposed | Accepted | Deprecated | Superseded by ADR-XXXX -->
- **Date:** 2026-07-16
- **Deciders:** Otto Hermann (ratified the targeted stance below on 2026-07-19)

## Context

[ADR-0029](0029-shared-disciplines-across-the-sibling-rust-mcp-repos.md) (the
cross-repo shared-disciplines index) records ferric-fred as **PARTIAL / gap** on
shared lesson **L2** — *"test the class, not the instance; pin every stated
assumption"* — whose canonical owner is `rustrolabe` ADR-0107. The L2 discipline
has three concrete forms:

1. **Universals → property tests** (`proptest`): an assertion true for *all*
   inputs earns a generator, not a handful of examples.
2. **Finite enums → exhaustive iteration**: a claim about *every* variant is
   tested over every variant, so adding a variant that breaks the claim fails.
3. **Type invariants → `compile_fail` doctests**: an invariant enforced by the
   type system is pinned by a doctest that must *fail* to compile.

Today ferric-fred has **none** of these. Its testing strategy
([ADR-0011](0011-testing-strategy.md)) is deliberately layered and
**example-based**: colocated unit tests, offline `wiremock` HTTP-mocked
integration tests (the CI authority), `#[ignore]`d live tests against the real
FRED API (the wire-format authority), and the agent-driven MCP audit
([ADR-0028](0028-agent-driven-mcp-testing.md)). ADR-0029 asks that the L2 stance
stop being *implicit-in-practice* and become a **written decision either way**:
adopt the L2 forms where they are cheap, **or** record why they are declined.

The point ADR-0029 (G2) makes is not "ferric-fred must adopt `proptest`." It is
"decide, and write the decision down." This ADR is that decision.

### What is different about ferric-fred

The sibling that owns L2's *property-test* branch most heavily is `time-value`,
and the reason is instructive: `time-value` is a **pure computational** library
whose correctness *is* a set of mathematical universals — PV inverts FV, NPV is
monotonic in the rate, a continuous round-trip is the identity. Those are exactly
what `proptest` is built to falsify, so a property test there has real teeth a
point test lacks.

ferric-fred's correctness surface is a **different shape**. It is an HTTP client
whose core job is *faithfully mapping FRED's wire format to Rust types*: build a
URL with the right query parameters, deserialize FRED's JSON into typed values,
classify errors. The load-bearing risk is **wire-format fidelity** — does the
deserializer handle FRED's *actual* responses? — and that risk is addressed by
tests against **real recorded/live JSON** (mocked + `#[ignore]`d live), not by
generating random structurally-valid inputs a `proptest` generator would produce.
A random `Observation` that round-trips through serde proves little about whether
ferric-fred reads what FRED actually sends. The domain has **few pure-function
universals** for `proptest` to earn its keep on.

There is, however, one L2 form that *does* have cheap teeth here.

## Decision

**Recommended (pending the owner's ratification): a *targeted* L2 stance —
decline broad `proptest` adoption, and adopt the one L2 form that catches a real,
plausible ferric-fred bug: exhaustive finite-enum round-trips.**

Concretely:

- **Decline broad `proptest`.** ferric-fred will not add `proptest` as a general
  testing tool. Its risk is wire-format fidelity, covered by example tests against
  real FRED JSON; random structurally-valid inputs add a dev-dependency and
  generator-maintenance cost for little marginal catch over the mocked/live layer.
  This is a considered decline, in the spirit of `time-value`'s own decided-skip
  of its `ALL × ALL` currency test — an L2 form is adopted only where it closes a
  *real* gap, not for symmetry.
- **Adopt exhaustive finite-enum round-trips** (L2 form 2) for the four
  **inbound, serde-deserialized** vocabularies — `Frequency`,
  `SeasonalAdjustment`, `RegionType`, `ShapeType`. These are the enums where a
  variant added without wiring its serde label is *silently swallowed* by the
  hand-written `Deserialize`'s `Other(String)` catch-all instead of failing — a
  genuinely plausible mistake given the hand-written `serde`/`Display` impls, and
  exactly the class-level bug L2 targets. Because these enums are
  `#[non_exhaustive]` with no `strum`/`EnumIter` (a dependency the crate declines),
  exhaustiveness is a hand-maintained variant list plus a round-trip assertion,
  mirroring `time-value`'s `Currency::ALL` drift tripwire.
- **The six outbound query-param enums need nothing further.** `Units`,
  `SortOrder`, `SearchType`, `OrderBy`, `UpdatesFilter`, and `AggregationMethod`
  are *outbound-only*: they carry no `serde` derive and no `Other(String)`, and
  map to FRED via a `query_code()` whose `match` is **exhaustive with no wildcard
  arm** — so a variant added without wiring its code is a **compile error**, not a
  silent swallow. Each already carries a `query_codes_match_fred()` test on top of
  that. The L2 "test every variant" intent is thus *already met* for these, by the
  type system plus an existing test; adding round-trip tests here would be
  redundant ceremony, not new teeth. The adopted scope above is deliberately the
  four inbound enums only.
- **`compile_fail` doctests (L2 form 3): optional, low priority.** The sealed
  `Paginate`/`Page` traits and the infallible-by-design id constructors are
  already enforced structurally; a `compile_fail` doctest would mostly *document*
  them. Adopt case-by-case if a specific invariant is worth pinning, not as a
  sweep.

The G1 auto-trait pin already landed (`tests/thread_safety.rs`) is, in effect, the
L5 sibling of L2 form 3 — a compile-time lock on a stated invariant — so ferric-fred
is not without class-level tests; this ADR settles the *`proptest`/enum* question
specifically.

> **Ratification note.** "Adopt vs. decline `proptest`" is an owner-level
> testing-posture call; this ADR was drafted to a clear recommendation and the
> owner ratified the targeted stance on 2026-07-19. Follow-ups: the exhaustive
> round-trip test for the four inbound enums lands as its own PR, and ADR-0029's
> **L2 row** is updated from *PARTIAL / gap* to the targeted-conformance decision.

## Consequences

- ferric-fred's L2 posture becomes a **written decision** rather than an
  implicit-in-practice absence, closing the ADR-0029 G2 action item.
- The example-based strategy of ADR-0011 stands as the primary correctness
  mechanism, now with an *explicit rationale* tied to the crate's wire-fidelity
  risk shape — not left looking like a lesson ferric-fred simply hasn't caught up
  to.
- A small, high-teeth safety net appears around the four inbound serde enums:
  adding a variant without its label becomes a test failure instead of a silent
  `Other(String)` swallow. The six outbound query-param enums are left as-is —
  their exhaustive `query_code()` match plus existing `query_codes_match_fred()`
  tests already meet the L2 "every variant" intent, so no new tests are owed there.
- The decline is **revisitable**: if a future endpoint introduces genuine
  pure-function universals (a non-trivial pagination cursor arithmetic, a
  date-window computation), a scoped `proptest` for *that* can be added under this
  ADR's "adopt where it closes a real gap" principle without reversing it.
- L2's canonical statement stays owned by `rustrolabe` ADR-0107; this ADR only
  records ferric-fred's conforming *position*, by reference.

## Alternatives considered

- **Adopt `proptest` broadly** (full L2 form 1). Rejected as the recommendation:
  ferric-fred has few pure-function universals, so a general `proptest` suite would
  test serde round-trips of randomly-generated-but-not-FRED-shaped values — high
  ceremony, low marginal catch over the mocked/live example tests that already own
  wire fidelity. Kept available for a *specific* future universal (see
  Consequences), not as a blanket adoption.
- **Decline all three L2 forms outright.** Rejected: the finite-enum round-trip is
  cheap and catches a real, plausible bug (a variant added without serde wiring,
  masked by `Other(String)`). Declining it too would be dishonest tidiness — it
  closes a gap `proptest` never would.
- **Adopt everything the canonical bar lists** (proptest + exhaustive + compile_fail
  sweep). Rejected: it would import a discipline shaped for a computational library
  wholesale into an I/O client, against ADR-0007's velocity posture and ADR-0011's
  deliberate example-based layering — ceremony that removes no real failure mode,
  the exact anti-pattern ADR-0027 warns against.
- **Leave the stance implicit** (do nothing). Rejected: that is precisely what
  ADR-0029 G2 flags. L2 asks for a *written* decision either way; an unwritten
  decline is indistinguishable from an oversight.
