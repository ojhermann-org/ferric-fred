# ferric-fred

A strongly-typed Rust client for [FRED](https://fred.stlouisfed.org/) — the
Federal Reserve Economic Data service from the Federal Reserve Bank of St. Louis
— plus a CLI (with TUI charts) and an MCP server built on top of it.

> `ferric` (iron oxide → *rust*) + `FRED`. Iron-clad, typed access to economic data.

## Workspace layout

This is a Cargo workspace. Planned crates:

| Crate | Kind | Purpose |
|-------|------|---------|
| `ferric-fred` | library | Strongly-typed async client for the FRED API |
| `ferric-fred-cli` | binary | Command-line tool with `ratatui` TUI charts |
| `ferric-fred-mcp` | binary | MCP server exposing FRED to MCP clients |

Consumers depend on the library **by workspace path**, so a breaking change in
the library cannot compile-pass its consumers without updating them — that
compile-time coupling is our primary "stay in sync" guarantee. Version numbers
are managed on top of that (see the ADRs).

## Status

Early construction. The library covers the `series` endpoints (`series`,
`series/observations`, `series/search`, `series/updates`, `series/vintagedates`,
`series/categories`, `series/release`, `series/tags`), `category` (`category`,
`category/children`, `category/series`),
`release` (`releases`, `releases/dates`, `release`, `release/series`,
`release/sources`, `release/dates`), `source` (`sources`, `source`,
`source/releases`), and `tag` (`tags`, `related_tags`, `tags/series`);
the `fred` CLI (this repo's first consumer) can search, show series metadata,
print observations, chart them in an interactive terminal UI, browse the
category tree, releases, and sources, and filter series by tags (with
related-tag discovery). The `fred-mcp` server (ADR-0010) speaks MCP over stdio
with the corresponding tools.

## Using the CLI

The `fred` binary reads `FRED_API_KEY` from the environment (see [Secrets](#secrets)):

```sh
fred search "unemployment rate" --order-by popularity --limit 3  # find series by text
fred series GNPCA                                                 # show one series' metadata
fred observations GDP --units pch --sort desc --limit 4          # transformed observations
fred observations GDP --frequency annual --aggregation avg       # aggregate to a lower frequency
fred chart GNPCA --start 1950-01-01                              # interactive terminal chart
fred category                                                    # browse the category tree (root)
fred category 13                                                 # a category and its children
fred category 125 --series --limit 5                            # series in a category
fred release                                                     # list all data releases
fred release 53                                                  # a release's metadata
fred release 53 --series --limit 5                              # series in a release
fred release 53 --sources                                       # sources a release draws from
fred release --dates --limit 5                                  # release calendar (all releases)
fred release 53 --dates                                         # one release's publication dates
fred source                                                     # list all data sources
fred source 18                                                  # a source's metadata
fred source 18 --releases --limit 5                            # releases produced by a source
fred updates --filter macro --limit 10                          # recently updated series
fred tags --search-text quarterly --limit 5                     # browse/search the tag vocabulary
fred tags gdp quarterly --limit 5                               # series carrying all these tags
fred tags gdp --related --limit 5                               # tags that co-occur with gdp
fred series GNPCA --tags                                         # a series' own tags
fred series GNPCA --categories                                   # the categories a series is in
fred series GNPCA --release                                      # the release a series belongs to
fred series GNPCA --vintages                                      # the dates a series was revised
fred series GNPCA --json | jq .frequency                        # JSON output for scripting
```

Add `--json` to any data command (`search`, `series`, `observations`) for
machine-readable output — each emits its domain type as JSON (`chart` ignores
it). `fred chart` opens an interactive [ratatui](https://ratatui.rs/) line chart
of a series' observations (it accepts the same flags as `observations`); press
`q`, `Esc`, or `Ctrl-C` to quit.

Run it from the workspace with `cargo run -p ferric-fred-cli -- <args>`, and see
`fred <command> --help` for every flag (`--units`, `--order-by`, … accept the
FRED value sets).

## Using the MCP server

`fred-mcp` is an [MCP](https://modelcontextprotocol.io/) server (ADR-0010) that
exposes FRED to MCP-capable clients over stdio. It reads `FRED_API_KEY` from the
environment and provides twenty-three tools:

| Tool | Purpose |
|------|---------|
| `search_series` | Find series by text (with ordering, sort, limit) |
| `get_series` | Metadata for a series id |
| `get_observations` | A series' observations (date range, units transform, frequency aggregation, sort, limit) |
| `get_series_updates` | Series updated most recently (with class filter, limit) |
| `get_series_vintagedates` | The dates a series was revised (with sort, limit) |
| `get_series_categories` | The categories a series belongs to |
| `get_series_release` | The release a series belongs to |
| `get_category` | A category's name and parent (id 0 is the tree root) |
| `get_category_children` | The child categories of a category (walk the tree) |
| `get_category_series` | The series in a category (with ordering, sort, limit) |
| `get_releases` | List all data releases (with sort, limit) |
| `get_releases_dates` | Release calendar across all releases (with sort, limit, include-no-data) |
| `get_release` | A release's name, press-release flag, and link |
| `get_release_series` | The series in a release (with ordering, sort, limit) |
| `get_release_sources` | The sources a release draws from |
| `get_release_dates` | One release's publication dates (with sort, limit, include-no-data) |
| `get_sources` | List all data sources (with sort, limit) |
| `get_source` | A source's name and link |
| `get_source_releases` | The releases produced by a source (with sort, limit) |
| `get_tags` | Browse/search the tag vocabulary (with search text, sort, limit) |
| `get_related_tags` | Tags co-occurring with a seed set (with search text, sort, limit) |
| `get_tags_series` | Series carrying all of a set of tags (with ordering, sort, limit) |
| `get_series_tags` | A series' own tags |

Tool results are returned as JSON (MCP structured content). Build the binary,
then point your MCP client at it:

```sh
cargo build --release -p ferric-fred-mcp   # -> target/release/fred-mcp
```

```json
{
  "mcpServers": {
    "fred": {
      "command": "/path/to/ferric-fred/target/release/fred-mcp",
      "env": { "FRED_API_KEY": "your-fred-api-key" }
    }
  }
}
```

## Development

A Nix flake provides a reproducible toolchain (recent stable Rust via
`oxalica/rust-overlay`, plus `cargo-nextest`, `cargo-deny`, `bacon`, and
`gitleaks`):

```sh
nix develop        # enter the dev shell
# or, with direnv: `direnv allow` once, then it loads automatically
```

Nix is optional — the project builds with a normal Rust toolchain too. Install
a recent stable Rust (e.g. via `rustup`) and use `cargo` as usual. Either way,
building is plain `cargo build` / `cargo test`; Nix supplies the environment,
not the build (see [ADR-0008](docs/adr/0008-nix-flake-dev-environment.md)).

### Git hooks

Two tracked hooks live in `.githooks/`:

- **`pre-commit`** — a secret guard ([ADR-0014](docs/adr/0014-pre-commit-secret-guard.md)):
  it blocks staged secret files (`.envrc`, `.env*`) and scans the staged diff
  with [`gitleaks`](https://github.com/gitleaks/gitleaks) for pasted keys.
- **`pre-push`** — runs formatting, clippy, and the offline test suite (the same
  gate as CI) and blocks on failure.

Enable them once per clone (`core.hooksPath` is local git config, not carried by
git):

```sh
git config core.hooksPath .githooks
```

### Continuous integration

`ci.yml` runs the offline gate (fmt, clippy, tests, doctests, `cargo deny`) on
every push and PR, through the same flake as local dev. A separate, **dormant**
`live.yml` runs the `#[ignore]` live FRED tests nightly (and on demand) — it
stays a green no-op until an Infisical machine identity is configured via the
`INFISICAL_CLIENT_ID` / `INFISICAL_CLIENT_SECRET` repository secrets (see
[ADR-0016](docs/adr/0016-ci-live-tests-machine-identity.md)).

## Secrets

The client reads a **FRED API key** from the `FRED_API_KEY` environment
variable. Get a free key at
<https://fredaccount.stlouisfed.org/apikeys>.

Secrets are injected via [Infisical](https://infisical.com) + direnv (see
[ADR-0009](docs/adr/0009-secret-management-infisical-direnv.md)). One-time setup:

```sh
cp .envrc.example .envrc     # local, git-ignored entry point
infisical login             # user auth (opens a browser)
infisical init              # link this dir → project, writes .infisical.json
direnv allow                # load the shell + inject secrets on cd-in
```

Store the key with `infisical secrets set FRED_API_KEY="…" --env=dev --path=/shared`.
No Infisical? Just set it directly in your git-ignored `.envrc`:
`export FRED_API_KEY="…"`. The library only reads the env var — it has no
dependency on Infisical.

## Architecture decisions

Design decisions are recorded as ADRs in [`docs/adr/`](docs/adr/). Start with
[the index](docs/adr/README.md).

## License

TBD (see ADR backlog).
