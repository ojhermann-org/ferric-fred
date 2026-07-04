# ferric-fred — Claude Code guidance

Rust workspace: a strongly-typed FRED API client library, a CLI (`ratatui` TUI),
and an MCP server. See `README.md` and `docs/adr/` for orientation.

These rules layer on top of the global permission model (auto mode + classifier).
Their job is to tell the classifier what counts as **"important"** to delete or
create *in this repo* — the global config only guards universal catastrophes.

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
