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

Early construction. The library covers the `series`, `series/observations`, and
`series/search` endpoints; the `fred` CLI (this repo's first consumer) can
search, show series metadata, and print observations. The MCP server follows.

## Using the CLI

The `fred` binary reads `FRED_API_KEY` from the environment (see [Secrets](#secrets)):

```sh
fred search "unemployment rate" --limit 3   # find series by text
fred series GNPCA                            # show one series' metadata
fred observations GNPCA --start 2020-01-01   # print observations (--end, --limit too)
```

Run it from the workspace with `cargo run -p ferric-fred-cli -- <args>`.

## Development

A Nix flake provides a reproducible toolchain (recent stable Rust via
`oxalica/rust-overlay`, plus `cargo-nextest`, `cargo-deny`, and `bacon`):

```sh
nix develop        # enter the dev shell
# or, with direnv: `direnv allow` once, then it loads automatically
```

Nix is optional — the project builds with a normal Rust toolchain too. Install
a recent stable Rust (e.g. via `rustup`) and use `cargo` as usual. Either way,
building is plain `cargo build` / `cargo test`; Nix supplies the environment,
not the build (see [ADR-0008](docs/adr/0008-nix-flake-dev-environment.md)).

### Pre-push checks

A tracked `pre-push` hook runs formatting, clippy, and the offline test suite —
the same gate as CI — before a push, and blocks on failure. Enable it once per
clone (`core.hooksPath` is local git config, not carried by git):

```sh
git config core.hooksPath .githooks
```

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
