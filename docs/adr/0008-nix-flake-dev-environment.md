# ADR-0008: Nix flake for the development environment

- **Status:** Accepted
- **Date:** 2026-07-04
- **Deciders:** Project owner

## Context

The project is developed in a Nix-centric environment and will eventually want
to run its MCP server declaratively (e.g. via home-manager). We want a
reproducible toolchain shared between local development and CI, without paying a
`nix build` cost on every compile.

## Decision

We will provide a **Nix flake** (`flake.nix` + committed `flake.lock`) scoped, for
now, to a **development shell only**.

- **Toolchain via `oxalica/rust-overlay`**: recent **stable** Rust
  (`rust-bin.stable.latest.default`) with `rust-src`, `rust-analyzer`, `clippy`,
  and `rustfmt` components. This matches ADR-0007 (track stable, no pinned MSRV
  yet); the exact version is reproducible through `flake.lock`.
- **Dev tools in the shell**: `cargo-nextest`, `cargo-deny` (more, e.g.
  `release-plz`, added when their ADRs land).
- **No OpenSSL / `pkg-config`** in the shell — ADR-0003's choice of `rustls-tls`
  means the HTTP stack has no system TLS dependency, keeping the closure small
  and cross-platform.
- **We still build with plain `cargo`** inside the shell; Nix supplies the
  environment, not the build. This preserves fast iteration.
- **`flake-utils`** provides multi-system outputs; `formatter` is
  `nixfmt-rfc-style` so `nix fmt` works.
- **direnv**: a `.envrc` containing `use flake` auto-loads the shell on entry;
  `.direnv/` and `result` are gitignored.
  > **Refined by [ADR-0009](0009-secret-management-infisical-direnv.md):** the
  > tracked entry point moved to a secret-free **`.envrc.shared`** (which holds
  > the `use flake` line plus Infisical injection); `.envrc` became git-ignored
  > and now just `source_env .envrc.shared`.

### Deferred to a future ADR (Phase 2)

Reproducible **package** outputs — building `ferric-fred-cli` and
`ferric-fred-mcp` as flake `packages` via `crane`, and exposing the MCP server
as a consumable module for home-manager/NixOS. Deferred until those binaries
exist and are worth packaging.

## Consequences

- `nix develop` (or direnv) yields an identical toolchain locally and in CI;
  "works on my machine" drift disappears.
- One additional input surface to maintain (`flake.nix`, `flake.lock`); bumping
  Rust is `nix flake update` on the overlay.
- Non-Nix contributors must install Rust themselves — the plain `cargo`
  workflow still works, and this will be noted in the README.

## Alternatives considered

- **`rustup` + a `rust-toolchain.toml`, no Nix** — simpler for non-Nix users,
  but no reproducible pin of the *other* tools and system deps, and off-idiom
  for this environment. We can still add a `rust-toolchain.toml` later for
  non-Nix users without conflict.
- **`fenix`** — comparable toolchain provider; `rust-overlay` chosen for its
  ubiquity. Revisitable.
- **Nix as the build tool now (`crane`/`naersk`)** — reproducible builds, but
  slower iteration and premature before the binaries exist. Deferred to Phase 2.
