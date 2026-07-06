# MCP server startup benchmark

How quickly does `fred-mcp` come up, and how much memory does it hold, compared
with other FRED MCP servers? This measures **cold start** (spawn → completed MCP
`initialize` handshake) and **idle RSS** (resident memory just after the
handshake).

It deliberately does **not** measure query latency. Every FRED MCP server makes
the same HTTP call to the St. Louis Fed, and that round-trip — plus the model's
own latency — dominates the time to an actual answer; the server's own overhead
is a rounding error against it. Startup and memory are the things the
implementation language actually changes, so those are what this measures. The
handshake (`initialize` + `tools/list`) never calls FRED, so a dummy API key is
enough and the network stays out of the numbers.

## Results

Median of 20 spawns each, same machine, identical harness
([`bench.sh`](bench.sh)):

| server                                                                   | language | cold start | idle RSS |
| ------------------------------------------------------------------------ | -------- | ---------- | -------- |
| [ferric-fred](https://crates.io/crates/ferric-fred-mcp) `0.3.1`          | Rust     | **~8 ms**  | **~7 MB** |
| [fred-mcp-server](https://github.com/stefanoamorelli/fred-mcp-server) `1.1.0` | Node/TS  | ~250 ms    | ~76 MB   |
| [mcp-fredapi](https://github.com/Jaldekoa/mcp-fredapi) (main)            | Python   | ~835 ms    | ~53 MB   |

So ferric-fred starts roughly **30×** faster than the Node server and **~100×**
faster than the Python one, and idles about **10×** and **~7×** lighter
respectively. It also ships as a single ~8 MB binary (5 system libraries, no
runtime), versus a 26 MB `node_modules` (~3,400 files) or a Python venv.

_Machine: AMD EPYC-Rome, 8 cores, Linux 6.18.35. Measured 2026-07-06. Numbers
are wall-clock on one box and will vary; the point is the order of magnitude, not
the third significant figure._

## The harness

[`bench.sh`](bench.sh) needs only `bash` + coreutils. It spawns a server, times
launch → `initialize` response over `RUNS` (default 20) iterations, then keeps
one instance alive to read its RSS from `/proc`:

```sh
bench.sh <label> -- <command to spawn the server...>
```

## Reproducing

The servers are fetched with [Nix](https://nixos.org) so no global Node/Python is
needed; adapt to your own toolchains as you like.

**ferric-fred (this repo):**

```sh
cargo build --release -p ferric-fred-mcp
bench.sh ferric-fred -- target/release/fred-mcp
```

**fred-mcp-server (Node):**

```sh
mkdir node && cd node && npm init -y && npm install fred-mcp-server
nix shell nixpkgs#nodejs_22 --command \
  ../bench.sh fred-mcp-node -- node node_modules/fred-mcp-server/build/index.js
```

**mcp-fredapi (Python):**

```sh
git clone --depth 1 https://github.com/Jaldekoa/mcp-fredapi && cd mcp-fredapi
nix shell nixpkgs#python313 nixpkgs#uv --command bash -c '
  uv venv .venv && uv pip install --python .venv/bin/python "mcp[cli]" httpx python-dotenv'
nix shell nixpkgs#python313 nixpkgs#uv --command \
  ../bench.sh mcp-fredapi -- .venv/bin/mcp run server.py
```

## Caveats

- **Startup and memory, not answers.** As above — the FRED round-trip dominates
  actual query time, and that is the same for every server here.
- **Favourable to the interpreters.** These spawn the servers directly with
  dependencies already installed. Running the Node server via `npx` or the Python
  one via `uvx` adds package-resolution overhead on top of the numbers above.
- **One implementation per language.** `fred-mcp-server` and `mcp-fredapi` are
  each one project among several; a different Node or Python server would land
  elsewhere. They are reasonable, real representatives, not a language verdict.
- **One machine, warm cache.** A first-ever spawn (binary/modules not in the page
  cache) is slower for everyone; this measures steady-state re-spawns, which is
  what a client actually does.
