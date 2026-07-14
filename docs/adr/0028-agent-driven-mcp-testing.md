# ADR-0028: Agent-driven MCP testing

- **Status:** Accepted
- **Date:** 2026-07-14
- **Deciders:** Otto Hermann

## Context

[ADR-0011](0011-testing-strategy.md) tests the MCP server the same way it tests
the library: unit tests, offline `wiremock` integration tests, `#[ignore]`d live
tests, and a JSON-RPC handshake check. Those catch **mechanical** defects —
wrong query params, a mis-mapped error, a broken handshake — and they run in CI.

They do **not** catch the class of defect that only shows up when a *language
model* consumes the server, which is its actual audience ([ADR-0010](0010-mcp-server-design.md)):

- a tool **description** that is accurate but ambiguous, so a model picks the
  wrong tool or the wrong argument;
- an **input schema** that omits a hint a model needs (which enum codes are
  valid, that two params are "both or neither");
- a **structured result that drifts from its advertised `outputSchema`**
  ([ADR-0023](0023-mcp-output-schemas.md)) in a way no current assertion pins;
- an **error message** that is technically correct but unhelpful to an agent
  trying to recover.

These are exactly the signals Glama's per-tool quality review grades (see
`CLAUDE.md`), and today we only get them second-hand, after a release. We want a
first-party way to surface them on demand — before a release, and as a repeatable
check when the MCP surface changes.

## Decision

**We add an agent-driven MCP audit: a headless Claude agent acts as an MCP
*client*, drives `ferric-fred-mcp` against live FRED, and reports defects a
deterministic test would miss.** It is committed, repeatable tooling, and it
complements — does not replace — the ADR-0011 layers.

- **`.mcp.json`** (repo root, committed) registers the server for any Claude Code
  session in the repo. It launches the server with `cargo run --release --quiet
  -p ferric-fred-mcp` and carries **no secret**: the server reads `FRED_API_KEY`
  from the ambient environment (direnv + Infisical, [ADR-0009](0009-secret-management-infisical-direnv.md)),
  so the config is safe to commit and doubles as a way to use the FRED tools
  interactively while developing.
- **`scripts/mcp-agent-audit.sh`** is the repeatable harness. It pre-builds the
  server (so a cold `cargo run` can't outrun the client's startup timeout), then
  runs `claude -p` with `--mcp-config .mcp.json --strict-mcp-config` and a
  structured audit brief, granting only the server's tools. It writes a markdown
  findings report (severity-ranked) under `target/mcp-audit/`. The key is never
  printed; the script fails fast if it is absent and expects to be run under
  `direnv exec .`.
- **Cadence.** Event-driven first: run it **on any change to the MCP surface**
  (tools, descriptions, annotations, schemas), alongside the Glama re-score the
  same change already requires. Periodic second, matching the Glama sweep in
  `CLAUDE.md`. Findings become GitHub issues; a surface-level fix ships as its own
  prompt release (the Glama rule), not batched.
- **Not in the CI gate.** The audit needs a live key and a model in the loop, so
  it is non-deterministic and human/agent-initiated, like the `#[ignore]`d live
  tests. CI keeps gating on the offline layers.

## Consequences

- We get the model's-eye view of the tool surface on demand, turning
  description/schema/error-quality problems into fixable findings before they
  reach Glama or a user — and giving each MCP-surface PR a natural pre-flight.
- The committed `.mcp.json` also makes the FRED tools available in-session for
  development, at the cost of one more top-level config file (and a first-call
  compile delay if the server binary isn't already built).
- The audit is only as good as the driving model and its brief; it finds
  *candidates*, and a human still triages which are real. It can miss issues and
  occasionally over-report — acceptable for an exploratory layer whose output is
  reviewed, not merged.
- A live-FRED dependency and a Claude CLI dependency for this one script; both are
  already in the dev shell, and neither touches the gate.

## Alternatives considered

- **A deterministic JSON-RPC conformance script** (canned requests over stdio,
  asserting result-vs-`outputSchema`). Worth having and partly covered by the
  ADR-0011 handshake test — but it can't judge *description clarity* or
  *error-message helpfulness*, which is the whole point of putting a model in the
  loop. The two are complementary; this ADR adds the model-driven half.
- **Rely on Glama's review alone.** Rejected as the only signal: it arrives after
  a release, is async and coarse (per-tool scores, not reproductions), and we
  can't run it against an unreleased branch. The local audit is the pre-flight
  Glama can't be.
- **A hand-maintained checklist a human runs.** Rejected: the same brief given to
  an agent is repeatable, exercises the tools for real, and doesn't rot as tools
  are added.
- **Wire the audit into CI.** Rejected: non-deterministic, needs a live key and a
  model, and would make the gate flaky — the same reasons the live tests are
  `#[ignore]`d (ADR-0011).
