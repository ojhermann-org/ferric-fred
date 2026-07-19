# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.8](https://github.com/ojhermann-org/ferric-fred/compare/ferric-fred-v0.3.7...ferric-fred-v0.3.8) - 2026-07-19

### Fixed

- *(mcp)* clarify aggregation, regional-data size, and release-table date ([#82](https://github.com/ojhermann-org/ferric-fred/pull/82))
- *(ferric-fred)* strip comma thousands-separators in release-table values ([#81](https://github.com/ojhermann-org/ferric-fred/pull/81))

### Other

- *(ferric-fred)* exhaustive serde round-trips for the 4 inbound enums (ADR-0030) ([#76](https://github.com/ojhermann-org/ferric-fred/pull/76))
- *(ferric-fred)* pin the public Send/Sync profile with a compile-time test ([#74](https://github.com/ojhermann-org/ferric-fred/pull/74))

## [0.3.7](https://github.com/ojhermann-org/ferric-fred/compare/ferric-fred-v0.3.6...ferric-fred-v0.3.7) - 2026-07-14

### Fixed

- *(geofred)* turn the "bad id" HTTP 500 into an actionable error ([#64](https://github.com/ojhermann-org/ferric-fred/pull/64))
- *(release-tables)* stop duplicating descendants into `roots` ([#61](https://github.com/ojhermann-org/ferric-fred/pull/61))

## [0.3.6](https://github.com/ojhermann-org/ferric-fred/compare/ferric-fred-v0.3.5...ferric-fred-v0.3.6) - 2026-07-07

### Other

- *(bench)* perf-tooling pilot — divan + hyperfine (ADR-0026, #42) ([#48](https://github.com/ojhermann-org/ferric-fred/pull/48))

## [0.3.5](https://github.com/ojhermann-org/ferric-fred/compare/ferric-fred-v0.3.4...ferric-fred-v0.3.5) - 2026-07-07

### Added

- *(ferric-fred)* GeoFRED / Maps API — library layer (ADR-0025) ([#44](https://github.com/ojhermann-org/ferric-fred/pull/44))

## [0.3.4](https://github.com/ojhermann-org/ferric-fred/compare/ferric-fred-v0.3.3...ferric-fred-v0.3.4) - 2026-07-06

### Added

- ALFRED point-in-time & vintage observations ([#40](https://github.com/ojhermann-org/ferric-fred/pull/40))
- fold observation values into release/tables ([#38](https://github.com/ojhermann-org/ferric-fred/pull/38))

## [0.3.3](https://github.com/ojhermann-org/ferric-fred/compare/ferric-fred-v0.3.2...ferric-fred-v0.3.3) - 2026-07-06

### Added

- *(mcp)* advertise an output schema on every tool ([#35](https://github.com/ojhermann-org/ferric-fred/pull/35))

## [0.3.2](https://github.com/ojhermann-org/ferric-fred/compare/ferric-fred-v0.3.1...ferric-fred-v0.3.2) - 2026-07-06

### Other

- adopt per-crate changelogs, seeded with full history

## [0.3.1] - 2026-07-06

### Documentation

- Add a crate `README.md` so the crates.io and docs.rs landing pages render
  content, and refresh the crate-level docs to describe full read-endpoint
  coverage and auto-pagination. No API changes.

## [0.3.0] - 2026-07-05

### Added

- Auto-pagination via the `Paginate` trait: `send_all` walks a paginated
  endpoint to exhaustion, and `stream` yields results lazily as a `Stream`
  (ADR-0020, ADR-0021).

## [0.2.0] - 2026-07-05

### Added

- A time window (`start_time` / `end_time`) on `series/updates` (ADR-0019).

## [0.1.0] - 2026-07-05

### Added

- Initial release: a strongly-typed async client covering FRED's read
  endpoints — series (search, lists, categories, release, vintage dates, and the
  recently-updated feed), observations, categories, releases (with their
  sources, dates, and the recursive release-table tree), sources, and tags
  (including related and scoped tag facets).
- Newtype identifiers, typed enums for FRED's closed value sets, and a typed
  error taxonomy that never panics on a network or parse failure.
- `Serialize` on the domain types (ADR-0010) and HTTP-mocked endpoint tests
  (ADR-0011).
