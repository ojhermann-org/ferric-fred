# ADR-0016: CI live tests via an Infisical machine identity

- **Status:** Accepted <!-- Proposed | Accepted | Deprecated | Superseded by ADR-XXXX -->
- **Date:** 2026-07-05
- **Deciders:** Project owner

## Context

The `#[ignore]` live tests hit the real FRED API and need `FRED_API_KEY`
([ADR-0011](0011-testing-strategy.md)). The main CI workflow (`ci.yml`) runs the
offline suite only. [ADR-0009](0009-secret-management-infisical-direnv.md) chose
the user-login path for local dev and explicitly deferred the CI secret path — "a
**machine identity + token** (`infisical run --token …`) is the path for CI, to
be specified when we wire CI/release (later ADR)" — and ADR-0011 anticipated the
live tests running in CI "once a machine identity lands."

We want that wiring in place now, so activation is just provisioning the
identity and adding two secrets — without the workflow failing in the interim on
a repo that hasn't set the token yet.

## Decision

We will add a separate, **dormant** workflow, `.github/workflows/live.yml`:

- **Cadence** — a nightly `schedule` plus manual `workflow_dispatch`, *not* on
  every PR. The live tests are an external-API smoke test (payload-drift check);
  keeping them off the PR path avoids FRED rate limits, network flakiness, and
  added latency on the fast gate.
- **Auth** — an Infisical **machine identity** via Universal Auth. Two repository
  secrets, `INFISICAL_CLIENT_ID` / `INFISICAL_CLIENT_SECRET`, feed
  `infisical login --method=universal-auth … --plain --silent` to obtain an
  access token; `infisical run --token … --projectId … --env=dev --path=/shared`
  then injects the secrets and runs `cargo nextest run --run-ignored
  ignored-only`. The env/path (`dev:/shared`) matches local dev, and the project
  id is read from the committed `.infisical.json` so it stays single-sourced.
- **Dormancy** — a `gate` job checks whether the identity secrets are present and
  exposes the result as an output; the `live` job runs only when configured.
  Absent the token the workflow is a green no-op. (Secrets can't be referenced in
  a job-level `if:`, hence the gate job rather than an inline condition.)
- The obtained token is masked with `::add-mask::`; only the ignored tests run
  here — the offline suite stays in `ci.yml`.

## Consequences

- The wiring is ready: activation is provisioning a machine identity with
  read-only access to `dev:/shared` and adding the two secrets — no workflow or
  code change flips it on.
- Once active, the live tests re-validate our types against real FRED payloads
  nightly, not only when someone runs them locally.
- Live tests stay off PR CI, so the fast gate keeps no external dependency; a
  nightly cadence means up to a day's lag before FRED payload drift is caught —
  acceptable, since the offline `wiremock` tests (which encode our understanding
  of the wire format) run on every PR.
- A machine identity is a standing credential to manage and rotate; scoping it
  read-only to `dev:/shared` limits the blast radius. Scheduled workflows run
  only on the default branch, which is fine for a smoke test.
- Infisical stays the single source of truth for the key (ADR-0009) — CI reads
  through it rather than holding a second copy.

## Alternatives considered

- **A job in `ci.yml` on every PR** — would exercise live FRED constantly (rate
  limits, flakiness, latency on the fast path) and can't get secrets on
  fork-based PRs, forcing skips anyway. Rejected in favour of nightly + dispatch.
- **A long-lived Infisical service token** instead of a machine identity —
  simpler to wire, but a static, broadly-scoped secret with manual rotation;
  ADR-0009 already leaned to a machine identity for automation. Rejected.
- **`FRED_API_KEY` as a GitHub secret directly** — simplest, but it forks the key
  out of Infisical into a second store to rotate and undercuts ADR-0009's single
  source of truth. Rejected; keep Infisical authoritative.
- **Hard-gating with `if: secrets.X != ''` at the job level** — unsupported
  (secrets aren't available in a job-level `if:`), which is why the presence
  check is its own `gate` job.
