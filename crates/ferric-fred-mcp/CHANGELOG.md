# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.3](https://github.com/ojhermann-org/ferric-fred/compare/ferric-fred-mcp-v0.3.2...ferric-fred-mcp-v0.3.3) - 2026-07-06

### Added

- *(mcp)* annotate all tools as read-only, idempotent, non-destructive, open-world ([#29](https://github.com/ojhermann-org/ferric-fred/pull/29))

### Other

- add Glama score + card badges to the READMEs ([#27](https://github.com/ojhermann-org/ferric-fred/pull/27))

## [0.3.2](https://github.com/ojhermann-org/ferric-fred/compare/ferric-fred-mcp-v0.3.1...ferric-fred-mcp-v0.3.2) - 2026-07-06

### Other

- *(mcp)* add Dockerfile, MCP registry manifest, and Glama config
- adopt per-crate changelogs, seeded with full history

## [0.3.1] - 2026-07-06

### Documentation

- Add a crate `README.md` (with an MCP client config snippet) for the crates.io
  landing page. No tool changes.

## [0.3.0] - 2026-07-05

### Changed

- Bumped in lockstep with the workspace; now builds on `ferric-fred` 0.3.0.
  No tool changes — auto-pagination is a library-level feature.

## [0.2.0] - 2026-07-05

### Added

- `start_time` / `end_time` on the `get_series_updates` tool (ADR-0019).

## [0.1.0] - 2026-07-05

### Added

- Initial release of the `fred-mcp` MCP server: 31 tools exposing FRED's read
  endpoints — series search, metadata, and observations; categories; releases
  (with sources, dates, and the release-table tree); sources; tags; the
  recently-updated feed; and vintage dates — as JSON over stdio (ADR-0010).
