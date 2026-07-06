# ADR-0023: MCP tool output schemas via a feature-gated `schemars` derive

- **Status:** Accepted
- **Date:** 2026-07-06
- **Deciders:** Otto Hermann

## Context

Each MCP tool already advertises an **input** schema (derived from its typed
parameter struct) and behavioural **annotations** (read-only, idempotent,
non-destructive, open-world — [ADR-0010], added in #29). What it does *not*
advertise is an **output** schema: the shape of the structured JSON a tool
returns. The MCP spec supports a per-tool `outputSchema`, and Glama's per-tool
quality scores flag its absence as the main remaining completeness gap. Declaring
it lets a client or agent know the exact return shape without a trial call.

The tool return types live in the `ferric-fred` **library** (`Series`,
`SeriesSearchResults`, `Observation`, `Release`, the recursive `ReleaseTable`
tree, `Source`, `Tag`, `VintageDates`, the pagination wrappers, and the id
newtypes). Generating a JSON Schema for them means deriving
`schemars::JsonSchema`. But [ADR-0010] deliberately kept `schemars` a
**MCP-crate-only** dependency — the library's input-arg schemas are derived in
the MCP crate precisely so plain library consumers (and the CLI) don't pay for a
schema generator they never use.

Two further wrinkles:

- A handful of tools don't return a single library type — they wrap a bare list
  in a small `{count, …}` envelope built ad hoc with `serde_json::json!`
  (`get_observations`, `get_category_children`, `get_category_related`,
  `get_release_sources`, `get_series_categories`). There is no type to derive a
  schema from.
- Two library types carry **split serde renames** — `SeriesSearchResults`
  serializes `series` but deserializes FRED's `seriess`; `ReleaseTable`
  serializes `roots` but deserializes FRED's `elements`. A schema generated under
  the default (deserialize) contract would advertise the FRED-side names, which
  is *not* what the tool emits.

## Decision

**We will add an optional, default-off `schemars` feature to the library and use
it to attach an output schema to every MCP tool.**

- **Library:** an optional `schemars` dependency behind a `schemars` cargo
  feature (`schemars = ["dep:schemars"]`), off by default. The public return
  types gain `#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]`;
  the two string-valued enums with custom serde impls (`Frequency`,
  `SeasonalAdjustment`) get a small hand-written `JsonSchema` impl that mirrors
  their `Serialize` (a plain string). The `chrono04` schemars integration
  supplies the schema for the `NaiveDate` fields.
- **MCP crate:** enables `ferric-fred`'s `schemars` feature, and gives each
  `#[tool(...)]` an explicit `output_schema = output_schema_of::<T>()`, where `T`
  is the type the tool structurally returns. The five envelope tools get real
  `Serialize + JsonSchema` output structs (in `output.rs`) and serialize through
  them, so the emitted value and the advertised schema share one definition.
- **Serialize contract.** `output_schema_of` generates the schema under
  schemars' **serialize** contract (draft 2020-12, top-level `title`/`description`
  stripped), so the split-rename types describe what the tool emits (`series`,
  `roots`), not what it parses (`seriess`, `elements`).
- **Error semantics unchanged.** Tools keep returning
  `Result<CallToolResult, ErrorData>` and still surface FRED-side failures as
  readable tool-level errors (`CallToolResult::error`, `isError: true`), not
  protocol errors — the explicit-schema path leaves the response bodies alone
  (no regression from #29).

## Consequences

- Every tool now reports a non-null, spec-correct `outputSchema`; the Glama
  completeness dimension improves after a Build & release + Sync.
- The library's public API gains a `JsonSchema` impl surface **only when the
  feature is on**. Default builds — and every current library/CLI consumer — are
  byte-for-byte unaffected: no new dependency, no compile-time cost. This keeps
  the spirit of [ADR-0010] (schemars is not forced on plain consumers) while
  letting the MCP crate opt in.
- The `schemars` derive becomes part of the library's compatibility surface under
  the feature: a field rename now also moves a schema property, so it's covered
  by the same "structured return" tests.
- Slight duplication: the five envelope shapes now exist as types in the MCP
  crate rather than inline `json!`. This is a net win — one source of truth for
  shape and schema, no drift.

## Alternatives considered

- **Auto-derive via `Json<T>` return types** (schemars off the `#[tool]` return
  signature). Rejected: rmcp's auto-derive path turns an `Err` into a *protocol*
  error, which would regress the readable tool-level FRED errors from #29. The
  explicit `output_schema = …` attribute keeps the response bodies as-is.
- **Derive schemas in the MCP crate via local mirror types** (keep `schemars`
  out of the library entirely, per ADR-0010's original stance). Rejected:
  re-declaring ~15 return types plus the recursive table tree would be a large,
  drift-prone shadow of the library. A feature-gated derive on the real types is
  smaller and cannot drift.
- **rmcp's built-in `schema_for_output`** helper. Rejected as-is: it hardcodes
  the *deserialize* contract, which would advertise FRED's `seriess`/`elements`
  keys instead of the emitted `series`/`roots`. `output_schema_of` mirrors its
  behaviour but under the serialize contract.
- **Always-on `schemars` (no feature gate).** Rejected: forces the generator and
  its dependency on the CLI and every library consumer for no benefit to them.

[ADR-0010]: 0010-mcp-server-design.md
