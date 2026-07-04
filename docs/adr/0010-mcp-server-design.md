# ADR-0010: MCP server design

- **Status:** Accepted <!-- Proposed | Accepted | Deprecated | Superseded by ADR-XXXX -->
- **Date:** 2026-07-04
- **Deciders:** Project owner

## Context

`ferric-fred-mcp` is the third planned crate ([ADR-0002](0002-workspace-layout.md)):
an [MCP](https://modelcontextprotocol.io/) server that exposes FRED to
MCP-capable clients (Claude Desktop/Code and others). The library and CLI
already cover three FRED endpoints — `series`, `series/observations`, and
`series/search` — behind a typed async API ([ADR-0003](0003-async-runtime-and-http-client.md),
[ADR-0005](0005-domain-modelling-and-strong-typing.md)). The server is a third
consumer of that library and needs four things: a protocol implementation, a
transport, a mapping from FRED endpoints to MCP tools, and a way to render
results for an LLM. We want to reuse the library's typing rather than restate
FRED's shape at the protocol layer.

## Decision

We will build `ferric-fred-mcp` as an **async stdio MCP server on the official
`rmcp` SDK**, exposing the library's three endpoints as typed tools.

- **SDK: `rmcp`** (the official Rust MCP SDK). It is `tokio`-native — a direct
  fit for our async client (ADR-0003) — and derives tool JSON schemas from typed
  input structs, extending the strong-typing posture (ADR-0005) to the protocol
  boundary.
- **Transport: stdio** for v1 — the transport local MCP clients use to launch a
  server. HTTP/SSE is deferred, not precluded.
- **Tools, one per endpoint:**
  - `search_series` — text search; parameters mirror the search builder
    (order-by, sort, limit).
  - `get_series` — metadata for a single series id.
  - `get_observations` — a series' observations; parameters mirror the
    observations builder (date range, units, frequency/aggregation, sort, limit).

  Tool input structs derive `serde::Deserialize` + `schemars::JsonSchema` and
  reuse the library's typed enums where practical.
- **Results are structured JSON.** Tools return the library's domain types
  serialized to JSON, so we will add `serde::Serialize` to the currently
  deserialize-only types (`Series`, `Observation`, and the metadata enums). This
  same change unblocks the CLI's future `--json` mode — one addition, two
  consumers.
- **Errors and secrets follow the existing binaries:** `anyhow` at the boundary
  ([ADR-0004](0004-error-handling.md)), mapping `ferric_fred::Error` into MCP
  tool errors; the API key via `Client::from_env` (`FRED_API_KEY`,
  [ADR-0009](0009-secret-management-infisical-direnv.md)).
- **Binary name: `fred-mcp`**, mirroring the CLI's `fred`.

## Consequences

- A third independent consumer re-validates the library API, as the CLI did —
  surfacing any ergonomic or typing gaps before they harden.
- The library gains `Serialize` on its domain types. This widens the public API
  (a surface we then keep stable), but it is a natural, low-risk addition shared
  with the CLI.
- `rmcp` is young and pre-1.0; its API may churn. We pin a specific version and
  accept occasional upgrade work. The endpoint→tool mapping is SDK-agnostic, so a
  future switch is contained.
- stdio-only means no remote or multi-client hosting in v1 — acceptable, since
  the primary target is a locally launched server.
- `schemars` joins the dependency tree (MCP crate only), gated by cargo-deny like
  everything else.

## Alternatives considered

- **Hand-rolled JSON-RPC over stdio** — maximal control and no SDK churn, but
  re-implements MCP framing and lifecycle we would rather not own. Rejected for
  v1; the portable tool mapping keeps this open as a fallback.
- **A third-party MCP crate** (e.g. `mcp-sdk`, `mcpr`) — workable, but the
  official `rmcp` is the safer long-term bet for protocol conformance.
- **Human-readable text results** instead of JSON — simpler and needs no
  `Serialize`, but structured JSON is better for programmatic tool use and
  composes with the CLI `--json` work. Chose JSON.
- **HTTP/SSE transport in v1** — needed for remote hosting, but adds surface
  (auth, connection lifecycle) for no immediate benefit. Deferred.
