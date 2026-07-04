# ADR-0004: Error-handling strategy

- **Status:** Accepted
- **Date:** 2026-07-04
- **Deciders:** Project owner

## Context

The library must report failures precisely enough that callers (the CLI, the MCP
server, and third parties) can branch on them — a transient network error, an
authentication failure, a FRED API error with a specific code, a bad
deserialization, or invalid caller input are meaningfully different. The
binaries, by contrast, mostly want to bubble errors to the top with context.

## Decision

**Library** returns `Result<T, ferric_fred::Error>`, where `Error` is a
`thiserror`-derived enum. Anticipated variants:

- `Transport(reqwest::Error)` — connection/timeout/TLS failures.
- `Api { status: u16, code: Option<u32>, message: String }` — FRED returned an
  error payload (FRED encodes errors in the body with a code + message).
- `Deserialize(...)` — response didn't match the expected shape.
- `InvalidInput(...)` — caller-side validation (e.g. malformed series id) caught
  before a request is made.
- `RateLimited { retry_after: Option<Duration> }` — surfaced distinctly so
  callers can back off.

Rules:

- The library never panics on network or parse failure — those are `Err`, not
  `panic!`.
- `Error` is `#[non_exhaustive]` so we can add variants without a breaking
  change.

**Binaries** (`ferric-fred-cli`, `ferric-fred-mcp`) use `anyhow` at the
top level for ergonomic context-adding (`.context(...)`) and reporting, while
still being able to `downcast` to `ferric_fred::Error` when they need to branch.

## Consequences

- Callers get a precise, matchable error taxonomy; `#[non_exhaustive]` keeps it
  evolvable.
- Two error idioms in the codebase (typed in the lib, `anyhow` in bins), which
  is the conventional Rust split and worth the minor inconsistency.

## Alternatives considered

- **`anyhow` everywhere, including the library** — easiest, but erases the typed
  error taxonomy that library consumers need to branch on. Rejected for the lib.
- **`Box<dyn Error>`** — avoids a dependency but loses matchability and a stable
  variant surface. Rejected.
