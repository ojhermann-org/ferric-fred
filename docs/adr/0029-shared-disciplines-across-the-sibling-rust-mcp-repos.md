# ADR-0029: Shared disciplines across the sibling Rust MCP repos

- **Status:** Accepted
- **Date:** 2026-07-15
- **Deciders:** Otto Hermann

## Context

Three sibling Rust repos by the same owner — `ferric-fred`, `rustrolabe`, and
`time-value` — are each a library + CLI + MCP server, and have independently
converged on the same handful of development disciplines: strong typing as a
design tool, typed MCP output schemas, closed-vs-open vocabularies, a pinned
auto-trait profile, MCP surface hygiene, a class-not-instance testing stance, and
a Nix-native single toolchain. Their ADR logs already cross-reference each other
ad hoc (this repo's [ADR-0027](0027-types-in-types-out.md) cites `rustrolabe`
ADR-0101 and `time-value` ADR-0005/0039; the siblings reciprocate), but there is
no single place that says, for a given shared lesson, **which repo's ADR is
canonical**, **how this repo conforms**, and **where this repo deliberately
diverges**.

The owner has decided to consolidate this as a **cross-repo ADR index with
canonical ownership**: each of the three repos gets one self-contained "shared
disciplines index" ADR that names the single canonical ADR per shared lesson,
states how *that* repo conforms, and records *that* repo's deliberate divergences
and gaps. This is the `ferric-fred` copy.

This is explicitly a **map, not a merge.** It creates no new shared repo and
copies no other repo's ADR text. Each shared lesson is owned by exactly one
canonical ADR in one repo; everyone else references it **by number**. The point is
navigability and honest self-assessment, not duplication — duplicated normative
text would drift, which is the exact failure mode the canonical-owner rule exists
to prevent.

## Decision

**We record, in this repo, a canonical index of the disciplines shared across
`ferric-fred`, `rustrolabe`, and `time-value`.** For each shared lesson the index
names the single canonical ADR that owns it, cites this repo's own conforming (or
non-conforming) ADRs, and marks partial conformance and gaps honestly rather than
papering over them. Divergences that are deliberate and local are recorded as
such, so a future reader does not mistake a considered difference for drift.

### Shared lessons → canonical owner

Each lesson has **one** canonical ADR (in whichever repo articulated it best);
the others conform by reference. "How ferric-fred conforms" cites this repo's own
ADRs and is deliberately honest about partials and gaps.

| # | Shared lesson | Canonical owner | How `ferric-fred` conforms |
|---|---------------|-----------------|-----------------------------|
| **L1** | **Types as a first-class design tool** — hold every design to *unrepresentable? at the chokepoint? documents-and-enforces-at-once?* **with the anti-ceremony boundary: genuinely open sets stay strings, and no fallible constructor that removes no real failure mode.** | **`rustrolabe` ADR-0101** | Conforms via **[ADR-0027](0027-types-in-types-out.md)** (and its base, **[ADR-0005](0005-domain-modelling-and-strong-typing.md)**). ferric-fred states the anti-ceremony **boundary** most explicitly of the three — open series ids stay strings, `Frequency::Other`/`SeasonalAdjustment::Other` exist precisely so we don't reject unmodelled values, and id constructors stay **infallible** because a format check removes no mistake a contributor could realistically make. That boundary clause is preserved verbatim in intent here. |
| **L2** | **Test the class, not the instance; pin every stated assumption** — universals → property tests, finite enums → exhaustive iteration, type invariants → `compile_fail` doctests. | **`rustrolabe` ADR-0107** | **PARTIAL / gap.** ferric-fred's **[ADR-0011](0011-testing-strategy.md)** uses layered *example-based* round-trip tests — unit + offline `wiremock` integration + `#[ignore]`d live tests as the authority — plus the agent-driven audit ([ADR-0028](0028-agent-driven-mcp-testing.md)). It has **no `proptest`, no exhaustive-enum iteration, and no `compile_fail` doctest** machinery. Recorded honestly as a gap, not a conformance (see G2). |
| **L3** | **Typed output layer** — MCP `outputSchema` derived from the real return type via `schemars` (feature-gated), with a conformance test. | **`rustrolabe` ADR-0102** (most rigorous: a real JSON-Schema validator + a negative "corrupt a token → must fail" test) | Conforms via **[ADR-0023](0023-mcp-output-schemas.md)**: `schemars` derive under the *serialize* contract, conformance covered by structured-return tests plus the agent audit ([ADR-0028](0028-agent-driven-mcp-testing.md)). Strong on the **derive**; lighter on the **validator** — no external JSON-Schema spec validator or negative-corruption test yet. A candidate to level up toward the canonical bar. |
| **L4** | **Closed vs open vocabularies** — `#[non_exhaustive]` always; an `Other(String)` catch-all **only** where a value must survive a serde round-trip; closed sets are curated enums with exhaustive metadata matches. | **`ferric-fred` ADR-0005 / ADR-0027** (this repo owns it — the two-tier *response-bearing vs request-only* articulation is the sharpest of the three) | This repo is **canonical.** [ADR-0005](0005-domain-modelling-and-strong-typing.md) sets the enum-with-`Other`/`#[non_exhaustive]` pattern; [ADR-0027](0027-types-in-types-out.md) draws the open-vs-constrained boundary. Conforming siblings: `rustrolabe` 0046/0103/0105, `time-value` 0034. |
| **L5** | **Auto-trait profile** — decide `Send`/`Sync` (and friends) deliberately, then **pin it with a compile-time test**. The profile is per-repo; opposite profiles are legitimate. | **`time-value` ADR-0046** | **GAP.** ferric-fred pins **nothing** — there is no `assert_send_sync`, no `static_assertions`, no `thread_safety.rs` anywhere. The client is `Send + Sync` in practice (tokio/reqwest), but that is unenforced. Recorded as a gap with a recommended follow-up (see G1). Conforming sibling: `rustrolabe` 0011 (`Send + !Sync`). |
| **L6** | **MCP surface hygiene** — read-only + open-world annotations, one-tool-per-operation, CLI/MCP parity, error classification by caller-fixability (`invalid_params` vs `internal_error`), reject unknown params / out-of-range at the boundary. | **`rustrolabe` ADR-0044 + ADR-0045** | Conforms via **[ADR-0010](0010-mcp-server-design.md)** and **[ADR-0023](0023-mcp-output-schemas.md)**. The **agent-driven MCP audit** is a shared *pattern* (ferric-fred [ADR-0028](0028-agent-driven-mcp-testing.md), `rustrolabe` 0088/0095), deliberately **not** wired into the CI gate. **Local specialization:** ferric-fred's "release an MCP-surface change on its own, promptly — Glama scores `main`" rule (`CLAUDE.md`, [ADR-0012](0012-ci-versioning-and-release.md)) is *ferric-fred-specific*, not a shared rule. |
| **L7** | **Nix-native single toolchain** — `nix develop --command …` is the one toolchain definition, identical locally and in CI. | **3-way identical — no single canonical** (all three are `ADR-0008`) | Conforms via **[ADR-0008](0008-nix-flake-dev-environment.md)**. Because all three repos landed on the identical decision under the same number, none is designated canonical; each repo's ADR-0008 stands on its own. |

