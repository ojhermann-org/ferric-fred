#!/usr/bin/env bash
#
# Declarative repo-level GitHub settings for ojhermann-org/ferric-fred.
#
# This file is the source of truth for the handful of settings that GitHub has
# no org-wide default for, and which therefore cannot be managed centrally in
# ojhermann-org/github-settings (that repo is org-level only — org settings and
# rulesets, including branch protection on `main`). See docs/adr/0022-*.md.
#
# The most important line here is `can_approve_pull_request_reviews=true`: with
# it off, release-plz cannot open the version-bump PR and releases fall back to
# manual bumps. It was silently off once; tracking it in version control with a
# CI drift-check keeps it from regressing unnoticed.
#
# Usage:
#   scripts/repo-settings.sh check   # report drift; exit 1 if GitHub differs (CI)
#   scripts/repo-settings.sh apply   # make GitHub match the desired state below
#
# Auth: uses `gh`, so it needs a token with admin on the repo (the shared PAT the
# dev shell already exports, or a CI token with `administration: write`).

set -euo pipefail

REPO="ojhermann-org/ferric-fred"

# CI injects the repo-admin PAT as REPO_SETTINGS_TOKEN (from Infisical,
# dev:/ferric-fred); `gh` authenticates from GH_TOKEN. Bridge them so the CI
# workflow needs no extra wiring. Locally neither is set and `gh` uses your
# logged-in session — also fine.
if [[ -z "${GH_TOKEN:-}" && -n "${REPO_SETTINGS_TOKEN:-}" ]]; then
  export GH_TOKEN="$REPO_SETTINGS_TOKEN"
fi

# --- Desired state -----------------------------------------------------------
# Values mirror the intended configuration. Changing a value here and running
# `apply` (or merging so CI applies) is how a settings change is made — always
# via a reviewed diff, never by hand in the GitHub UI.

# Core repository settings (GET/PATCH /repos/{repo}).
# NOTE: all three merge methods stay enabled — the release flow merges PRs as
# merge commits to preserve the conventional-commit history release-plz reads.
read -r -d '' DESIRED_REPO <<'JSON' || true
{
  "description": "A strongly-typed async Rust client for the FRED economic-data API, with a terminal-charting CLI and an MCP server.",
  "homepage": null,
  "allow_squash_merge": true,
  "allow_merge_commit": true,
  "allow_rebase_merge": true,
  "allow_auto_merge": false,
  "delete_branch_on_merge": false,
  "has_issues": true,
  "has_wiki": false,
  "has_projects": true
}
JSON

# Actions default workflow permissions (GET/PUT /repos/{repo}/actions/permissions/workflow).
read -r -d '' DESIRED_WORKFLOW <<'JSON' || true
{
  "default_workflow_permissions": "read",
  "can_approve_pull_request_reviews": true
}
JSON

# Repository topics (GET/PUT /repos/{repo}/topics). Kept sorted.
DESIRED_TOPICS='["api-client","cli","economics","finance","fred","mcp","ratatui","rust"]'

# --- Machinery ---------------------------------------------------------------

drift=0

# compare <label> <desired-json> <current-json>
# Reports per-key differences for every key present in <desired-json>.
#
# GitHub only populates the merge-method fields (allow_*_merge,
# delete_branch_on_merge) in GET /repos for tokens with *write* Administration.
# The CI token is read-only by design, so it sees them as null — a token blind
# spot, not drift. When actual is null but desired is not, we skip that field
# (and say so) rather than false-fail. A full admin token (local `apply`/`check`)
# sees the real values and verifies them normally.
compare() {
  local label="$1" desired="$2" current="$3"
  local keys key d c
  keys="$(jq -r 'keys[]' <<<"$desired")"
  while IFS= read -r key; do
    d="$(jq -c --arg k "$key" '.[$k]' <<<"$desired")"
    c="$(jq -c --arg k "$key" '.[$k]' <<<"$current")"
    if [[ "$c" == "null" && "$d" != "null" ]]; then
      printf '  skip   %-40s not visible to this token (read-only) — verify locally\n' "$label.$key"
      continue
    fi
    if [[ "$d" != "$c" ]]; then
      printf '  drift  %-40s desired=%s  actual=%s\n' "$label.$key" "$d" "$c"
      drift=1
    fi
  done <<<"$keys"
}

check() {
  echo "Checking repo-level settings for $REPO ..."
  compare "repo" "$DESIRED_REPO" "$(gh api "repos/$REPO")"
  compare "workflow" "$DESIRED_WORKFLOW" "$(gh api "repos/$REPO/actions/permissions/workflow")"

  # Topics: order-insensitive comparison.
  local want got
  want="$(jq -cS '.' <<<"$DESIRED_TOPICS")"
  got="$(gh api "repos/$REPO/topics" --jq '.names' | jq -cS '.')"
  if [[ "$want" != "$got" ]]; then
    printf '  drift  %-40s desired=%s  actual=%s\n' "topics" "$want" "$got"
    drift=1
  fi

  if [[ "$drift" -eq 0 ]]; then
    echo "OK — GitHub matches the desired state."
  else
    echo "DRIFT — run 'scripts/repo-settings.sh apply' to reconcile." >&2
    return 1
  fi
}

apply() {
  echo "Applying repo-level settings to $REPO ..."

  # Core settings: PATCH accepts the object as-is.
  jq -c '.' <<<"$DESIRED_REPO" | gh api -X PATCH "repos/$REPO" --input - >/dev/null
  echo "  set repository settings"

  # Workflow permissions.
  jq -c '.' <<<"$DESIRED_WORKFLOW" | gh api -X PUT "repos/$REPO/actions/permissions/workflow" --input - >/dev/null
  echo "  set actions workflow permissions"

  # Topics.
  jq -c '{names: .}' <<<"$DESIRED_TOPICS" | gh api -X PUT "repos/$REPO/topics" --input - >/dev/null
  echo "  set topics"

  echo "Done. Re-run 'check' to confirm."
}

case "${1:-}" in
  check) check ;;
  apply) apply ;;
  *)
    echo "usage: $0 {check|apply}" >&2
    exit 2
    ;;
esac
