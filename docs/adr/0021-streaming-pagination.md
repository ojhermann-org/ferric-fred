# ADR-0021: Streaming pagination (`Paginate::stream`)

- **Status:** Accepted
- **Date:** 2026-07-06
- **Deciders:** Project owner

## Context

[ADR-0020](0020-auto-pagination.md) added `Paginate::send_all`, which walks a
paginated endpoint to exhaustion and returns the full `Vec`. That is the right
default, but it has two costs a caller can't opt out of: it holds every result in
memory at once, and it either returns everything or fails — a page error late in
a large walk discards the results already fetched.

Some callers want to process results as they arrive (write each to a database,
filter and stop early, bound memory regardless of the total). The `Paginate`
trait from ADR-0020 already abstracts the page walk, so a lazy variant is a small
addition on the same scaffolding. ADR-0020 deferred it here so its new
dependencies could be weighed on their own.

## Decision

We will add a provided **`Paginate::stream`** method that returns a lazy
`Stream`:

```rust
use ferric_fred::Paginate;
use futures_util::StreamExt;

let mut series = client.search("gdp").stream();
while let Some(item) = series.next().await {
    let series = item?;               // each item is a Result
    // …process one series, holding at most one page in memory…
}
```

- **`fn stream(self) -> impl Stream<Item = Result<Item>> + Send`.** Items are
  yielded one at a time; the next page is fetched only when the current one is
  drained, so memory stays flat. Each item is a `Result`: a mid-stream error is
  surfaced as a final `Err` item and ends the stream, so results already yielded
  are not lost (unlike `send_all`, which discards them). Paging semantics match
  `send_all` — a builder `limit` is a ceiling, `offset` is the start — and the
  same bounded `429` retry applies.
- **Two new dependencies, both small and ubiquitous:** `futures-core` (for the
  `Stream` trait in the public signature) and `async-stream` (the `try_stream!`
  generator that implements the walk). `futures-core` is the stable, minimal home
  of the `Stream` trait; `async-stream` lets the paging loop read like the
  `send_all` loop rather than a hand-written `poll_next` state machine.
- **Consumers bring their own `StreamExt`.** We return a bare `Stream` and don't
  re-export a combinator extension trait; callers use `futures`/`futures-util`
  (or `tokio-stream`) to drive it. `futures-util` is a dev-dependency here, for
  tests and the doc example only.
- **`send_all` stays.** It remains the ergonomic default; `stream` is the
  lower-level, memory-bounded alternative. `send_all` is not reimplemented on top
  of `stream` — both call the shared `send_page_with_retry`, keeping each a
  direct, readable loop.

## Consequences

- Callers get constant-memory iteration, early termination, and partial results
  on error — the cases `send_all` can't serve.
- The library takes two new direct dependencies. They are in the `futures`
  ecosystem that `reqwest`/`tokio` already pull in transitively (ADR-0003), so the
  added build cost is marginal, but they are a real surface and are called out in
  `Cargo.toml`.
- The public API now exposes a `futures_core::Stream`, tying that trait's version
  (`0.3`, long stable) into our semver surface. Acceptable: `Stream` in `std` is
  not yet stable, and `futures-core`'s `Stream` is the de-facto standard.
- Two ways to page (`send_all` vs `stream`) is a slightly larger API to document;
  we frame `send_all` as the default and `stream` as the memory-bounded option.

## Alternatives considered

- **A hand-rolled `Stream` over `futures-core` only** (a manual `poll_next`, or
  `futures-util::stream::unfold`) — avoids the `async-stream` dependency, but the
  buffered-page state machine is materially more code and easier to get wrong than
  the `try_stream!` loop, which mirrors `send_all`. Rejected: the macro dependency
  is tiny and the shared shape is worth more than dropping it.
- **Feature-gating `stream` behind an off-by-default feature** — keeps the default
  build free of the two deps, but splits the API across feature flags and adds
  config surface for a small, widely-wanted capability. Rejected in favour of
  always-on; revisit if the dependency weight ever matters.
- **Only `stream`, dropping `send_all`** — one way to page, but the common "just
  give me the Vec" case would then require every caller to pull in `StreamExt` and
  collect. Rejected: `send_all` is the friendlier default.
