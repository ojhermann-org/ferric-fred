# ADR-0022: Repo-level GitHub settings as code

- **Status:** Accepted
- **Date:** 2026-07-06
- **Deciders:** Otto Hermann

## Context

Org-level GitHub settings for `ojhermann-org` — organization settings and
rulesets, including branch protection on `main` across every repo — are managed
as infrastructure-as-code with OpenTofu in the `ojhermann-org/github-settings`
repo. That repo is **deliberately org-level only**: it does not manage
individual repositories or their per-repo settings.

But a handful of settings have no org-wide GitHub equivalent and can only be set
per repository:

- **Actions "allow GitHub Actions to create and approve pull requests"**
  (`can_approve_pull_request_reviews`) and the default workflow-token
  permissions (`default_workflow_permissions`).
- `delete_branch_on_merge`, the allowed merge methods, description, homepage,
  topics, and the issues/wiki/projects toggles.

These lived only in GitHub's UI — untracked and invisible to review. That bit us
concretely: `can_approve_pull_request_reviews` was silently `false`, so
release-plz could not open its version-bump PR and every release fell back to a
manual bump. The setting that gates our release automation had no source of
truth and no way to notice when it drifted.

We wanted these settings tracked in version control. A full OpenTofu project per
repo was considered and rejected (see Alternatives) — for ~8 settings on one
repo it means a state backend, provider auth, a lockfile, and plan/apply
workflows bolted onto a Rust crate workspace, mixing two toolchains.

## Decision

We will track ferric-fred's repo-level settings as a **declarative script in the
repo**, [`scripts/repo-settings.sh`](../../scripts/repo-settings.sh), which is
the source of truth for those settings:

- `repo-settings.sh check` reports any drift between the script's desired state
  and GitHub, exiting non-zero if they differ.
- `repo-settings.sh apply` reconciles GitHub to the desired state.

A settings change is made by editing the desired values in that script and
running `apply` (via a reviewed PR), never by hand in the GitHub UI. This
mirrors how `github-settings` manages its *own* repo-level merge-queue ruleset —
with `gh api`, keeping OpenTofu reserved for the org level.

The script needs a token with `administration` scope (the shared admin PAT the
dev shell / `gh` already provides locally); the default Actions `GITHUB_TOKEN`
cannot carry that scope, which is why this is not wired into a workflow that
relies on `GITHUB_TOKEN`.

**CI drift-check (follow-up):** a scheduled/PR workflow running
`repo-settings.sh check` would catch regressions like the one above
automatically. It requires an admin-scoped PAT provisioned into ferric-fred's
Infisical scope (the repo routes secrets through Infisical, not raw GitHub
secrets — ADR-0018), then injected the same way `release.yml` injects the
crates.io token. That credential provisioning is the one remaining step;
tracked in issue #15.

## Consequences

- The release-gating Actions toggle (and the rest) now have a reviewable source
  of truth; a change is a diff, not an invisible UI click.
- `check` gives an on-demand audit today; once the CI credential is provisioned,
  drift is caught automatically.
- `apply` requires an admin token — an intentional constraint. Contributors
  without one can still read the desired state and propose changes; only the
  reconcile step needs the elevated credential.
- This is a second, non-Rust "config surface" in the repo. It is one small,
  self-contained shell script with no new toolchain or state backend — the cost
  we accept for keeping settings self-managed and in version control.
- Branch protection and other org-wide rules remain in `github-settings`; this
  script never touches them. The boundary is: org-wide → OpenTofu in
  `github-settings`; per-repo-only → this script.

## Alternatives considered

- **A full OpenTofu project in this repo** (the `github-settings` stack shape:
  `github_repository` + `github_workflow_repository_permissions` +
  `github_actions_repository_permissions`, R2 state backend, provider lockfile,
  plan/apply workflows). The provider *can* manage every setting we need. We
  rejected it as disproportionate: a state backend and a second toolchain inside
  a Rust workspace to manage ~8 settings on one repo, plus the risk of Terraform
  owning the `github_repository` lifecycle (destroy semantics) for a repo it did
  not create.
- **Extending `github-settings` to manage repos.** Rejected by design — that
  repo is org-level only, and each repo self-manages its own settings.
- **A raw GitHub Actions secret for the admin PAT.** Rejected for consistency:
  this repo routes secrets through Infisical (ADR-0018), not hand-managed GitHub
  secrets.
- **Leaving settings in the UI.** The status quo that hid the release-gating
  toggle regression. Rejected.
