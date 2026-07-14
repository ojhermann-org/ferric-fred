# Contributing to ferric-fred

Thanks for your interest! `ferric-fred` is a typed Rust client for the
[FRED](https://fred.stlouisfed.org/) API, plus a CLI and an MCP server. Bug
reports, questions, and pull requests are all welcome.

By contributing, you agree that your contributions are licensed under the
project's dual **MIT OR Apache-2.0** license (see
[ADR-0006](docs/adr/0006-license.md) and `LICENSE-MIT` / `LICENSE-APACHE`).

## Getting set up

A [Nix](https://nixos.org/) flake supplies a reproducible toolchain (recent
stable Rust, `cargo-nextest`, `cargo-deny`, `gitleaks`, and the rest):

```sh
nix develop        # enter the dev shell
# or, with direnv: `direnv allow` once, then it loads automatically
```

Nix is optional — a normal `rustup` toolchain works too; the flake supplies the
environment, not the build ([ADR-0008](docs/adr/0008-nix-flake-dev-environment.md)).

Most tests run offline (HTTP is mocked — [ADR-0011](docs/adr/0011-testing-strategy.md)).
The `#[ignore]`d live tests hit the real FRED API and need a free key in
`FRED_API_KEY` (get one at <https://fredaccount.stlouisfed.org/apikeys>); see the
README's [Secrets](README.md#secrets) section.

## The gate

CI runs the same checks the `pre-push` hook does, through the same flake, so a
push that passes locally passes CI. Before opening a PR, make sure these are
green:

```sh
cargo fmt --all                          # formatting (CI checks --check)
cargo clippy --all-targets -- -D warnings  # lints; warnings are errors
cargo nextest run                        # the offline test suite
cargo test --doc                         # doctests
cargo deny check                         # licenses, advisories, bans, sources
```

Enable the tracked git hooks once per clone (`core.hooksPath` is local config and
isn't carried by git):

```sh
git config core.hooksPath .githooks
```

`pre-commit` is a secret guard ([ADR-0014](docs/adr/0014-pre-commit-secret-guard.md));
`pre-push` runs fmt + clippy + tests.

## Benchmarks

Perf tooling from the Tech Radar pilot ([ADR-0026](docs/adr/0026-perf-tooling-pilot.md)).
They are not part of the gate — run them when a change might affect the
deserialization hot path or CLI startup:

```sh
cargo bench -p ferric-fred --bench deserialization   # divan: observations parse
scripts/bench-cli.sh                                  # hyperfine: CLI wall-clock
```

CI checks that the benches compile on every PR (`cargo bench --no-run`), and a
separate `bench.yml` uploads results to [Bencher](https://bencher.dev) to track
them and comment regressions on the PR (the criterion mirror + hyperfine startup;
Bencher has no divan adapter). If you touch `Observation`/`ReleaseTable`
deserialization or the CLI's startup path, run the relevant bench locally and
note the before/after in your PR too.

## Agent-driven MCP audit

Beyond the offline tests, the MCP server has an **agent-driven audit**
([ADR-0028](docs/adr/0028-agent-driven-mcp-testing.md)): a headless Claude agent
drives the `ferric-fred-mcp` tools against live FRED and reports defects a
deterministic test misses — confusing tool descriptions, input/output-schema
gaps, and rough error handling. It needs a live `FRED_API_KEY` and the `claude`
CLI (both in the dev shell), so it is **not** part of the CI gate.

```sh
direnv exec . scripts/mcp-agent-audit.sh   # writes target/mcp-audit/findings.md
```

Run it **on any change to the MCP surface** (tools, descriptions, annotations,
schemas), alongside the Glama re-score that change already requires; distil the
findings into issues. The committed `.mcp.json` also registers the server for any
Claude Code session in the repo, so you can use the FRED tools interactively while
developing (it holds no secret — the server reads `FRED_API_KEY` from the
environment).

## Commit messages

Commits follow [Conventional Commits](https://www.conventionalcommits.org/) —
they are load-bearing: [`release-plz`](https://release-plz.dev/) derives each
crate's version bump and changelog from them
([ADR-0012](docs/adr/0012-ci-versioning-and-release.md)).

Use `feat`, `fix`, `docs`, `test`, `ci`, `build`, `refactor`, `style`, or
`chore`; a `!` (e.g. `feat!:`) or a `BREAKING CHANGE:` footer marks a breaking
change. Keep each commit to one logical layer where practical.

## Adding an endpoint

New FRED endpoints follow the vertical-slice pattern in
[ADR-0013](docs/adr/0013-endpoint-addition-pattern.md): a typed request/response
in the library, then the CLI subcommand, then the MCP tool — with offline
(wiremock) tests plus an `#[ignore]`d live test at each layer, one commit per
layer. Significant design choices get their own ADR (copy
`docs/adr/0000-adr-template.md`, take the next number, add it to
[the index](docs/adr/README.md)).

## Pull requests

1. Fork the repo and branch off `main`.
2. Make your change with tests; keep the gate green.
3. Open a PR against `main`. CI must pass; a maintainer will review and merge.

`main` is protected: changes land through PRs, and the CI check must pass before
merge. Please be respectful and constructive in issues and reviews — assume good
faith and keep discussion focused on the work.
