# ADR-0013: Endpoint-addition pattern

- **Status:** Accepted <!-- Proposed | Accepted | Deprecated | Superseded by ADR-XXXX -->
- **Date:** 2026-07-05
- **Deciders:** Project owner

## Context

Since the first `series/observations` slice, the client has grown to cover most
of FRED's read surface: the `series` endpoints (metadata, observations, search,
updates, vintagedates, categories, release, tags) and the four discovery axes —
`category`, `release`, `source`, and `tag` — each with its filtered-series or
filtered-releases variant. Every one of these was added the same way: a domain
type and request builder in the library, a subcommand or view in the `fred` CLI,
and a `#[tool]` in the `fred-mcp` server, each layer committed and verified on
its own.

Having repeated that shape roughly seven times, the *how* is now a de-facto
convention rather than a decision. Two things are worth recording: the slice
procedure itself (so future additions are mechanical and consistent), and the
list-builder unification we landed midway through, so the next contributor knows
to extend a builder rather than copy one. This ADR does not introduce anything
new — it ratifies what the codebase already does and marks its boundaries.

## Decision

We will add each new FRED endpoint as a **vertical slice through the three
crates, library → CLI → MCP, in that order**, with each layer a separate commit
that passes the full gate and is verified live before the next begins.

**Library (`ferric-fred`).**

- Model the response as a domain type deriving `Debug, Clone, PartialEq, Eq`
  (where the fields allow) and `Serialize + Deserialize`. Numeric identifiers get
  a `Copy` `u32` newtype with `#[serde(transparent)]` (per
  [ADR-0005](0005-domain-modelling-and-strong-typing.md)); string-keyed entities
  (e.g. tags) stay bare. Reuse existing response envelopes and result types
  rather than duplicating them.
- Any endpoint with optional parameters gets a request **builder**: `#[must_use]`,
  chainable setters returning `Self`, a `pub(crate) query_params(&self) ->
  Vec<(&'static str, String)>`, and an `async send(self)` that delegates to a
  private `Client::execute_*`. `api_key`/`file_type` are added by the client, not
  the builder.
- **Share one builder across endpoints that differ only in path + one fixed
  facet.** `SeriesListRequest` serves `category/series`, `release/series`, and
  `tags/series`; `ReleasesRequest` serves `releases` and `source/releases`;
  `TagsRequest` serves `tags` and `related_tags`. Such a builder stores a
  `path: &'static str` and its optional facet and dispatches on the stored path.
  Add a *constructor*, not a new builder type. Standalone list endpoints keep
  their own builder.

**CLI (`ferric-fred-cli`).** Expose the endpoint as a subcommand, or — for a
series-scoped reverse lookup — a `--flag` view on the `series` command, with
mutually exclusive views grouped via a clap `group`. Argument enums live in
`args.rs` as `clap::ValueEnum` mirrors that `From`-convert to the library enums
(keeping `clap` out of the library). Every data command honours the global
`--json`; flags that need another argument use clap `requires` so misuse is a
parse-time error.

**MCP (`ferric-fred-mcp`).** One `#[tool]` per endpoint with a typed
`Parameters` struct (`schemars` stays in this crate only, per
[ADR-0010](0010-mcp-server-design.md)), a structured-JSON success result, and
FRED-side failures surfaced as `CallToolResult::error`. Update the `get_info`
instructions and the module tool list.

**Testing** follows [ADR-0011](0011-testing-strategy.md): a `wiremock` mocked
test asserting the exact query parameters reach the right path, a small
`#[ignore]` live test, CLI subprocess tests for parsing/validation, and MCP
param-deserialization tests plus a periodic end-to-end stdio check. **Docs**:
update the README endpoint list, CLI examples, and MCP tool table each slice.

## Consequences

- New endpoints are near-mechanical and land with a consistent, predictable
  surface across all three crates; a reader who knows one endpoint knows them
  all.
- Committing per layer keeps each change small and independently reviewable, and
  the live check per layer catches wire-format surprises early (three commits per
  endpoint is the accepted cost).
- The shared list-builders keep duplication down, but they **couple** the
  endpoints that share them: if FRED diverges the response shape or parameter set
  of one, that builder or envelope must be split back out. We accept this because
  the shared shapes have been stable.
- The pattern is deliberately prescriptive and only fits endpoints shaped like
  the ones we have (a flat list or a single object, filtered by ids/text). It is
  **not** a mandate to force every remaining endpoint through it — see below.

## Alternatives considered

- **One combined change per endpoint (all three layers at once)** — fewer
  commits, but a larger diff to review and no per-layer live checkpoint. Rejected
  in favour of the incremental rhythm.
- **Code generation from an endpoint spec (macro or build script)** — would
  remove the remaining hand-written boilerplate, but the endpoint shapes vary
  enough (optional vs. required facets, differing param sets, reused envelopes)
  that a generator would be more machinery than the handful of remaining
  endpoints justify, and it would obscure the readable builders. Rejected as
  premature.
- **A single fully-generic list builder for every list endpoint** — rejected:
  forcing e.g. `releases` (no `order_by`) and `category/series` (with `order_by`)
  into one type would leak parameters an endpoint doesn't accept. We unify only
  builders that are identical modulo path + facet.
- **Applying this pattern to structurally different endpoints** — endpoints with
  a genuinely different shape (`release/tables`' recursive table tree, or a
  non-ISO timestamp window like `series/updates`' `start_time`/`end_time`) are
  **out of scope** here and warrant their own design note or ADR rather than
  being bent to fit.
