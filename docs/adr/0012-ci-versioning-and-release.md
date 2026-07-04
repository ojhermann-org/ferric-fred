# ADR-0012: CI versioning & release strategy

- **Status:** Accepted <!-- Proposed | Accepted | Deprecated | Superseded by ADR-XXXX -->
- **Date:** 2026-07-04
- **Deciders:** Project owner

## Context

The workspace now has three publishable crates — `ferric-fred` (library),
`ferric-fred-cli` (the `fred` binary), and `ferric-fred-mcp` (the `fred-mcp`
binary) — all pinned at a shared `0.0.0` via `version.workspace = true`
([ADR-0002](0002-workspace-layout.md)). To ship, we need a versioning scheme, a
release mechanism, and the secrets CI requires. Commits already follow
[Conventional Commits](https://www.conventionalcommits.org/). Live tests still
can't run in CI (no Infisical machine identity —
[ADR-0009](0009-secret-management-infisical-direnv.md)); the HTTP-mocked tests
([ADR-0011](0011-testing-strategy.md)) gate CI in the meantime.

## Decision

- **Independent semver per crate, driven by [`release-plz`](https://release-plz.dev/)
  from Conventional Commits.**
  - Drop `version.workspace = true`; each crate carries its own `version`.
    `release-plz` computes each crate's bump from its unreleased conventional
    commits and updates internal dependency requirements (the CLI/MCP
    `ferric-fred` path-deps) automatically.
  - The library follows strict semver for its downstream consumers; the binaries
    version by their own features. Pre-1.0 (`0.x`) rules apply until the library
    API stabilises.
- **Conventional Commits are the standard** (already de facto: `feat` / `fix` /
  `docs` / `test` / `ci` / `build` / `refactor` / `style` / `chore`; `!` or a
  `BREAKING CHANGE:` footer marks a break). They drive both the version bump and
  a per-crate `CHANGELOG.md`.
- **`release-plz` runs in CI as two jobs:**
  - a *release-PR* job on push to `main` that opens/updates a PR bumping versions
    and changelogs (uses the default `GITHUB_TOKEN`);
  - a *release* job that, once that PR merges, tags and publishes to crates.io
    (needs `CARGO_REGISTRY_TOKEN`).
- **Publish all three crates** — the binaries so they are `cargo install`-able,
  the library for downstream use. The internal path-deps already carry an
  explicit `version` for crates.io.
- **First release is `0.1.0`** for all three crates (from `0.0.0`).
- **Secrets:** `CARGO_REGISTRY_TOKEN` as a GitHub Actions secret (crates.io
  publish). Running the `#[ignore]` live tests in CI additionally needs an
  Infisical **machine-identity** token to inject `FRED_API_KEY` (ADR-0009) —
  still deferred; release does not depend on it.

## Consequences

- Each crate gets its own version, changelog, and git tag; downstream users of
  the library see semver that reflects its API, not CLI/MCP churn.
- Releases are PR-reviewed (the release PR) before anything is tagged or
  published — no surprise publishes from a routine push to `main`.
- Conventional Commits become load-bearing: a mislabelled commit misses or
  mis-sizes a bump. Accepted — the discipline is already in place.
- Two repo secrets/permissions to manage; publishing is gated on
  `CARGO_REGISTRY_TOKEN`, which only the owner can set, so the mechanism can land
  before the first publish is authorised.
- Independent versions mean the internal path-dep requirements drift over time;
  `release-plz` maintains them, but any manual `cargo` edit must keep them
  consistent.

## Alternatives considered

- **Unified / lockstep versioning** (keep `version.workspace`) — one version for
  all three, bumped together. Simpler to reason about, but couples the library's
  public semver to binary feature releases. Rejected for a workspace whose
  library is a consumable API; revisit only if the crates prove inseparable.
- **Manual releases** (`cargo publish` by hand, or `cargo release`) — no CI
  machinery, but easy to desync version/changelog/tag across three crates.
  Rejected in favour of `release-plz`'s automation plus a review PR.
- **`cargo-smart-release` / `cargo-workspaces`** — capable alternatives;
  `release-plz` chosen for its PR-based flow, per-crate changelogs, and
  Conventional-Commit bump inference.
- **Publishing only the library** — but the CLI and MCP server are meant to be
  installable; publish all three.
