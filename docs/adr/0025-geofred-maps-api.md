# ADR-0025: GeoFRED / Maps API support (regional & geographic data)

- **Status:** Accepted
- **Date:** 2026-07-06
- **Deciders:** Project owner

## Context

FRED's **Maps API** (formerly GeoFRED) is a separate surface the client does not
cover: economic indicators by geographic region (state / county / MSA / country
/ BEA region / …) plus the shape files to draw them. It is served from a
**different base URL** — `https://api.stlouisfed.org/geofred/` — from the core
API's `https://api.stlouisfed.org/fred/`; same host, different path prefix. Our
`Client` today holds a single `base_url` fixed to `.../fred`
(`crates/ferric-fred/src/client.rs`), and every existing response envelope is a
core-FRED shape (`{seriess: […]}`, `{observations: […]}`, …). None of that
reaches GeoFRED.

The API has four endpoints. **A live probe (2026-07-06, all four returning real
data) settled their shapes:**

- **`shapes/file?shape=<type>`** — returns **raw GeoJSON**: a `FeatureCollection`
  with `{type, name, crs, features:[{type, properties:{…}, geometry:{type,
  coordinates}}]}`. `properties` vary by shape type (`bea` → `{bea_region,
  bea_regi_1}`); geometry is `MultiPolygon` in a **projected display coordinate
  space** (integer pixel-like pairs, e.g. `[1485, 2651]`), not lat/lon. Large
  (tens of KB) and unwrapped — no `file_type` envelope.
- **`series/group?series_id=<id>`** — a small metadata object:
  `{series_group: {title, region_type, series_group (the group id, a string),
  season, units, frequency, min_date, max_date}}`.
- **`series/data?series_id=<id>&date=&start_date=`** — the regional values of one
  series over time: `{meta: {title, region, seasonality, units, frequency,
  data: {"<date>": [{region, code, value, series_id}]}}}`. `data` is a map from
  ISO date to an array of one row per region; `value` is a JSON number (integer
  or float, e.g. `1506.5`).
- **`regional/data?series_group=<id>&region_type=&date=&units=&frequency=&season=`**
  — a group's cross-section: the **identical `meta`/`data` envelope** as
  `series/data`.

Two facts shape the design. First, `series/data` and `regional/data` return the
**same envelope**. Second, the envelope's scalar `frequency` / `seasonality` /
`units` come back as **display words** ("Monthly", "Not Seasonally Adjusted",
"Thousands of Persons") — *not* the single-letter codes the core API uses and
that our `Frequency` / `SeasonalAdjustment` enums parse.

## Decision

**We will add GeoFRED as a new endpoint family, library-first, following the
[ADR-0013](0013-endpoint-addition-pattern.md) vertical-slice rhythm**, extending
the existing `Client` rather than introducing a second client type.

**Client wiring — a second base URL on the one `Client`.**

- Add a `GEOFRED_BASE_URL = "https://api.stlouisfed.org/geofred"` const and a
  `geofred_base_url: String` field. `Client::new` sets it to the const;
  `with_base_url` (the `pub(crate)` wiremock constructor) points **both**
  `base_url` and `geofred_base_url` at the mock server so path-only matching keeps
  working unchanged.
- GeoFRED calls route through a private `execute_geofred_*` that builds
  `{geofred_base_url}{path}` and appends `api_key` (and `file_type=json` for the
  JSON endpoints; `shapes/file` returns GeoJSON directly). This mirrors the
  existing `execute_*` and keeps the "client adds `api_key`/`file_type`, not the
  builder" invariant from ADR-0013.

**Library types.**

- **`RegionalData`** — the shared envelope for both `series/data` and
  `regional/data`: `{meta: RegionalDataMeta}`, where `RegionalDataMeta` is
  `{title, region, seasonality, units, frequency, data: BTreeMap<String,
  Vec<RegionalDataPoint>>}` (date → rows) and `RegionalDataPoint` is `{region,
  code, value: Option<f64>, series_id}`. `value` is `Option<f64>` with the same
  lenient `"."`/empty → `None` handling the core client already uses for
  observation values. The `BTreeMap` gives a deterministic date order.
- **`SeriesGroup`** — `{title, region_type, series_group, season, units,
  frequency, min_date, max_date}`, deserialized from the `{series_group: …}`
  wrapper (a `#[serde(rename)]`/newtype envelope, per the existing pattern).
- **`ShapeFile`** — a lean GeoJSON `FeatureCollection`: `{name, crs, features:
  Vec<Feature>}`, `Feature {properties: serde_json::Map<String, Value>, geometry:
  Geometry}`, `Geometry {type: String, coordinates: serde_json::Value}`.
  Properties genuinely vary by shape type and geometry coordinates are deep
  nested arrays in a display projection we do not interpret, so those two carry
  `serde_json::Value` rather than being over-modelled (consistent with this
  project's refusal to force dynamic shapes into rigid types — cf. ADR-0024's
  deferral). This keeps GeoJSON support in-library with **no new dependency**.

**Strong typing (per [ADR-0005](0005-domain-modelling-and-strong-typing.md)).**

- **Request** `frequency` and `season` reuse the existing `Frequency` and
  `SeasonalAdjustment` enums — GeoFRED accepts the same codes (`a`/`q`/`m`,
  `NSA`/`SA`). Two **new** enums: `RegionType` (`state`, `county`, `msa`,
  `country`, `bea`, …) and `ShapeType` (the `shape=` values). Both get
  `clap::ValueEnum` mirrors in the CLI per ADR-0013, keeping `clap` out of the
  library.
