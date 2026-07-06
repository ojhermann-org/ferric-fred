# ADR-0020: Auto-pagination (`Paginate` trait + `send_all`)

- **Status:** Accepted
- **Date:** 2026-07-06
- **Deciders:** Project owner

## Context

FRED's list endpoints (`series/search`, `category/series`, `release/series`,
`tags/series`, `tags`, `releases`, `releases/dates`, `sources`, `series/updates`,
`series/vintagedates`, …) are paginated: each response carries `count` (the total
number of results across *all* pages), `offset`, and `limit`, and returns at most
`limit` items. To retrieve everything, a caller has to loop by hand — bump
`.offset()` by the page size and re-`.send()` until `offset >= count`. That is
mechanical, easy to get subtly wrong (off-by-one, a misreported `count`), and the
same for every endpoint.

The builders are uniform (all `Clone`, all with `.limit()` / `.offset()` and an
async `send()`), and every result type exposes `count` plus a `Vec` of items —
so the walk can be written once against a small abstraction. This is
[issue #8](https://github.com/ojhermann-org/ferric-fred/issues/8).

## Decision

We will add a **`Paginate` trait** with a provided **`send_all`** method that
pages an endpoint to exhaustion and returns the full `Vec`.

```rust
let all_sources = client.sources().send_all().await?;             // everything
let top_250     = client.search("gdp").limit(250).send_all().await?; // capped
```

- **Two sealed traits** in `paginate.rs`. `Page` abstracts a result page (`total()`,
  `items_len()`, `into_items()`); `Paginate` abstracts a request builder
  (`MAX_PAGE`, `with_paging`, `send_page`, the requested limit/offset). Both are
  sealed via a private `sealed::Sealed` supertrait, so the set of paginated types
  stays closed and we can add methods later without a breaking change — the same
  forward-compatibility posture as [ADR-0004](0004-error-handling.md).
- **`send_all` is a provided method** returning `impl Future<…> + Send`. The
  explicit `+ Send` bound (rather than a bare `async fn` in the trait) keeps the
  returned future usable on a multi-threaded runtime, which is the common case for
  consumers. The paging logic lives once in a free `collect_all` function.
- **A set `.limit(n)` is a ceiling on the total returned**, not a per-page size:
  `send_all` fetches in chunks of at most `MAX_PAGE` until it has `n` items (or FRED
  is exhausted), then truncates. A set `.offset()` is the starting point. With
  neither set, it returns the entire result set.
- **Per-endpoint `MAX_PAGE`.** Most lists cap at 1000; `releases/dates` and
  `series/vintagedates` cap at 10000. Each builder declares its own constant.
- **Bounded 429 handling.** On a rate-limit response mid-walk, `send_all` retries a
  small, fixed number of times, waiting FRED's `Retry-After` when present and
  otherwise backing off exponentially (1s, 2s, 4s). To support this, `api_error`
  now parses the `Retry-After` header into `Error::RateLimited { retry_after }`
  (previously always `None`), and the library takes a single new tokio feature,
  `time`, for the async sleep.
- **`observations` is out of scope.** It returns a bare `Vec<Observation>` (its
  page envelope is discarded today) and its `limit` maxes at 100000, not 1000; a
  separate change would give it a `count`-bearing result type first.

## Consequences

- Callers get one-line "fetch everything" ergonomics across ten endpoints, with the
  paging arithmetic written and tested once.
- The library gains its first direct tokio dependency — a single feature (`time`),
  for the retry sleep. reqwest already pulls tokio transitively, so this is a small,
  honest addition rather than a new runtime coupling, but it *is* a real dependency
  and is called out here and in `Cargo.toml`.
- `send_all` can issue many requests for a large result set (⌈`count` / `MAX_PAGE`⌉).
  We document that and handle 429s defensively, but a full client-wide rate limiter
  is deliberately left to a separate change — `send_all`'s bounded retry is not a
  substitute for pacing sustained traffic.
- The sealed traits are additive: existing `send()` calls and result types are
  unchanged. `Retry-After` is now populated, which only adds information to an
  already-existing error variant.
- Streaming (a lazy `Stream` that yields items as pages arrive) is a natural next
  step on the same `Paginate` scaffolding and is deferred to its own change, so its
  new dependencies (`futures-core`, `async-stream`) can be weighed separately.

## Alternatives considered

- **A bare `async fn send_all` in the trait** — simplest to write, but its returned
  future isn't guaranteed `Send`, which breaks callers on a multi-threaded tokio
  runtime. Rejected in favour of an explicit `-> impl Future + Send`.
- **An inherent `send_all` copy-pasted onto each builder** — no trait, but duplicates
  the (non-trivial) paging + retry loop eight times. Rejected: the trait writes it
  once and is the exact surface a future `stream()` reuses.
- **`.limit()` as a per-page size** (auto-paging always fetches *everything*, ignoring
  the ceiling) — surprising, since `.limit(n)` already means "at most n" on a single
  `send()`. Rejected: treating it as a ceiling keeps one meaning for the method.
- **A full client-side token-bucket rate limiter now** — larger scope, and it belongs
  to the separate rate-limit/retry idea the issue calls out. Rejected for this change
  in favour of bounded per-page retry plus documentation.
