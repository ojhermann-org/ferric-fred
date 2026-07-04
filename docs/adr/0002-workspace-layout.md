# ADR-0002: Workspace layout & crate boundaries

- **Status:** Accepted
- **Date:** 2026-07-04
- **Deciders:** Project owner

## Context

The project comprises three deliverables — a typed FRED library, a CLI with TUI
charts, and an MCP server — that share types and a client. We want the consumers
to stay in lockstep with the library at the type level, and we want a release
process that can version them coherently.

## Decision

We will use a **single Cargo workspace** with three member crates:

| Crate | Kind | Depends on |
|-------|------|-----------|
| `ferric-fred` | library (`lib.rs`) | — (the foundation) |
| `ferric-fred-cli` | binary | `ferric-fred` (by path) |
| `ferric-fred-mcp` | binary | `ferric-fred` (by path) |

- The library depends on **neither** binary crate. Dependencies point one way,
  toward the library.
- Consumers depend on the library **by workspace path**, so a breaking library
  change cannot compile-pass its consumers without updating them. This
  compile-time coupling is our primary "stay in sync" guarantee; version numbers
  (a later ADR) ride on top of it.
- Shared dependency versions and lints are declared once via
  `[workspace.dependencies]` and `[workspace.lints]` and inherited by members.
- The CLI binary name (e.g. `fred` vs `ferric-fred`) is deferred to a small
  follow-up decision; it does not affect layout.

We build the library first; the binary crates are added when we reach them.

## Consequences

- One `Cargo.lock`, one `cargo test`/`clippy` invocation covers everything.
- Refactoring library types surfaces breakage in consumers immediately.
- Slightly more ceremony than a single crate, justified by the three distinct
  deliverables and their release story.

## Alternatives considered

- **Separate repositories per crate** — maximal independence, but loses the
  compile-time sync guarantee and multiplies CI/release wiring. Rejected; the
  whole point is that these move together.
- **One crate with feature flags** (`cli`, `mcp` features) — fewer moving parts,
  but muddies the dependency graph, forces binary deps into library builds, and
  complicates per-artifact versioning.
