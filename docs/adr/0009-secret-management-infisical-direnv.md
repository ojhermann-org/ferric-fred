# ADR-0009: Secret management via Infisical + direnv

- **Status:** Accepted
- **Date:** 2026-07-04
- **Deciders:** Project owner

## Context

The library needs a **FRED API key** at runtime (and tests/examples that hit the
live API need one too). We want the key injected into the dev environment
without ever committing it, a reproducible way to get the secret tooling, and a
graceful fallback when the secret store is unavailable. The owner already uses
[Infisical](https://infisical.com) as a secret store and an established
`.envrc.shared` / `.envrc` split in a sibling repo; we adopt that pattern here.

This is a **consumer** of secrets (we read a key), not a manager of secret
infrastructure — so we use the simple user-login path, not a machine identity.

## Decision

We mirror the sibling repo's direnv split and inject secrets with the Infisical
CLI:

- **`.envrc.shared`** — committed, **secret-free**. Runs `use flake` (ADR-0008's
  toolchain) and then injects Infisical secrets into the shell via
  `eval "$(infisical export --format=dotenv-export --env=… --path=…)"`. The
  injection is **non-fatal**: if Infisical is absent or not logged in, nothing
  is injected and we fall through to the local `.envrc`. `env`/`path` default to
  `dev` / `/` and are overridable via `INFISICAL_ENV` / `INFISICAL_PATH`.
- **`.envrc.example`** — committed template; `cp .envrc.example .envrc`.
- **`.envrc`** — **git-ignored**, local entry point. Does `source_env
  .envrc.shared` and is the only place a raw secret (`export FRED_API_KEY=…`)
  may be hand-set, as the fallback "on hand in case we need it".
- **`.infisical.json`** — committed, non-secret project link, created by
  `infisical init`.
- **`infisical` CLI is pinned in the flake** devShell (ADR-0008), so everyone
  runs one known version.
- **Auth:** interactive `infisical login` (user identity) per machine for local
  dev. A **machine identity + token** (`infisical run --token …`) is the path
  for CI, to be specified when we wire CI/release (later ADR).
- **Secret convention:** `FRED_API_KEY` at `--env=dev --path=/` for local dev.

The library code itself only reads `std::env` (e.g. `FRED_API_KEY`); it has **no
dependency on Infisical**. Infisical is purely how the environment gets
populated, so non-Infisical users can export the variable any way they like.

## Consequences

- Secrets never touch the repo; the one local file that may hold a raw secret
  (`.envrc`) is git-ignored.
- Reproducible secret tooling via the flake; one-command onboarding
  (`infisical login` + `infisical init`, then `direnv allow`).
- Graceful degradation: no Infisical → set `FRED_API_KEY` in `.envrc` and carry
  on.
- A residual risk remains that someone force-adds `.envrc`; a pre-commit
  gitleaks/secret-file guard (as in the sibling repo) is a proposed follow-up.

## Alternatives considered

- **`infisical run -- <cmd>` per command** — stronger process isolation, but
  you must prefix every `cargo` invocation; worse ergonomics in an interactive
  dev shell. The direnv `export` approach loads once per `cd`-in.
- **Machine identity + Proton Pass references** (the sibling repo's automation
  path) — necessary for unattended automation, overkill for a developer reading
  one key locally. Reserved for CI.
- **`.env` file on disk** — simplest, but puts the plaintext secret on disk and
  invites accidental commits. Rejected.
