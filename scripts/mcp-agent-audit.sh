#!/usr/bin/env bash
#
# Agent-driven MCP audit (ADR-0028). Spawns a headless Claude agent as an MCP
# *client* and points it at the `ferric-fred-mcp` server running against live
# FRED, asking it to exercise every tool and report defects a deterministic test
# wouldn't catch: confusing/wrong tool descriptions, input-schema gaps,
# output-vs-`outputSchema` mismatches, error-handling rough edges, and
# missing-pairing surprises.
#
# This complements — does not replace — the offline wiremock/handshake tests
# (ADR-0011). It is an exploratory, LLM-in-the-loop pass, run by a human (or an
# agent session), not part of the CI gate.
#
# Requirements (all supplied by the dev shell + direnv, ADR-0008/0009):
#   - cargo            (builds the server)
#   - claude           (the Claude Code CLI, drives the audit)
#   - FRED_API_KEY     (live key; the server reads it from the environment)
#
# The key is never printed. Run it through direnv so the key is present:
#   direnv exec . scripts/mcp-agent-audit.sh
#
# Output: a markdown findings report under target/mcp-audit/ (path printed at the
# end). Distil it into GitHub issues; any fix that changes the MCP *surface*
# (tool text, annotations, schemas) ships as its own prompt release (Glama rule,
# see CLAUDE.md).

set -euo pipefail

cd "$(dirname "$0")/.."

if [ -z "${FRED_API_KEY:-}" ]; then
  echo "FRED_API_KEY not in env — run under direnv: 'direnv exec . $0'." >&2
  echo "(The server reads the key from the environment; this script never prints it.)" >&2
  exit 1
fi

for bin in cargo claude; do
  command -v "$bin" >/dev/null 2>&1 || { echo "$bin not on PATH (enter the dev shell)." >&2; exit 1; }
done

# Pre-build so the server starts instantly when the agent connects (a cold
# `cargo run` can outrun the MCP client's startup timeout).
echo "== building ferric-fred-mcp (release) ==" >&2
cargo build --release --quiet -p ferric-fred-mcp

OUT_DIR="target/mcp-audit"
mkdir -p "$OUT_DIR"
# No wall-clock in the name (deterministic path); the agent stamps the report.
REPORT="$OUT_DIR/findings.md"
AUDIT_MODEL="${AUDIT_MODEL:-}"        # empty => the CLI's configured model
model_flag=()
[ -n "$AUDIT_MODEL" ] && model_flag=(--model "$AUDIT_MODEL")

# The audit brief. Real FRED anchors are provided so the agent has known-good
# inputs to probe around; the closing format keeps the report distillable into
# issues. The agent may ONLY use the ferric-fred MCP tools.
read -r -d '' PROMPT <<'BRIEF' || true
You are auditing the `ferric-fred` MCP server (tools prefixed `mcp__ferric-fred__`)
as a demanding downstream consumer. It wraps the FRED economic-data API. Your job
is to find defects a deterministic unit test would MISS — issues of clarity,
contract, and robustness — by actually calling the tools against live FRED.

Work systematically:
1. List every tool. For each, read its description, input schema, and output
   schema as an unfamiliar agent would.
2. Call each tool at least once with a realistic input, then probe edges. Useful
   real anchors: series GNPCA and UNRATE; category 0 (root) and 125; release 53;
   source 1; tag "usa"; a GeoFRED series-group id like 1223.
3. Deliberately stress the contract:
   - Bad/nonexistent ids (e.g. series "NOTAREALSERIES") — is the error clear and
     actionable, or confusing?
   - Missing required params, and the "both or neither" pairing rules
     (realtime_start/realtime_end; start_time/end_time) — does omitting one give
     a helpful message?
   - Bad date/datetime formats.
   - Does the STRUCTURED result actually conform to the advertised outputSchema
     (field names, types, required-ness)? Flag any drift.
   - Are enum-valued inputs (units, frequency, sort_order, aggregation, …)
     discoverable from the schema, and do their advertised codes work?
4. Judge the tool DESCRIPTIONS and annotations: accurate? unambiguous? Would a
   model pick the right tool from the description alone? Note any that mislead.

Rules:
- Use ONLY the `mcp__ferric-fred__*` tools. Do not edit files or run other tools.
- Never print the API key or any secret.
- Be concrete: every finding must cite the tool, the exact input, and what you
  observed vs. expected.

Your FINAL message must be the complete findings report in this markdown shape
(and nothing else):

# ferric-fred MCP audit — findings

_Model: <you>. Tools exercised: <n>/<total>._

## Summary
<2-4 sentences: overall health, and the highest-impact issues.>

## Findings
Each as:
### [SEVERITY] <tool> — <one-line title>
- **Category:** description | input-schema | output-schema | error-handling | pairing | other
- **Input:** <exact args used>
- **Observed:** <what happened>
- **Expected:** <what a good consumer would want>
- **Suggested fix:** <concrete, minimal>

SEVERITY is one of: BLOCKER, MAJOR, MINOR, NIT. If a tool is clean, say so under
a "## Clean" list rather than inventing findings.
BRIEF

echo "== running agent audit (this exercises live FRED; may take a few minutes) ==" >&2
# --strict-mcp-config: ignore any other MCP config, use only this repo's server.
# --allowedTools: pre-approve the server's tools so the headless run never blocks
# on a permission prompt; nothing else is granted.
claude -p "$PROMPT" \
  --mcp-config .mcp.json \
  --strict-mcp-config \
  --allowedTools "mcp__ferric-fred" \
  --output-format text \
  "${model_flag[@]}" \
  >"$REPORT"

echo "== audit complete ==" >&2
echo "$REPORT"
