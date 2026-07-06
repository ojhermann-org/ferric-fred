# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.4](https://github.com/ojhermann-org/ferric-fred/compare/ferric-fred-cli-v0.3.3...ferric-fred-cli-v0.3.4) - 2026-07-06

### Added

- ALFRED point-in-time & vintage observations ([#40](https://github.com/ojhermann-org/ferric-fred/pull/40))
- fold observation values into release/tables ([#38](https://github.com/ojhermann-org/ferric-fred/pull/38))

## [0.3.3](https://github.com/ojhermann-org/ferric-fred/compare/ferric-fred-cli-v0.3.2...ferric-fred-cli-v0.3.3) - 2026-07-06

### Other

- updated the following local packages: ferric-fred

## [0.3.2](https://github.com/ojhermann-org/ferric-fred/compare/ferric-fred-cli-v0.3.1...ferric-fred-cli-v0.3.2) - 2026-07-06

### Other

- adopt per-crate changelogs, seeded with full history

## [0.3.1] - 2026-07-06

### Documentation

- Add a crate `README.md` for the crates.io landing page. No behavior changes.

## [0.3.0] - 2026-07-05

### Added

- Global `--all` flag on the list commands to page through results to
  exhaustion (ADR-0020).

## [0.2.0] - 2026-07-05

### Added

- `updates --start-time` / `--end-time` to bound the recently-updated feed by
  a time window (ADR-0019).

## [0.1.0] - 2026-07-05

### Added

- Initial release of the `fred` command-line tool: browse and search series,
  fetch observations with an interactive terminal chart, and explore categories,
  releases (with their sources, dates, and table tree), sources, and tags.
- Global `--json` output on the data commands.
