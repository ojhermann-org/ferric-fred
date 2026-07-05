# ADR-0018: Route the crates.io token through Infisical

- **Status:** Accepted <!-- Proposed | Accepted | Deprecated | Superseded by ADR-XXXX -->
- **Date:** 2026-07-05
- **Deciders:** Project owner

## Context

[ADR-0012](0012-ci-versioning-and-release.md) set up the release mechanism
(`release-plz`, independent per-crate semver, publish all three crates) and, for
the publish credential, decided: **`CARGO_REGISTRY_TOKEN` as a GitHub Actions
secret.** At the time that was the only secret the release job needed and there
was no other secret machinery in CI.

Since then the repo grew an Infisical integration for CI:
[ADR-0016](0016-ci-live-tests-machine-identity.md) added the `ferric-fred-ci`
machine identity (Universal Auth) so `live.yml` injects `FRED_API_KEY` from
Infisical (`dev:/shared`) rather than storing it as a GitHub secret. GitHub
Actions now holds only the two Infisical bootstrap secrets
(`INFISICAL_CLIENT_ID` / `INFISICAL_CLIENT_SECRET`); every real value lives in
Infisical. The machine identity holds the built-in `viewer` role — project-wide
read — so it can already read any path without a new grant.

Against that backdrop, keeping `CARGO_REGISTRY_TOKEN` as a *second* GitHub
Actions secret is the odd one out: two places to manage secrets, two mental
models. This ADR revisits only ADR-0012's secret-storage sub-decision.

## Decision

We will **store the crates.io token in Infisical and inject it into the release
job**, the same way `FRED_API_KEY` is injected — superseding ADR-0012's
"GitHub Actions secret" choice for this token (the rest of ADR-0012 stands).

- The token lives at **`dev:/shared/CARGO_REGISTRY_TOKEN`** in the `otto-infra`
  project, alongside `FRED_API_KEY`. No IaC change is needed: the folder exists
  and the `ferric-fred-ci` identity's `viewer` role already grants read. (A
  dedicated per-app folder would be tidier, but it isn't worth a new
  `infisical_secret_folder` + `tofu apply` for one CI-only credential; revisit
  if more release secrets accrue.)
- The value is managed out-of-band via the `infisical` CLI / Proton Pass, never
  in Terraform state (the standard structure-vs-values split).
- `release.yml` authenticates the machine identity via Universal Auth and runs
  `release-plz` under `infisical run --env=dev --path=/shared`, so
  `CARGO_REGISTRY_TOKEN` reaches `cargo publish` as an environment variable and
  is never written to a GitHub secret or to disk — mirroring `live.yml`.
- GitHub Actions therefore holds **only** the two Infisical bootstrap secrets;
  the publish credential is one more value in the single Infisical source of
  truth.

## Consequences

- One secrets model for all of CI: everything but the Infisical bootstrap creds
  lives in Infisical. Adding or rotating the crates.io token is an `infisical`
  operation, not a GitHub-secret one.
- Publishing now depends on Infisical availability and the machine identity, on
  top of crates.io — a slightly longer trust chain than a bare GitHub secret.
  Accepted: `live.yml` already has this dependency, and the identity is
  IP-restricted to GitHub runners.
- `CARGO_REGISTRY_TOKEN` sits in `dev:/shared`, which is nominally for
  cross-cutting values; it is ferric-fred-specific. A minor purity compromise
  taken to avoid standing up a new Infisical folder (an `~/infisical` `tofu`
  change) for a single value.
- The release job carries the same Infisical bootstrap/auth boilerplate as
  `live.yml`; the two workflows now share that shape.

## Alternatives considered

- **Keep ADR-0012's plain GitHub Actions secret.** Simplest single-purpose
  option and no Infisical dependency on the publish path, but it reintroduces a
  second secrets model for one value. Rejected for consistency now that all
  other CI secrets flow through Infisical.
- **A dedicated `prod:/ferric-fred` Infisical folder** (mirroring
  `prod:/hetzner`). Tidier separation and matches the per-app-folder convention,
  but needs an `~/infisical` `tofu apply` (interactive `pass-cli` auth) to
  create the folder. Rejected as not worth the IaC step for one CI credential;
  `dev:/shared` reuses existing structure. Revisit if release secrets multiply.
- **A `prod` environment for release.** More faithful (publishing is a
  production action), but there is no `prod`-env secret consumer yet and it would
  mean managing another environment; deferred with the folder question above.