- **Request `units` is a required free-form `String`, NOT our `Units` enum.**
  Our core `Units` enum is FRED's *data transformation* (`lin`, `chg`, `pch`…);
  GeoFRED's `units` is a **unit-of-measurement label** ("Dollars", "Thousands of
  Persons"). A live probe (2026-07-06) confirmed it is **required** (omitting it
  returns `"Bad Request. Must have units set."`) and **unvalidated** — FRED
  accepts *any* string (even `"Bogus"` or `"lin"`), interpolates it verbatim into
  the returned `title`, and does not otherwise transform the data. Modelling it as
  an enum would falsely constrain a free field, so it is a plain required `String`.
- **Response** scalar `frequency` / `seasonality` / `units` / `region` are kept as
  `String`. They are FRED **display labels** ("Monthly", "Not Seasonally
  Adjusted", "Dollars"), not codes our enums parse; round-tripping them through
  enums would be lossy and brittle. We model what FRED sends.

**Request builders.**

- `regional/data` takes `series_group`, `region_type`, `date`, and the required
  `units` (proven required by probe), plus `frequency` / `season`. `series/data`
  requires `series_id` with optional `date` / `start_date`. The **precise
  required-vs-optional split, and any further optional params** (e.g.
  `start_date` on `regional/data`), are pinned during the implementation slice's
  live test per ADR-0013 rather than guessed here — the FRED Maps docs page 403s
  to automated fetch, so the wire is the source of truth.
- `series/group` (only `series_id`) and `shapes/file` (only `shape`) are single
  required-arg calls — a `Client` method each, no builder, per ADR-0013.

**CLI / MCP exposure — data first, geometry library-first.**

- The three JSON endpoints (`series/group`, `series/data`, `regional/data`)
  become `fred` subcommands and `fred-mcp` `#[tool]`s, honouring `--json` and the
  ADR-0023 output schemas.
- `shapes/file` is exposed in the **library and CLI** (CLI dumps the raw GeoJSON
  under `--json`), but **not as an MCP tool** in this slice: a tens-of-KB polygon
  payload in a projection an LLM cannot use is poor tool ergonomics. Adding it
  later is a documented follow-up if a real need appears.

## Consequences

- The client reaches an entire new data class — regional economics with mapping
  geometry — completing FRED's read surface (the last open backlog item).
- The `Client` now carries two base URLs. This is a small, contained change and
  keeps a single connection-pooled client and one construction path; the wiremock
  constructor pointing both at the mock keeps every existing test green.
- Sharing `RegionalData` across `series/data` and `regional/data` follows
  ADR-0013's "one envelope for endpoints with the same shape" and couples them:
  if FRED ever diverges the two, the type splits back out. The shapes have been
  stable, so we accept it.
- `ShapeFile` carrying `serde_json::Value` for properties and coordinates means
  GeoJSON is *transported and re-serialized* faithfully but not *statically
  typed* down to the polygon. Consumers who need typed geometry re-parse with a
  GeoJSON crate. We accept partial modelling here rather than a large, per-shape
  type zoo for coordinates we do not interpret.
- Request `frequency`/`season` are enums while `units` and every response scalar
  are `String`. This split is honest to the wire — GeoFRED validates codes for
  the former but treats `units` and the response labels as free display text — and
  avoids a lossy parse, at the cost of a slightly uneven-looking parameter surface.
- This slice keeps ferric-fred a **faithful mirror of FRED's API surface**:
  `ShapeFile` transports GeoJSON without interpreting it, and no derived
  capability (map rendering, coordinate transforms, unit math) is added here.
  Such derived work is explicitly out of scope for this crate and would live in a
  separate library — recorded in `docs/ideas.md` rather than bolted on.

## Alternatives considered

- **A separate `GeoClient` type.** Cleaner URL separation, but duplicates
  construction, the `reqwest` client, error handling, and API-key plumbing for
  what is one extra base path on the same host. Rejected in favour of a second
  base URL on the existing `Client`.
- **Depend on the `geojson` crate for `shapes/file`.** Gives fully-typed
  `Feature`/`Geometry`, but adds a dependency (and its own `serde` types) to model
  coordinates we treat as opaque, in a non-geographic display projection. Rejected
  as more machinery than the one shape endpoint justifies; a consumer can layer
  `geojson` on the re-serialized value. Revisit if we ever render maps ourselves.
- **Two distinct types for `series/data` and `regional/data`.** Rejected: their
  probed envelopes are byte-for-byte the same shape; two types would duplicate the
  `value`/`"."` handling and the date-map, exactly the split ADR-0013 unifies.
- **Parse response `frequency`/`seasonality`/`units` into the existing enums.**
  Rejected: GeoFRED returns display words, not the core API's codes; the enums
  would need a second parse vocabulary or would fail on valid data.
- **Model the request `units` param as an enum (fixed set of measurement units).**
  Rejected on probe evidence: GeoFRED accepts *any* string for `units` (verified
  with `"Bogus"` and `"lin"`), echoing it into the title without validation. An
  enum would reject values FRED accepts and imply a domain that does not exist;
  a required `String` matches the wire.
- **Expose `shapes/file` as an MCP tool now.** Rejected for this slice: large
  projected-polygon GeoJSON is not usefully consumable by an LLM tool caller;
  library + CLI cover the real use, and an MCP tool can follow if warranted.
