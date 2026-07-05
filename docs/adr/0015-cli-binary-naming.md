# ADR-0015: CLI & server binary names

- **Status:** Accepted <!-- Proposed | Accepted | Deprecated | Superseded by ADR-XXXX -->
- **Date:** 2026-07-05
- **Deciders:** Project owner

## Context

[ADR-0002](0002-workspace-layout.md) fixed the crate names —
`ferric-fred` (library), `ferric-fred-cli`, `ferric-fred-mcp` — but explicitly
deferred the *binary* name ("`fred` vs `ferric-fred`") as "a small follow-up
decision [that] does not affect layout." When the two binary crates were built
out, each set a `[[bin]] name` decoupled from its package name: `fred` for the
CLI and `fred-mcp` for the MCP server. This ADR resolves that deferral and
records why, so the choice is a decision rather than an accident of
implementation.

The binary name is the user-facing command (`fred series GNPCA`) and the path an
MCP client is pointed at; the crate name is the workspace/registry identifier.
They need not match, and Cargo's `[[bin]] name` lets them differ.

## Decision

We will name the binaries **`fred`** (from `ferric-fred-cli`) and **`fred-mcp`**
(from `ferric-fred-mcp`), decoupled from their package names via `[[bin]] name`.

- **`fred`** — short, matches the domain (FRED), and reads naturally as an
  interactive command. The `ferric-` prefix is a crate-namespacing device
  (workspace/crates.io), not something a user should type every invocation.
- **`fred-mcp`** — the same stem with an `-mcp` suffix, marking it as the MCP
  stdio server rather than the interactive CLI. Two separate binaries (not one
  multiplexed `fred mcp`) keep the interactive tool and the machine-facing server
  cleanly separable, each with its own lifecycle and a stable path for MCP
  clients to invoke.

## Consequences

- The command is terse and memorable; `cargo install ferric-fred-cli` installs a
  `fred` binary, and the README states this so the crate-name/binary-name gap
  doesn't surprise.
- Two thin binary crates rather than one, consistent with ADR-0002's boundaries
  and with [ADR-0010](0010-mcp-server-design.md)'s separate MCP server.
- `fred` is a short, common word, so a name collision on a user's `PATH` is
  possible. Accepted while the tool is unpublished; the crates.io release
  (ADR-0012) is the natural point to revisit if a clash surfaces — the crate
  names are already unambiguous, so only the installed command would change.

## Alternatives considered

- **`ferric-fred` as the binary** — matches the crate and project name and is
  unambiguous, but long to type for a CLI used interactively. Rejected for
  ergonomics; the crate name already carries the unambiguous identifier.
- **`ff`** — very short, but cryptic and at high risk of colliding with other
  tools. Rejected.
- **One multiplexed binary** (e.g. `fred mcp` launching the server) — rejected:
  the MCP server is a long-lived stdio JSON-RPC endpoint with a different
  lifecycle from the interactive CLI; a distinct `fred-mcp` keeps the two
  concerns and their invocation paths separate.
