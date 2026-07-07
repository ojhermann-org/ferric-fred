#!/usr/bin/env bash
#
# Wall-clock CLI timing for the `fred` binary, via hyperfine (perf-tooling
# pilot — issue #42 / docs/adr/0026-*.md).
#
# Two benchmarks, split by what they need:
#
#   1. startup   — `fred --version`: pure process start + clap init. Offline and
#                  deterministic, so it is the reliable regression signal (binary
#                  size, static-init, and link cost show up here). Always runs.
#
#   2. fetch     — `fred observations <series> --limit N`: a representative
#                  fetch-and-render round trip (network + deserialize + tabular
#                  print). Needs FRED_API_KEY and hits the live API, so it is
#                  wall-clock-noisy — informative, not a tight regression gate.
#                  Skipped (not failed) when FRED_API_KEY is unset.
#
# TUI startup (`fred chart`) is intentionally NOT timed here: the ratatui event
# loop blocks on `event::read()` from the tty and has no headless render-and-exit
# path for hyperfine to drive to completion. Adding one is a scoped follow-up
# (see the ADR); a fragile stdin-injection hack would measure the harness, not
# the app.
#
# Usage:
#   scripts/bench-cli.sh                  # startup (+ fetch if FRED_API_KEY set)
#   scripts/bench-cli.sh --series UNRATE  # choose the fetch series (default GNPCA)
#   scripts/bench-cli.sh --json DIR       # also write hyperfine JSON to DIR/
#
# Secrets: FRED_API_KEY comes from the dev shell (Infisical via direnv — see
# README "Secrets"). This script never prints it. Run under `direnv exec . ...`
# if your shell is not already in the project env.

set -euo pipefail

SERIES="GNPCA"
LIMIT=1000
JSON_DIR=""

while [ $# -gt 0 ]; do
  case "$1" in
    --series) SERIES="$2"; shift 2 ;;
    --limit)  LIMIT="$2";  shift 2 ;;
    --json)   JSON_DIR="$2"; shift 2 ;;
    -h|--help) sed -n '2,40p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

if ! command -v hyperfine >/dev/null 2>&1; then
  echo "hyperfine not found — install it (it ships in the dev flake) and retry." >&2
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "Building release binary…" >&2
cargo build --release -p ferric-fred-cli >&2
FRED="$ROOT/target/release/fred"

[ -n "$JSON_DIR" ] && mkdir -p "$JSON_DIR"
json_flag() { [ -n "$JSON_DIR" ] && printf -- '--export-json=%s/%s.json' "$JSON_DIR" "$1"; }

echo "== startup: fred --version ==" >&2
# shellcheck disable=SC2046
hyperfine --warmup 10 --shell=none $(json_flag startup) "$FRED --version"

if [ -n "${FRED_API_KEY:-}" ]; then
  echo "== fetch-and-render: fred observations $SERIES --limit $LIMIT ==" >&2
  # A network round trip: fewer runs, and warm up once to prime DNS/TLS.
  # shellcheck disable=SC2046
  hyperfine --warmup 1 --runs 10 $(json_flag fetch) \
    "$FRED observations $SERIES --limit $LIMIT"
else
  echo "== fetch-and-render: SKIPPED (FRED_API_KEY unset) ==" >&2
fi
