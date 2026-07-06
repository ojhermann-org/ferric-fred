#!/usr/bin/env bash
#
# Measure an MCP (stdio) server's cold start and idle memory.
#
# Cold start = wall time from spawning the server process to receiving its
# response to a JSON-RPC `initialize` request. Idle RSS = resident memory of the
# server process tree just after the handshake. Neither touches the FRED API, so
# a dummy key is fine and the network is out of the picture — this measures the
# server itself, not query latency.
#
# Usage:  bench.sh <label> -- <command to spawn the server...>
# Env:    RUNS (default 20), FRED_API_KEY (default "dummy").
set -euo pipefail

label="$1"; shift
[ "${1:-}" = "--" ] && shift

INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"bench","version":"0"}}}'
INITED='{"jsonrpc":"2.0","method":"notifications/initialized"}'
export FRED_API_KEY="${FRED_API_KEY:-dummy}"
RUNS="${RUNS:-20}"

# warm the page/module cache so we measure steady-state spawn, not first-touch I/O
printf '%s\n' "$INIT" | "$@" 2>/dev/null | head -n1 >/dev/null

# cold start: N spawns, each timed from launch to the initialize response
for _ in $(seq 1 "$RUNS"); do
  s=$(date +%s%N)
  printf '%s\n' "$INIT" | "$@" 2>/dev/null | head -n1 >/dev/null
  e=$(date +%s%N)
  echo $(( (e - s) / 1000000 ))
done | sort -n | awk -v l="$label" '
  {a[NR]=$1; sum+=$1}
  END{printf "%-14s cold-start ms:  min=%-4s median=%-4s mean=%-4s max=%-4s (n=%d)\n",
             l, a[1], a[int((NR+1)/2)], int(sum/NR), a[NR], NR}'

# idle RSS: keep one server alive past the handshake, sum the process tree
FIFO=$(mktemp -u); mkfifo "$FIFO"
"$@" < "$FIFO" >/dev/null 2>&1 &
PID=$!
exec 3>"$FIFO"
printf '%s\n%s\n' "$INIT" "$INITED" >&3
sleep 0.6
rss=$(awk '/VmRSS/{print $2}' "/proc/$PID/status" 2>/dev/null || echo 0)
for c in $(pgrep -P "$PID" 2>/dev/null || true); do
  rss=$(( rss + $(awk '/VmRSS/{print $2}' "/proc/$c/status" 2>/dev/null || echo 0) ))
done
printf "%-14s idle RSS:       %d MB\n" "$label" "$(( rss / 1024 ))"
exec 3>&-; kill "$PID" 2>/dev/null || true; rm -f "$FIFO"
