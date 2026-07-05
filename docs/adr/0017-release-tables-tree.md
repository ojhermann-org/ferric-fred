# ADR-0017: Modelling `release/tables` (the recursive table tree)

- **Status:** Proposed <!-- Proposed | Accepted | Deprecated | Superseded by ADR-XXXX -->
- **Date:** 2026-07-05
- **Deciders:** Project owner

## Context

`fred/release/tables` is the last unimplemented endpoint in FRED's read surface,
and the only one [ADR-0013](0013-endpoint-addition-pattern.md) explicitly ruled
out of its mechanical slice pattern. Every endpoint built so far returns either a
single object or a **flat list** filtered by ids/text. `release/tables` returns
neither: it returns a **recursive tree** of table elements — the layout a release
uses to present its series (sections, headers, and the series nested under them).

The response shape (structural fields only; see the values note below) is:

```json
{
  "name": "Consumer Price Index for All Urban Consumers: All Items",
  "element_id": 12886,
  "release_id": "10",
  "elements": {
    "12887": {
      "element_id": 12887, "release_id": 10, "parent_id": 12886,
      "series_id": "", "type": "section", "name": "Expenditure category",
      "line": "1", "level": "0",
      "children": [
        {
          "element_id": 12888, "release_id": 10, "parent_id": 12887,
          "series_id": "CPIAUCSL", "type": "series", "name": "All items",
          "line": "2", "level": "1",
          "children": []
        }
      ]
    }
  }
}
```

Four things make this unlike anything we have modelled:

1. **It is recursive.** `children` holds more elements of the same type, nested
   to arbitrary depth. No existing domain type refers to itself.
2. **`elements` is a JSON object keyed by (stringified) element id**, not an
   array — and its values are the *root* elements, each already carrying its
   full subtree inline via `children`.
3. **An element is polymorphic.** A `series` element carries a real `series_id`;
   a `section`/`header` element carries `series_id: ""`. The `type` field
   discriminates, and its vocabulary is open-ended and thinly documented.
4. **Two request axes with no analogue elsewhere.** `element_id` fetches a
   *subtree* rooted at one element (not the whole table), and
   `include_observation_values` (+ `observation_date`) folds observation data
   into each series element — a second, value-shaped response dimension layered
   on the structural one.

The question this ADR settles: how do we model the tree in the library, how do
the CLI and MCP present it, and where do we bound the first slice.

## Decision

