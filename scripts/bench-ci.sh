#!/usr/bin/env bash
#
# CI-only: run the benchmarks and upload results to Bencher (bencher.dev), for
# regression tracking on `main` and per-PR comparison (perf pilot — issue #42 /
# ADR-0026). Not for local use: developers run `cargo bench` and
# `scripts/bench-cli.sh` directly.
#
# Assumes on PATH / in env (the `bench.yml` workflow arranges all of this):
#   - cargo + hyperfine   (the dev flake)
#   - bencher             (installed by the bencherdev/bencher action)
#   - BENCHER_API_TOKEN   (injected by `infisical run` from dev:/shared)
#   - GITHUB_TOKEN        (optional; enables the PR comment)
#
# Adapter note (the pilot's finding): Bencher has **no divan adapter**. It parses
# `rust_criterion` and `shell_hyperfine` natively, so CI tracks the *criterion*
# mirror (not divan) and *hyperfine* here; divan stays the fast local harness.
# See ADR-0026.

set -euo pipefail

PROJECT="ferric-fred"
TESTBED="${BENCHER_TESTBED:-ci-ubuntu}"

if ! command -v bencher >/dev/null 2>&1; then
  echo "bencher CLI not on PATH — the bencherdev/bencher action must run first." >&2
  exit 1
fi

# Credential normalization. Bencher deprecated JWT "API tokens"
# (--token / BENCHER_API_TOKEN) in favour of "API keys"
# (--key / BENCHER_API_KEY); our Infisical secret holds a project API key
# (`bencher_run_…`). If it arrived under the older name, remap it to
# BENCHER_API_KEY and clear the token var — otherwise bencher routes the value to
# --token and fails JWT validation. Works whether the secret is named
# BENCHER_API_KEY (preferred) or BENCHER_API_TOKEN.
if [ -z "${BENCHER_API_KEY:-}" ] && [ -n "${BENCHER_API_TOKEN:-}" ]; then
  export BENCHER_API_KEY="$BENCHER_API_TOKEN"
fi
unset BENCHER_API_TOKEN 2>/dev/null || true

if [ -z "${BENCHER_API_KEY:-}" ]; then
  echo "BENCHER_API_KEY/BENCHER_API_TOKEN not in env — Infisical injection failed?" >&2
  exit 1
fi

# Post a PR comment only when a token is present (absent on forks / some events).
gh_flag=()
[ -n "${GITHUB_TOKEN:-}" ] && gh_flag=(--github-actions "$GITHUB_TOKEN")

# 1) Deserialization — criterion adapter. The workload is deterministic and
#    synthetic (no network), so its variance is low enough to gate: `--err`
#    fails the job if Bencher's threshold flags a regression.
echo "== bencher: deserialization (rust_criterion) ==" >&2
bencher run \
  --project "$PROJECT" \
  --testbed "$TESTBED" \
  --adapter rust_criterion \
  "${gh_flag[@]}" \
  --err \
  "cargo bench -p ferric-fred --bench deserialization_criterion"

# 2) CLI startup — hyperfine adapter. Offline and deterministic, but shared-runner
#    wall-clock is noisier than a parse microbench, so it is tracked (history +
#    PR comment) but NOT gated: no `--err` here.
echo "== bencher: CLI startup (shell_hyperfine) ==" >&2
cargo build --release -p ferric-fred-cli >&2
startup_json="$(mktemp)"
bencher run \
  --project "$PROJECT" \
  --testbed "$TESTBED" \
  --adapter shell_hyperfine \
  "${gh_flag[@]}" \
  --file "$startup_json" \
  "hyperfine --warmup 10 --shell=none --export-json $startup_json 'target/release/fred --version'"
rm -f "$startup_json"
