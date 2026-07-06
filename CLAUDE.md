# ferric-fred — Claude Code guidance

Rust workspace: a strongly-typed FRED API client library, a CLI (`ratatui` TUI),
and an MCP server. See `README.md` and `docs/adr/` for orientation.

These rules layer on top of the global permission model (auto mode + classifier).
Their job is to tell the classifier what counts as **"important"** to delete or
create *in this repo* — the global config only guards universal catastrophes.

## Keep documentation current

Documentation is part of the change, not a follow-up. **Before opening any PR,
check whether it touches user-facing behavior, the public API, or a design
decision, and update the affected docs in the same PR.** The doc surfaces here:

- **Workspace `README.md`** — CLI usage examples, the MCP tool table, and the
  endpoint/status summary. Update when commands, flags, tools, or coverage change.
- **Per-crate `README.md`** (`crates/*/README.md`) — the crates.io / docs.rs
  landing pages; keep the install/usage/feature blurbs in step with reality.
- **Crate-level rustdoc** (`//!` in each crate's `lib.rs`/`main.rs`) plus item
  docs — the docs.rs front page; refresh when the surface or the story changes.
- **ADRs** (`docs/adr/`) — record a *new* ADR for any non-trivial design or
  process decision (see Deletion & creation for the append-only rule).
- **CLI `--help`** (clap doc comments) and **MCP tool descriptions** — these *are*
  the docs for those surfaces; a new flag or tool means new/updated help text.
- **`CONTRIBUTING.md`** when the contribution workflow changes.

Rule of thumb: a new or changed **endpoint** touches the library docs + workspace
README + MCP tool table; a new **feature** touches the relevant crate README(s)
and crate-level rustdoc; a **decision** gets an ADR. If a change makes an existing
doc wrong, fixing it is in scope for that PR — not a later one.

## Repo settings are code

Repo-level GitHub settings (the Actions "create/approve PRs" toggle,
`delete_branch_on_merge`, merge methods, description, topics, …) are the source
of truth in [`scripts/repo-settings.sh`](scripts/repo-settings.sh), per
[ADR-0022](docs/adr/0022-repo-settings-as-code.md). **Change them by editing that
script and running `apply` via a reviewed PR — not by hand in the GitHub UI**, or
the drift-check (`repo-settings.sh check`) will flag it. Org-wide settings (branch
protection, rulesets) are *not* here — they live in `ojhermann-org/github-settings`.

## MCP quality review (Glama)

The MCP server (`ferric-fred-mcp`) is listed and scored on
[Glama](https://glama.ai/mcp/servers/ojhermann-org/ferric-fred) — a quality score
(tool-definition quality + server coherence) plus per-tool feedback. Treat that
feedback as a standing signal, **event-driven first, periodic second**:

- **On every change to the MCP server** (tools, descriptions, annotations, output
  schemas) re-scoring is part of the release, not a follow-up. The score only
  moves when we change the server or Glama changes its methodology — that's where
  the real signal is.
- **Periodic sweep:** pull the score, the Server Quality Checklist, and the Tool
  Scores; compare against last time; open a GitHub issue per gap or regression and
  work them down (e.g. tool annotations, output schemas). Cadence: **monthly** by
  default, **weekly while a quality backlog is open**.

**Refreshing the score after a change is a two-step gotcha** (Glama builds from
git `main`, not crates.io):

1. **Build & release** on the Glama server admin page — recompiles the binary from
   `main` and refreshes the *tool capture*.
2. **Sync Server** in the admin UI — forces the otherwise-async (~daily) LLM
   *re-score*. The public `/score` page is CDN-cached, so cache-bust (`?cb=…`
   plus no-cache headers) to confirm; a rebuild alone won't visibly move the number.

**Keep Glama and crates.io in sync by releasing MCP changes promptly.** Glama
reads the version straight from `main`'s `crates/ferric-fred-mcp/Cargo.toml` (not
crates.io — `server.json`/`glama.json` don't carry an authoritative version), and
release-plz bumps that Cargo.toml in the *same commit* that publishes to crates.io.
So the two are in sync at every release and only drift *while an MCP-surface PR
sits merged-but-unreleased on `main`* — Glama builds the new tools under the old,
still-published version, which both mislabels the score and causes the version
collision that blocks a fresh **Build & release** capture. The rule: **release any
MCP-surface change on its own, promptly — don't batch it** behind unrelated work.
Non-MCP PRs (library-only, CLI-only, docs) can batch freely; they don't move
Glama's build, so their version drift is cosmetic and self-heals at the next
release. This is *not* a "release on every push" rule — it targets exactly the
changes that move the Glama surface.

This review is kicked off from a session (Claude can't self-schedule); the
checklist keeps it repeatable.

## Deletion & creation

**Ask before deleting or substantively rewriting:**

- **ADRs (`docs/adr/NNNN-*.md`).** The ADR log is append-only decision history.
  Don't delete or gut an accepted ADR — supersede it with a *new* ADR that
  references it and flip the old one's status. Typo/link fixes are fine.
- **Lockfiles (`flake.lock`, `Cargo.lock`).** Regenerate through tooling
  (`nix flake update`, `cargo update`) — never hand-delete.
- **Tracked env config (`.envrc.shared`, `.envrc.example`).** Changing how
  secrets load affects everyone — confirm first.
- **`scripts/repo-settings.sh`.** The source of truth for repo-level GitHub
  settings (ADR-0022) — edit desired values deliberately; don't gut it.
- **`flake.nix`, `README.md`.**

**Never:**

- Never delete, move, or print the contents of a git-ignored secret file
  (`.envrc`, `.envrc.local`, `.env*`) — they hold the `FRED_API_KEY`. Don't echo
  secret values into logs, commits, or the terminal.
- Never commit a real secret or a populated `.envrc`. `.gitignore` already covers
  them; keep it that way.

**When creating:**

- **New ADR:** copy `docs/adr/0000-adr-template.md`, use the next sequential
  number (no gaps), and add it to the index at `docs/adr/README.md`.
- **New crate:** follow the workspace layout in
  [ADR-0002](docs/adr/0002-workspace-layout.md); consumers depend on the library
  by workspace path, so keep that compile-time coupling intact.
