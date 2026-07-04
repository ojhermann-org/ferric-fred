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

Early construction. The library comes first; the CLI and MCP server follow.

## Architecture decisions

Design decisions are recorded as ADRs in [`docs/adr/`](docs/adr/). Start with
[the index](docs/adr/README.md).

## License

TBD (see ADR backlog).
