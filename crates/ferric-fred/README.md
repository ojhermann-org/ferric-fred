# ferric-fred

[![Crates.io](https://img.shields.io/crates/v/ferric-fred.svg)](https://crates.io/crates/ferric-fred)
[![Docs.rs](https://docs.rs/ferric-fred/badge.svg)](https://docs.rs/ferric-fred)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/ferric-fred.svg)](#license)

A strongly-typed async Rust client for [FRED](https://fred.stlouisfed.org/) —
Federal Reserve Economic Data, from the Federal Reserve Bank of St. Louis.

> `ferric` (iron oxide → *rust*) + `FRED`. Iron-clad, typed access to economic data.

## Highlights

- **Async, typed, and complete** — covers all of FRED's read endpoints (series,
  observations, search, categories, releases, sources, tags, and the release
  table tree) behind ergonomic builders.
- **Strong domain modelling** — newtype identifiers (`SeriesId`, `CategoryId`,
  `ReleaseId`, `SourceId`), typed enums for FRED's closed value sets (units,
  frequency, ordering, …), and a typed error taxonomy that never panics on a
  network or parse failure.
- **Auto-pagination** — `Paginate::send_all` walks a paginated endpoint to
  exhaustion; `Paginate::stream` yields results lazily as a `Stream`.
- **rustls by default** — no system OpenSSL build dependency.

## Install

```sh
cargo add ferric-fred
```

The client reads a free [FRED API key](https://fredaccount.stlouisfed.org/apikeys)
from the `FRED_API_KEY` environment variable (or pass one to `Client::new`). It
needs an async runtime such as [tokio](https://tokio.rs/).

## Example

```rust,no_run
use ferric_fred::{Client, Paginate, SeriesId};

#[tokio::main]
async fn main() -> ferric_fred::Result<()> {
    let client = Client::from_env()?; // reads FRED_API_KEY

    // One series' observations:
    let observations = client.observations(&SeriesId::new("GNPCA")).send().await?;
    println!("{} observations", observations.len());

    // Search, paged to exhaustion (or `.stream()` for lazy iteration):
    let matches = client.search("unemployment rate").send_all().await?;
    println!("{} matching series", matches.len());

    Ok(())
}
```

## Related crates

- [`ferric-fred-cli`](https://crates.io/crates/ferric-fred-cli) — the `fred`
  command-line tool, with interactive terminal charts.
- [`ferric-fred-mcp`](https://crates.io/crates/ferric-fred-mcp) — an MCP server
  exposing FRED to MCP-capable clients.

## Documentation & source

- API docs: [docs.rs/ferric-fred](https://docs.rs/ferric-fred)
- Repository, examples, and design ADRs:
  [github.com/ojhermann-org/ferric-fred](https://github.com/ojhermann-org/ferric-fred)

## License

Dual-licensed under **MIT OR Apache-2.0**, at your option — the Rust ecosystem
default. FRED data itself is subject to the St. Louis Fed's terms of use; you
supply your own API key, and this crate ships no data and no key.
