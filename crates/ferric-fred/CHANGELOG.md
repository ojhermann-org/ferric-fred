# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
