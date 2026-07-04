# ADR-0003: Async runtime & HTTP client

- **Status:** Accepted
- **Date:** 2026-07-04
- **Deciders:** Project owner

## Context

The library makes HTTP calls to the FRED REST API. It underpins an MCP server
(inherently async) and a CLI. We need a transport strategy that fits both
without doubling the maintained surface area.

## Decision

We will build an **async-first** library on **`tokio` + `reqwest`**.

- The public client API is async (`async fn`). No blocking API in v1.
- `reqwest` is configured with **`rustls-tls`** (not native-tls) so there is no
  system OpenSSL build dependency — better portability and simpler CI.
- The CLI drives the async client with a `#[tokio::main]` entry point; the MCP
  server is async natively.
- We depend on `tokio` with only the features we need rather than `full`.

## Consequences

- One code path to maintain and test; natural fit for the async MCP server.
- Consumers of the library must provide an async runtime. This is the idiomatic
  expectation for a modern Rust HTTP client, so we accept it.
- A synchronous convenience wrapper (behind a `blocking` feature) remains
  possible later without breaking the async API, if demand appears. Explicitly
  out of scope now.

## Alternatives considered

- **Async + optional `blocking` feature** — nicer for quick scripts, but ~2×
  the surface to build, document, and test on day one. Deferred, not adopted.
- **Blocking-only (`ureq`)** — simplest for a CLI, but a poor fit for the async
  MCP server and less idiomatic as a library. Rejected.
