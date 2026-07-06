# ferric-fred-mcp

[![Crates.io](https://img.shields.io/crates/v/ferric-fred-mcp.svg)](https://crates.io/crates/ferric-fred-mcp)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/ferric-fred-mcp.svg)](#license)
[![ferric-fred MCP server](https://glama.ai/mcp/servers/ojhermann-org/ferric-fred/badges/score.svg)](https://glama.ai/mcp/servers/ojhermann-org/ferric-fred)

`fred-mcp` — a [Model Context Protocol](https://modelcontextprotocol.io/) server
that exposes [FRED](https://fred.stlouisfed.org/) (Federal Reserve Economic Data)
to MCP-capable clients over stdio, built on the
[`ferric-fred`](https://crates.io/crates/ferric-fred) client.

It provides **31 tools** covering all of FRED's read endpoints — searching and
inspecting series, fetching observations, and browsing categories, releases,
sources, and tags. Tool results are returned as JSON (MCP structured content),
and every tool declares an input **and output** schema plus behavioural
annotations (read-only, idempotent, non-destructive, open-world), so a client
knows a call's shape and effects before making it.

## Install

```sh
cargo install ferric-fred-mcp   # installs the `fred-mcp` binary
```

## Configure your MCP client

Point your client at the binary and give it a free
[FRED API key](https://fredaccount.stlouisfed.org/apikeys) via `FRED_API_KEY`:

```json
{
  "mcpServers": {
    "fred": {
      "command": "fred-mcp",
      "env": { "FRED_API_KEY": "your-fred-api-key" }
    }
  }
}
```

(Use an absolute path to the binary if `fred-mcp` isn't on the client's `PATH`.)

## Run in a container

A [`Dockerfile`](https://github.com/ojhermann-org/ferric-fred/blob/main/Dockerfile)
at the repo root builds a small image that runs `fred-mcp` over stdio. Pass your
key at run time (the server speaks MCP on stdin/stdout, so keep it attached):

```sh
docker build -t ferric-fred-mcp .
docker run -i -e FRED_API_KEY=your-fred-api-key ferric-fred-mcp
```

## Tools

A few of the 31 tools:

| Tool | Purpose |
|------|---------|
| `search_series` | Find series by text (ordering, sort, limit) |
| `get_series` | Metadata for a series id |
| `get_observations` | A series' observations (date range, units transform, aggregation, sort, limit) |
| `get_category_series` | The series in a category |
| `get_release_series` | The series in a release |
| `get_tags_series` | Series carrying all of a set of tags |

The full tool list, with parameters, is in the
[repository README](https://github.com/ojhermann-org/ferric-fred#using-the-mcp-server).

## Documentation & source

[github.com/ojhermann-org/ferric-fred](https://github.com/ojhermann-org/ferric-fred)
— full tool reference and design ADRs (see
[ADR-0010](https://github.com/ojhermann-org/ferric-fred/blob/main/docs/adr/0010-mcp-server-design.md)
for the server design).

## Listed on Glama

[![ferric-fred MCP server](https://glama.ai/mcp/servers/ojhermann-org/ferric-fred/badges/card.svg)](https://glama.ai/mcp/servers/ojhermann-org/ferric-fred)

## MCP registry

Published to the [official MCP registry](https://github.com/modelcontextprotocol/registry).
The token below is how the registry verifies that this crate and its registry
entry share an owner (it is matched against the crate's published README):

mcp-name: io.github.ojhermann-org/ferric-fred-mcp

## License

Dual-licensed under **MIT OR Apache-2.0**, at your option. FRED data itself is
subject to the St. Louis Fed's terms of use; you supply your own API key.
