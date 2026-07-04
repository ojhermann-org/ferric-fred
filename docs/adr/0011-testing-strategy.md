# ADR-0011: Testing strategy

- **Status:** Accepted <!-- Proposed | Accepted | Deprecated | Superseded by ADR-XXXX -->
- **Date:** 2026-07-04
- **Deciders:** Project owner

## Context

The library's three endpoints are currently exercised only by `#[ignore]` live
tests that hit the real FRED API and require `FRED_API_KEY` plus network. CI
can't run them (no machine identity yet — deferred in
[ADR-0009](0009-secret-management-infisical-direnv.md)), so request-building,
response parsing, and especially the error-mapping paths (`Error::Api`,
`Error::RateLimited`, `Error::Deserialize`) have no automated coverage. Some of
those paths — HTTP 429, or a 400 with a FRED error body — are also awkward to
trigger against live FRED on demand. We want deterministic, offline coverage
that runs in CI, without giving up the live smoke tests that check FRED's actual
behaviour.

## Decision

We will test in layers, each with a clear job:

1. **Unit tests** — pure logic, colocated in `#[cfg(test)]` modules: value/enum
   (de)serialization, query-code mappings, `query_params()` output. No I/O.
2. **HTTP-mocked integration tests** — the client driven against a local
   **`wiremock`** server, offline and CI-run. Cover each endpoint's happy path
   plus the error mapping (`Api`, `RateLimited`, `Deserialize`), and assert that
   requests carry `api_key`/`file_type` and the expected query parameters. The
   client points at the mock through a `#[cfg(test)] pub(crate)` base-URL
   constructor — a test seam, not public API.
3. **Live tests** — `#[ignore]`, hit real FRED, require `FRED_API_KEY`. Run
   locally / in the dev shell (and in CI once a machine identity lands, per
   ADR-0009). They stay small: their job is to confirm our types still match
   FRED's live payloads.
4. **CLI & server tests** — `assert_cmd` drives the built `fred` binary for arg
   parsing, validation, and error output (offline), with `#[ignore]` live
   happy-paths; the MCP server is verified by driving its stdio JSON-RPC
   handshake.

Canned FRED JSON lives inline in the tests for now; recorded fixture files are
deferred until payloads grow enough to warrant them.

## Consequences

- CI gains real coverage of the client's request/response/error handling,
  including paths (429, 400-with-body, malformed JSON) that are impractical to
  trigger live.
- One dev-dependency (`wiremock`) and a `#[cfg(test)]` seam on `Client`; no
  change to the shipped public API.
- Live tests remain the authority on FRED's actual behaviour — the mocks encode
  our *understanding* of the wire format, which the live tests periodically
  re-validate.
- Some overlap between the `query_params()` unit tests and the mocked
  query-parameter assertions; accepted, as they verify different layers
  (serialization vs. the request actually sent over the wire).

## Alternatives considered

- **`mockito` / `httpmock`** — both capable; `wiremock` chosen for its
  async-first ergonomics with `reqwest`/`tokio`.
- **Recorded fixtures / VCR-style playback** — convenient for large payloads,
  but heavier machinery than inline JSON needs today. Deferred.
- **A public base-URL override** for testability — rejected; keeps the public
  API minimal (ADR-0005). The override stays `pub(crate)` / `#[cfg(test)]`.
- **Live-only (status quo)** — leaves error paths uncovered and cannot run in
  CI. Rejected.