We will implement `release/tables` as a vertical slice (library → CLI → MCP,
per ADR-0013's rhythm), but with the tree-specific shape decided here rather
than forced into the flat-list mould.

**Library — a recursive domain type.**

- Add `ReleaseTableElement`, a node deriving `Debug, Clone, PartialEq, Eq,
  Serialize, Deserialize`, with `children: Vec<ReleaseTableElement>`. `Vec` is a
  heap pointer, so the recursion needs no explicit `Box`; the type has a known
  size.
- Add a numeric newtype `ReleaseElementId(u32)` (`#[serde(transparent)]`, `Copy`)
  per [ADR-0005](0005-domain-modelling-and-strong-typing.md) for `element_id`
  and `parent_id`.
- `series_id` deserializes to `Option<SeriesId>`, mapping FRED's empty-string
  sentinel (`""`, used by non-series elements) to `None` via a
  `deserialize_with` helper.
- `type` (a Rust keyword) is captured as a field named `element_type` of plain
  `String`. We deliberately do **not** introduce a `ReleaseElementType` enum in
  this slice — the type vocabulary is open and poorly documented, so a typed
  enum would risk a wrong/incomplete variant set. A `String` is
  forward-compatible and can be promoted to a typed enum later without changing
  the wire contract. `line` and `level` are likewise kept as `String` (FRED
  sends them as strings; `level` is redundant with nesting depth but retained
  for fidelity).
- Add `ReleaseTable`, the top-level envelope: `name`, `element_id`
  (`ReleaseElementId`), `release_id` (`ReleaseId`), and the roots. The keyed
  `elements` object is deserialized into an ordered `Vec<ReleaseTableElement>`
  (a `deserialize_with` that collects the object's *values*); each element
  already carries its own id, so the map keys are redundant, and a `Vec` gives a
  clean tree-walking API.
- Optional params → a builder, per ADR-0013: `Client::release_tables(release_id)
  -> ReleaseTablesRequest`, with `.element(ReleaseElementId)` to scope to a
  subtree, `.send() -> ReleaseTable`, and a private `Client::execute_release_tables`.

**CLI — an indented tree view.** Expose it as `fred release <id> --tables`
(joining the existing view group with `--series`/`--sources`/`--dates`/`--tags`),
with an optional `--element <id>` to print a subtree. This is the first CLI view
that renders a *tree* rather than a flat list: a small recursive helper walks
`children`, indenting by depth and printing `name`, `type`, and `series_id`
where present. `--json` still emits the raw structure.

**MCP — a structured-tree tool.** Add `get_release_tables` (`release_id`,
optional `element_id`) returning the tree as MCP structured content. Recursion is
a non-issue for JSON output, and the `Parameters` struct stays flat, so
`schemars` (input-only) is unaffected.

**Scope boundary — defer observation values.** The first slice models the
**structural tree and `element_id` subtree navigation only**.
`include_observation_values` / `observation_date` (and the per-element value
fields they add) are a distinct, value-shaped dimension whose exact response
shape needs live confirmation; folding it in is a documented follow-up once the
base tree lands and we can inspect a real payload. The structural tree is
independently useful and keeps this slice bounded and verifiable.

**Testing** follows [ADR-0011](0011-testing-strategy.md): a `wiremock` fixture
with a 2–3-level nested tree asserting the recursion deserializes and children
nest correctly; a small `#[ignore]` live test against a release with a known
table (e.g. release 10, CPI); a CLI subprocess test on the rendered indentation;
and an MCP param test plus a stdio end-to-end check. Docs: the README endpoint
list, CLI examples, and MCP tool table gain the new entry, as every slice does.

## Consequences

- FRED's read surface reaches **complete coverage** (31/31 endpoints).
- We take on the codebase's **first self-referential domain type** and its
  **first tree-rendering CLI view**. Both are contained (one recursive struct,
  one recursive print helper) but are genuinely new shapes to maintain.
- Two `deserialize_with` helpers (empty-string `series_id` → `None`; keyed
  `elements` object → `Vec`) are the first custom deserializers in the library;
  they are small and local, but they are hand-written parsing we now own.
- Keeping `element_type`/`line`/`level` as `String` trades some type-safety for
  robustness against an undocumented, possibly-growing vocabulary. A later ADR
  or slice can promote `element_type` to a forward-compatible enum if the type
  set proves stable and worth enumerating.
- Deferring observation values means the endpoint ships **structure-only** at
  first; a caller wanting the numbers inline must wait for the follow-up. We
  accept a temporarily-incomplete endpoint over guessing an unverified response
  shape.
- The slice still follows ADR-0013's three-commit, live-checked-per-layer
  rhythm; only the *shape* of the modelled data departs from the flat-list
  template, which is exactly the departure ADR-0013 anticipated.

## Alternatives considered

- **Flatten the tree to a list with `parent_id` links, and let callers rebuild
  it.** Rejected: it discards the nesting FRED already hands us and pushes tree
  reconstruction onto every consumer — the opposite of the typed-convenience
  goal in ADR-0005.
- **Pass the `elements` blob through as `serde_json::Value`.** Rejected:
  abandons strong typing (ADR-0005); callers get untyped soup and lose the whole
  point of the client.
- **A `BTreeMap<ReleaseElementId, ReleaseTableElement>` for `elements`** instead
  of a `Vec` of roots. Reasonable and preserves keyed access, but each element
  already carries its id, so the keys are redundant; a `Vec` reads more directly
  as "the roots of the tree." Rejected in favour of the `Vec`.
- **A typed `ReleaseElementType` enum now.** Deferred, not rejected outright: the
  vocabulary (`section`, `series`, `header`, `line`, …) is not authoritatively
  documented, so enumerating it now risks silently mismatching real data. A
  `String` is safe today and promotable later.
- **Include observation values in the first slice.** Deferred for the reasons
  under Scope boundary — the value/date response shape wants live confirmation,
  and the structural tree stands on its own.
- **Skip the CLI tree rendering and expose `--tables` as JSON-only.** Rejected:
  every other endpoint renders as readable text by default; an indented tree is
  the consistent choice, and `--json` remains for machine use.
