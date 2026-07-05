# ADR-0014: Pre-commit secret guard

- **Status:** Accepted <!-- Proposed | Accepted | Deprecated | Superseded by ADR-XXXX -->
- **Date:** 2026-07-05
- **Deciders:** Project owner

## Context

`FRED_API_KEY` lives in git-ignored files — `.envrc`, `.envrc.local`, `.env*` —
per [ADR-0009](0009-secret-management-infisical-direnv.md). `.gitignore` stops a
plain `git add` of those files, but it has two gaps: `git add -f` bypasses it
entirely, and a key pasted into an otherwise-normal tracked file (a test, a doc,
a scratch snippet) is never an ignored *file*, so `.gitignore` can't help. We
want a guard that fails the commit locally, before a secret ever enters history.

The repository already runs a native `.githooks/pre-push` (fmt/clippy/tests)
wired through `core.hooksPath = .githooks`. The sibling `infisical` repo guards
secrets with the `prek` pre-commit framework plus `gitleaks`. The question is
what shape the guard should take here.

## Decision

We will add a native **`.githooks/pre-commit`** with two layers, both scanning
only staged content so a commit stays fast:

1. **A hard block on staged secret files** — pure `bash`, no dependencies. It
   rejects staged paths named `.envrc`, `.envrc.local`, `.env`, or `.env.*`,
   while allowing the tracked, deliberately secret-free `.envrc.shared`,
   `.envrc.example`, and `.env.example`. This is belt-and-suspenders over
   `.gitignore` for the exact files we know hold the key, and it catches the
   `git add -f` case.
2. **A `gitleaks` scan of the staged diff** (`gitleaks git --staged`) for secret
   *patterns* anywhere in the change, catching a key pasted into a normal file.
   It is skipped with a warning if `gitleaks` isn't on `PATH`; the file block in
   layer 1 still holds regardless.

`gitleaks` is added to the flake dev shell so it is present in the normal working
environment (and available to CI later). The hook needs no new activation — the
same `core.hooksPath = .githooks` that enables `pre-push` enables it.

## Consequences

- Secrets are caught locally before they enter a commit, covering both a
  force-added secret file and a key pasted into an ordinary file.
- The hard guarantee (no `.envrc` committed) is pure `bash` and does not depend
  on `gitleaks`; the scanner only widens the net. If `gitleaks` is missing the
  commit still gets the file block.
- One package (`gitleaks`) joins the dev shell. Because both layers scan only
  staged content, per-commit latency stays low (tens of milliseconds).
- Local hooks are advisory: a contributor can `git commit --no-verify`, so this
  stops honest mistakes, not a determined actor. A defense-in-depth `gitleaks`
  run in CI (over the PR/branch) is a natural follow-up, left out of scope here.
- The guard is not a substitute for rotating any key that does leak.

## Alternatives considered

- **`prek` + `.pre-commit-config.yaml`** (as in the sibling repo) — a capable
  framework, but it adds a config format and a framework dependency for what is
  one hook here, and diverges from this repo's existing native-hook approach
  (`pre-push`). Rejected for consistency and simplicity; a short shell script is
  easy to read, audit, and edit.
- **`gitleaks` only, no file block** — a scanner may not flag every shape of a
  `.envrc` full of `export`s, and the file block gives a hard, dependency-free
  guarantee for the exact files that hold the key. Both layers kept.
- **CI-only scanning** — catches secrets only after they're pushed and already in
  history; a worse feedback loop. A local guard is preferred; a CI scan can be
  layered on top later.
- **Server-side (pre-receive) hook** — not available on the GitHub-hosted remote,
  and it would reject after the secret is already committed locally.
