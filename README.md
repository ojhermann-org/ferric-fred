# ferric-fred

[![CI](https://github.com/ojhermann-org/ferric-fred/actions/workflows/ci.yml/badge.svg)](https://github.com/ojhermann-org/ferric-fred/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![ferric-fred MCP server](https://glama.ai/mcp/servers/ojhermann-org/ferric-fred/badges/score.svg)](https://glama.ai/mcp/servers/ojhermann-org/ferric-fred)

A strongly-typed Rust client for [FRED](https://fred.stlouisfed.org/) — the
Federal Reserve Economic Data service from the Federal Reserve Bank of St. Louis
— plus a CLI (with TUI charts) and an MCP server built on top of it.

> `ferric` (iron oxide → *rust*) + `FRED`. Iron-clad, typed access to economic data.

## Workspace

A Cargo workspace of three crates — each with its own README (the crates.io /
docs.rs landing page) that carries the full usage detail:

| Crate | Binary | What it is | Details |
|-------|--------|------------|---------|
| `ferric-fred` | — | Strongly-typed async FRED client | [README](crates/ferric-fred/README.md) · [docs.rs](https://docs.rs/ferric-fred) |
| `ferric-fred-cli` | `fred` | Command-line tool with `ratatui` TUI charts | [README](crates/ferric-fred-cli/README.md) |
| `ferric-fred-mcp` | `fred-mcp` | MCP server exposing FRED to MCP clients | [README](crates/ferric-fred-mcp/README.md) |

Published versions (these badges are the source of truth — the crates version
independently, so they can drift out of lockstep):

[![ferric-fred](https://img.shields.io/crates/v/ferric-fred.svg?label=ferric-fred)](https://crates.io/crates/ferric-fred)
[![ferric-fred-cli](https://img.shields.io/crates/v/ferric-fred-cli.svg?label=ferric-fred-cli)](https://crates.io/crates/ferric-fred-cli)
[![ferric-fred-mcp](https://img.shields.io/crates/v/ferric-fred-mcp.svg?label=ferric-fred-mcp)](https://crates.io/crates/ferric-fred-mcp)

Consumers depend on the library **by workspace path**, so a breaking change in
the library cannot compile-pass its consumers without updating them — that
compile-time coupling is the primary "stay in sync" guarantee (versions are
managed on top; see the ADRs).

## What it covers

The library wraps **all of FRED's read endpoints** — series and observations
(including ALFRED point-in-time / vintage data via a real-time window),
search, categories, releases (including the nested release-table tree, with
optional inline observation values), sources,
and tags — plus the **GeoFRED / Maps API** (regional data and the geographic
shape files to map it, [ADR-0025](docs/adr/0025-geofred-maps-api.md)) — behind
ergonomic builders, with newtype identifiers, typed enums for
FRED's closed value sets, a non-panicking error taxonomy, and **auto-pagination**
(`Paginate::send_all` walks an endpoint to exhaustion, `Paginate::stream` yields
lazily; `--all` on the CLI). See [ADR-0020](docs/adr/0020-auto-pagination.md) and
[ADR-0021](docs/adr/0021-streaming-pagination.md).

GeoFRED support spans the **library**, **CLI** (`fred geofred`), and **MCP**
(`get_regional_data`, `get_series_data`, `get_series_group`) layers. The one
exception is the geographic `shapes/file` endpoint, which is library/CLI-only —
a large projected-GeoJSON blob is poor ergonomics for an MCP tool caller
([ADR-0025](docs/adr/0025-geofred-maps-api.md)).

Pick an entry point:

- **Library** — `cargo add ferric-fred`; typed async access from your own code.
  See the [crate README](crates/ferric-fred/README.md) and
  [docs.rs](https://docs.rs/ferric-fred).
- **CLI** (`fred`) — `cargo install ferric-fred-cli`; search, show metadata,
  print or **chart** observations in the terminal, browse categories,
  releases, sources, and tags, and pull **GeoFRED** regional data and map shapes
  (`fred geofred`). See the [crate README](crates/ferric-fred-cli/README.md)
  or `fred <command> --help`.
- **MCP server** (`fred-mcp`) — `cargo install ferric-fred-mcp`; **34 tools** over
  stdio covering the same read surface, for MCP-capable clients ([ADR-0010](docs/adr/0010-mcp-server-design.md)).
  Each tool declares input and output schemas plus behavioural annotations
  ([ADR-0023](docs/adr/0023-mcp-output-schemas.md)). See the
  [crate README](crates/ferric-fred-mcp/README.md).

The MCP server is listed and scored on
[Glama](https://glama.ai/mcp/servers/ojhermann-org/ferric-fred):

[![ferric-fred MCP server](https://glama.ai/mcp/servers/ojhermann-org/ferric-fred/badges/card.svg)](https://glama.ai/mcp/servers/ojhermann-org/ferric-fred)

## Development

A Nix flake provides a reproducible toolchain (`nix develop`, or `direnv allow`
once), but the project builds with a plain Rust toolchain too — Nix supplies the
environment, not the build ([ADR-0008](docs/adr/0008-nix-flake-dev-environment.md)).

Contributor setup, the fmt/clippy/test **gate**, the tracked git hooks, and the
workflow for adding an endpoint live in **[CONTRIBUTING.md](CONTRIBUTING.md)**.
CI (`ci.yml`) runs that same offline gate on every push and PR; a dormant
`live.yml` runs the live FRED tests once an Infisical machine identity is
configured ([ADR-0016](docs/adr/0016-ci-live-tests-machine-identity.md)).

### Benchmarks

Performance tooling from the org Tech Radar pilot
([ADR-0026](docs/adr/0026-perf-tooling-pilot.md), issue #42):

```sh
# Deserialization microbenches (divan) — the observations parse hot path.
cargo bench -p ferric-fred --bench deserialization
# Same workload under criterion (the divan-vs-criterion baseline).
cargo bench -p ferric-fred --bench deserialization_criterion

# CLI wall-clock timing (hyperfine): startup + a live fetch-and-render.
# The fetch benchmark needs FRED_API_KEY; startup runs offline.
scripts/bench-cli.sh                    # add --json DIR to export hyperfine JSON
```

CI keeps the benches compiling (`cargo bench --no-run`) but does not time them;
tracking results over time is the deferred bencher.dev step in ADR-0026.

## Secrets

The client reads a free **FRED API key** from the `FRED_API_KEY` environment
variable (get one at <https://fredaccount.stlouisfed.org/apikeys>). Locally,
secrets are injected via [Infisical](https://infisical.com) + direnv
([ADR-0009](docs/adr/0009-secret-management-infisical-direnv.md)):

```sh
cp .envrc.example .envrc     # local, git-ignored entry point
infisical login             # user auth (opens a browser)
infisical init              # link this dir → project
direnv allow                # load the shell + inject secrets on cd-in
```

Store the key with `infisical secrets set FRED_API_KEY="…" --env=dev --path=/shared`.
No Infisical? Just set it directly in your git-ignored `.envrc`:
`export FRED_API_KEY="…"` — the library only reads the env var and has no
dependency on Infisical.

## Architecture decisions

Design decisions are recorded as ADRs in [`docs/adr/`](docs/adr/). Start with
[the index](docs/adr/README.md).

## License

Dual-licensed under **MIT OR Apache-2.0**, at your option — the Rust ecosystem
default ([ADR-0006](docs/adr/0006-license.md)). See [`LICENSE-MIT`](LICENSE-MIT)
and [`LICENSE-APACHE`](LICENSE-APACHE). Unless you state otherwise, any
contribution you submit is licensed under the same dual terms (see
[`CONTRIBUTING.md`](CONTRIBUTING.md)).

This covers *our code*; FRED data itself is subject to the St. Louis Fed's terms
of use, and you supply your own API key — the project ships no data and no key.
