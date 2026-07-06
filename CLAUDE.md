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

## Deletion & creation

**Ask before deleting or substantively rewriting:**

- **ADRs (`docs/adr/NNNN-*.md`).** The ADR log is append-only decision history.
  Don't delete or gut an accepted ADR — supersede it with a *new* ADR that
  references it and flip the old one's status. Typo/link fixes are fine.
- **Lockfiles (`flake.lock`, `Cargo.lock`).** Regenerate through tooling
  (`nix flake update`, `cargo update`) — never hand-delete.
- **Tracked env config (`.envrc.shared`, `.envrc.example`).** Changing how
  secrets load affects everyone — confirm first.
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