### ferric-fred's deliberate divergences

These are considered, local choices — not drift, and not gaps to close. They are
recorded so a reader does not "correct" ferric-fred toward a sibling.

- **Per-crate MSRV.** ferric-fred **declines** a pinned MSRV
  ([ADR-0007](0007-rust-edition-and-msrv.md): edition 2021, "no hard MSRV
  guarantee"); `time-value` pins core to 1.85. Local by design.
- **no_std / zero-dep.** A `time-value`-only stance (its ADR-0009). ferric-fred is
  a `tokio`/`reqwest`/`rustls` HTTP client
  ([ADR-0003](0003-async-runtime-and-http-client.md)) — **not** `no_std`, and must
  not adopt it.
- **Release posture.** ferric-fred is **public**, publishing to crates.io via a
  `release-plz` auto-PR ([ADR-0012](0012-ci-versioning-and-release.md)), with
  MCP-surface changes released promptly for Glama. The siblings differ. Only the
  **meta-rule** is shared: *state your release model explicitly and hold the line.*
- **Async vs sync.** ferric-fred is deliberately **async**
  ([ADR-0003](0003-async-runtime-and-http-client.md)); the siblings are
  synchronous. Local.

## Consequences

- This repo now has a single, citable map from each shared discipline to its
  canonical ADR, plus an honest ledger of where ferric-fred conforms, where it is
  only partial (L2, L3-validator), and where it has a gap (L2, L5).
- Because the index references canonical ADRs **by number** rather than copying
  their text, the normative statement of each lesson lives in exactly one place
  and cannot drift across the three repos. The cost is one indirection: a reader
  chasing L1's full rationale follows the number to `rustrolabe` ADR-0101.
- Deliberate divergences are on the record, so a future contributor (or the owner
  months later) does not mistake ferric-fred's async/public/no-MSRV choices for
  lag behind a sibling.
- **Two recommended follow-ups this ADR surfaces but does not perform** (each its
  own future ADR/PR, so it lands as a reviewed, appropriately-versioned change):
  - **G1 — conform to L5.** Add a `thread_safety.rs` compile-time `Send`/`Sync`
    pin (e.g. a `static_assertions`/`assert_impl_all` check) for the public client
    types, so the auto-trait profile is *enforced* rather than merely true in
    practice — matching `time-value` ADR-0046's canonical bar.
  - **G2 — make the L2 stance explicit.** Either adopt property / exhaustive-enum
    / `compile_fail` tests where they are cheap (enum round-trips, wire-format
    universals), **or** record a short ADR that *deliberately declines* `proptest`
    with its rationale. Today the decline is only implicit-in-practice; L2 asks for
    it to be a written decision either way.
- A small ongoing cost: when a shared discipline moves in its canonical repo, this
  index is a place that may need a one-line update to stay accurate.

## Alternatives considered

- **A new shared repo (or crate) holding the common ADRs.** Rejected: it would
  pull decision history away from the code each decision governs, add a fourth repo
  to maintain, and force a lowest-common-denominator statement of lessons that are
  legitimately per-repo (auto-trait profile, release posture, async/sync). The
  canonical-owner-by-reference model keeps each lesson beside its best articulation.
- **Copy each shared lesson's text into all three repos.** Rejected: three copies
  of a normative statement drift, which is exactly what "one canonical ADR per
  lesson" prevents. Reference by number instead.
- **Leave the cross-references ad hoc** (each ADR citing siblings inline, as
  today). Rejected: it works for a single lesson an ADR happens to touch, but there
  is no one place that answers "who owns L-whatever, and does ferric-fred conform?"
  — which is the whole value of an index.
- **Only list conformances, omit the gaps.** Rejected as dishonest: L2 and L5 are
  real gaps, and an index that hid them would misrepresent the repo and waste the
  follow-up signal (G1/G2). The value is in the honest ledger, not a clean-looking
  one.
