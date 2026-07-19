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
| [0018](0018-release-secret-via-infisical.md) | Route the crates.io token through Infisical | Accepted |
| [0019](0019-series-updates-time-window.md) | series/updates time window (non-ISO timestamps) | Accepted |
| [0020](0020-auto-pagination.md) | Auto-pagination (`Paginate` trait + `send_all`) | Accepted |
| [0021](0021-streaming-pagination.md) | Streaming pagination (`Paginate::stream`) | Accepted |
| [0022](0022-repo-settings-as-code.md) | Repo-level GitHub settings as code | Accepted |
| [0023](0023-mcp-output-schemas.md) | MCP tool output schemas via a feature-gated `schemars` derive | Accepted |
| [0024](0024-alfred-point-in-time-observations.md) | ALFRED point-in-time & vintage observations | Accepted |
| [0025](0025-geofred-maps-api.md) | GeoFRED / Maps API support (regional & geographic data) | Accepted |
| [0026](0026-perf-tooling-pilot.md) | Perf-tooling pilot — divan, hyperfine, bencher | Accepted |
| [0027](0027-types-in-types-out.md) | Types in, types out — make illegal states unrepresentable | Accepted |
| [0028](0028-agent-driven-mcp-testing.md) | Agent-driven MCP testing | Accepted |
| [0029](0029-shared-disciplines-across-the-sibling-rust-mcp-repos.md) | Shared disciplines across the sibling Rust MCP repos | Accepted |
| [0030](0030-l2-testing-stance-proptest-adopt-or-decline.md) | The L2 testing stance — targeted class-tests, broad `proptest` declined | Accepted |

## Backlog (proposed, not yet written)

Decisions we intend to record, roughly in the order we expect to need them.
Order and contents will change as we build.

Typed-invariant candidates licensed by [ADR-0027](0027-types-in-types-out.md)
(deferred there so each lands as its own reviewed, appropriately-versioned change):

- **Bounded `Limit` / `Offset` newtypes.** Validated numerics honouring FRED's
  per-endpoint caps (e.g. `limit` 1–1000, `offset >= 0`), so an out-of-range page
  request is unrepresentable at construction rather than a 400 from FRED. Routes
  through the currently-underused `Error::InvalidInput`.
- **A pairing/ordering type for realtime & update windows.** Lift the "both or
  neither" and `start <= end` checks (today runtime `if`s in the MCP layer) into a
  single library type at the chokepoint every surface funnels through.
