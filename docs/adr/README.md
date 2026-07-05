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
| [0008](0008-nix-flake-dev-environment.md) | Nix flake for the development environment | Accepted (direnv wiring refined by 0009) |
| [0009](0009-secret-management-infisical-direnv.md) | Secret management via Infisical + direnv | Accepted |
| [0010](0010-mcp-server-design.md) | MCP server design | Accepted |
| [0011](0011-testing-strategy.md) | Testing strategy | Accepted |
| [0012](0012-ci-versioning-and-release.md) | CI versioning & release strategy | Accepted |
| [0013](0013-endpoint-addition-pattern.md) | Endpoint-addition pattern | Accepted |
| [0014](0014-pre-commit-secret-guard.md) | Pre-commit secret guard | Accepted |
| [0015](0015-cli-binary-naming.md) | CLI & server binary names | Accepted |
| [0016](0016-ci-live-tests-machine-identity.md) | CI live tests via an Infisical machine identity | Accepted |
| [0017](0017-release-tables-tree.md) | Modelling `release/tables` (the recursive table tree) | Accepted |

## Backlog (proposed, not yet written)

Decisions we intend to record, roughly in the order we expect to need them.
Order and contents will change as we build.

_Currently empty — the foundational decisions are all recorded. The
`release/tables` recursive shape, once a backlog example, is now
[ADR-0017](0017-release-tables-tree.md) (Accepted). New entries land here as they
surface._
