# ADR-0006: License

- **Status:** Accepted
- **Date:** 2026-07-04
- **Deciders:** Project owner

## Context

We are publishing open-source Rust crates and want maximal downstream
compatibility with the ecosystem's norms.

## Decision

We will license the project under **`MIT OR Apache-2.0`** (dual license, user's
choice), matching the Rust ecosystem default.

- `Cargo.toml` sets `license = "MIT OR Apache-2.0"` for every crate.
- The repo root contains both `LICENSE-MIT` and `LICENSE-APACHE`.
- The same license applies to all workspace members.

Note: this covers *our code*. FRED data itself is subject to the St. Louis Fed's
terms of use, and users need their own FRED API key — the library ships no data
and no key.

## Consequences

- Downstream users pick whichever license suits them; Apache-2.0 supplies an
  explicit patent grant, MIT supplies maximum simplicity.
- Contributions are understood to be under the same dual license.

## Alternatives considered

- **MIT only** — simplest, but no patent grant.
- **Apache-2.0 only** — patent grant, but less common as a sole license in Rust
  and slightly less permissive for some downstreams.
