# Architecture Decision Records

This directory records significant design and process decisions for
`ferric-fred` as [ADRs](https://adr.github.io/). See
[ADR-0001](0001-record-architecture-decisions.md) for what an ADR is and how we
use them, and [`0000-adr-template.md`](0000-adr-template.md) for the template.

## Index

| # | Title | Status |
|---|-------|--------|
| [0001](0001-record-architecture-decisions.md) | Record architecture decisions | Accepted |
| [0002](0002-workspace-layout.md) | Workspace layout & crate boundaries | Accepted |
| [0003](0003-async-runtime-and-http-client.md) | Async runtime & HTTP client | Accepted |
| [0004](0004-error-handling.md) | Error-handling strategy | Accepted |
| [0005](0005-domain-modelling-and-strong-typing.md) | Domain modelling & strong typing | Accepted |
| [0006](0006-license.md) | License | Accepted |
| [0007](0007-rust-edition-and-msrv.md) | Rust edition & MSRV policy | Accepted |
| [0008](0008-nix-flake-dev-environment.md) | Nix flake for the development environment | Accepted |

## Backlog (proposed, not yet written)

Decisions we intend to record, roughly in the order we expect to need them.
Order and contents will change as we build.

- CLI binary naming (is the CLI binary `fred`?) — small follow-up to ADR-0002
- API-key & configuration handling (env var, config file, precedence)
- Testing strategy (unit, HTTP mocking, recorded fixtures)
- Versioning & release strategy (independent semver, `release-plz`,
  conventional commits) — needed once we have more than one crate
