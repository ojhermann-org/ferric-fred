# ADR-0007: Rust edition & MSRV policy

- **Status:** Accepted
- **Date:** 2026-07-04
- **Deciders:** Project owner

## Context

We must pick a Rust edition and decide how conservative to be about the minimum
supported Rust version (MSRV). This is a young project optimising for velocity
over enterprise-toolchain reach.

## Decision

- **Edition 2021** across all workspace crates.
- **No hard MSRV guarantee** for now. We build against recent stable Rust and
  aim to keep the crates compiling on the last few stable releases, but we do
  not pin or CI-enforce an explicit `rust-version` yet.
- We may adopt a documented, CI-tested `rust-version` **later**, once the API
  stabilises and we understand who depends on us. Doing so would be its own ADR.

## Consequences

- Lowest maintenance overhead; we can freely use recent stable features and
  dependency versions.
- Users on older toolchains have no guarantee, which is acceptable for an
  early-stage library.
- Adopting edition 2024 or a pinned MSRV remains an easy, additive future change.

## Alternatives considered

- **Edition 2024, latest-stable-only** — newest features, but demands an
  up-to-date toolchain from users for little near-term benefit. Deferred.
- **Pin a conservative MSRV now** — friendlier to enterprise users, but
  constrains language/dependency choices before we have any such users.
  Premature.
