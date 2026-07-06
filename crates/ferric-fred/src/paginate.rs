//! Auto-pagination: fetch every page of a paginated FRED endpoint in one call.
//!
//! FRED's list endpoints return one page at a time, alongside `count` (the total
//! number of results across *all* pages), `offset`, and `limit`. Walking those
//! pages by hand — bumping `offset` by `limit` until `offset >= count` — is
//! mechanical and easy to get wrong, so [`Paginate::send_all`] does it for you
//! and returns the full `Vec`.
//!
//! The two traits here are **sealed**: they are implemented for the crate's
//! request builders and `*Results` types and cannot be implemented downstream,
//! so the set of paginated endpoints stays closed and new methods can be added
//! without a breaking change (the same forward-compatibility posture as
//! [`Error`](crate::Error), ADR-0004). See ADR-0020 for the design.

use std::future::Future;
use std::time::Duration;

use chrono::NaiveDate;
use futures_core::Stream;

use crate::{
    Error, Release, ReleaseDate, ReleaseDatesResults, ReleasesResults, Result, Series,
    SeriesSearchResults, Source, SourcesResults, Tag, TagsResults, VintageDates,
};

pub(crate) mod sealed {
    /// Sealing marker: implemented only within this crate, so [`Page`](super::Page)
    /// and [`Paginate`](super::Paginate) cannot be implemented downstream.
    pub trait Sealed {}
}

/// One page of results from a paginated FRED endpoint.
///
/// Implemented for the crate's `*Results` types (and [`VintageDates`]). Sealed —
/// see the [module docs](self).
pub trait Page: sealed::Sealed {
    /// The element type of this page (e.g. [`Series`], [`Tag`]).
    type Item: Send;

    /// FRED's `count`: the total number of results across *all* pages, not just
    /// this one.
    fn total(&self) -> u32;

    /// The number of items on this page.
    fn items_len(&self) -> usize;

    /// Consume the page into its items.
    fn into_items(self) -> Vec<Self::Item>;
}

/// A request builder for a paginated FRED endpoint.
///
/// Its headline method is [`send_all`](Paginate::send_all), which pages an
/// endpoint to exhaustion. Sealed and implemented for the crate's paginated
/// request builders (see the [module docs](self)); you don't implement it.
pub trait Paginate: Clone + Send + Sized + sealed::Sealed {
    /// The page type this request returns.
    type Page: Page + Send;

    /// FRED's maximum page size for this endpoint — the largest `limit` it
    /// honors. 1000 for most lists; 10000 for release dates and vintage dates.
    const MAX_PAGE: u32;

    /// The caller's requested `limit`, if set. [`send_all`](Paginate::send_all)
    /// treats it as a *ceiling* on the total number of items returned.
    fn requested_limit(&self) -> Option<u32>;

    /// The caller's requested `offset`, if set — the point
    /// [`send_all`](Paginate::send_all) starts paging from.
    fn requested_offset(&self) -> Option<u32>;

    /// Return a copy of this request with `limit` and `offset` set.
    #[must_use]
    fn with_paging(self, limit: u32, offset: u32) -> Self;

    /// Send a single page — equivalent to the builder's own `send`.
    fn send_page(self) -> impl Future<Output = Result<Self::Page>> + Send;

    /// Fetch **every** result across all pages, in order.
    ///
    /// Pages are requested in chunks of at most [`MAX_PAGE`](Paginate::MAX_PAGE).
    /// A `limit` set on the builder caps the *total* number of items returned (a
    /// ceiling, not a per-page size); an `offset` set on the builder is the
    /// starting point. With neither set, the entire result set is returned.
    ///
    /// This issues up to ⌈`count` / `MAX_PAGE`⌉ HTTP requests. On a `429` it
    /// retries a bounded number of times, waiting FRED's `Retry-After` when
    /// present (see [`Error::RateLimited`]) and otherwise backing off. Mind
    /// FRED's rate limits when collecting large result sets.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// use ferric_fred::Paginate;
    /// // Every source FRED knows about, across as many pages as it takes:
    /// let sources = client.sources().send_all().await?;
    /// // At most 250 matches for a search, however many pages that spans:
    /// let series = client.search("gdp").limit(250).send_all().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns the first error encountered on any page (transport, API, or
    /// deserialize).
    fn send_all(self) -> impl Future<Output = Result<Vec<<Self::Page as Page>::Item>>> + Send {
        collect_all(self)
    }

    /// Stream every result across all pages, lazily — items are yielded as they
    /// arrive, and the next page is fetched only once the current one is drained.
    ///
    /// Same paging semantics as [`send_all`](Paginate::send_all) (a builder
    /// `limit` is a ceiling, `offset` is the start), but a `Stream` rather than a
    /// materialized `Vec`: memory stays flat regardless of the total, a consumer
    /// can stop early, and each item is a `Result` — a mid-stream error is
    /// surfaced as an `Err` item and ends the stream, so results retrieved before
    /// it aren't lost. On a `429` the same bounded retry as `send_all` applies.
    ///
    /// Drive it with [`StreamExt`](https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html)
    /// from the `futures` crate. The returned stream is `!Unpin`, so to consume it
    /// with `.next()` in a loop, pin it first (`pin_mut!`); the by-value
    /// combinators like `try_collect` / `for_each` need no pinning.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// use ferric_fred::Paginate;
    /// use futures_util::{pin_mut, StreamExt};
    ///
    /// let series = client.search("gdp").stream();
    /// pin_mut!(series);
    /// while let Some(item) = series.next().await {
    ///     let series = item?;
    ///     println!("{}", series.id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn stream(self) -> impl Stream<Item = Result<<Self::Page as Page>::Item>> + Send {
        stream_pages(self)
    }
}

/// Maximum number of retries for a single page when FRED returns `429`.
const MAX_RETRIES: u32 = 3;

/// Walk every page of `request`; see [`Paginate::send_all`].
async fn collect_all<P: Paginate>(request: P) -> Result<Vec<<P::Page as Page>::Item>> {
    let ceiling = request.requested_limit();
    let mut offset = request.requested_offset().unwrap_or(0);
    let mut collected: Vec<<P::Page as Page>::Item> = Vec::new();

    loop {
        // Under a ceiling, never fetch more than we still need.
        let page_size = match ceiling {
            Some(cap) => {
                let remaining = cap.saturating_sub(collected.len() as u32);
                if remaining == 0 {
                    break;
                }
                remaining.min(P::MAX_PAGE)
            }
            None => P::MAX_PAGE,
        };

        let page = send_page_with_retry(request.clone().with_paging(page_size, offset)).await?;
        let total = page.total();
        let got = page.items_len() as u32;
        collected.extend(page.into_items());
        offset = offset.saturating_add(got);

        // Stop on an empty page (guards against a misreported `count`) or once
        // we've walked past the total.
        if got == 0 || offset >= total {
            break;
        }
    }

    // A ceiling that lands mid-page: trim the overshoot.
    if let Some(cap) = ceiling {
        collected.truncate(cap as usize);
    }
    Ok(collected)
}

/// Stream every page of `request`, one item at a time; see [`Paginate::stream`].
///
/// A page is fetched only when the previous one has been fully yielded, so at
/// most one page is held in memory at a time. An error ends the stream after
/// surfacing it as an `Err` item.
fn stream_pages<P: Paginate>(
    request: P,
) -> impl Stream<Item = Result<<P::Page as Page>::Item>> + Send {
    async_stream::try_stream! {
        let ceiling = request.requested_limit();
        let mut offset = request.requested_offset().unwrap_or(0);
        let mut yielded: u32 = 0;

        loop {
            let page_size = match ceiling {
                Some(cap) => {
                    let remaining = cap.saturating_sub(yielded);
                    if remaining == 0 {
                        break;
                    }
                    remaining.min(P::MAX_PAGE)
                }
                None => P::MAX_PAGE,
            };

            let page = send_page_with_retry(request.clone().with_paging(page_size, offset)).await?;
            let total = page.total();
            let got = page.items_len() as u32;
            if got == 0 {
                break;
            }

            for item in page.into_items() {
                yield item;
                yielded += 1;
                if ceiling.is_some_and(|cap| yielded >= cap) {
                    break;
                }
            }

            offset = offset.saturating_add(got);
            if offset >= total || ceiling.is_some_and(|cap| yielded >= cap) {
                break;
            }
        }
    }
}

/// Send one page, retrying on `429` up to [`MAX_RETRIES`] times — waiting FRED's
/// `Retry-After` when it provides one, otherwise backing off exponentially.
async fn send_page_with_retry<P: Paginate>(request: P) -> Result<P::Page> {
    let mut attempt = 0;
    loop {
        match request.clone().send_page().await {
            Err(Error::RateLimited { retry_after }) if attempt < MAX_RETRIES => {
                let delay = retry_after.unwrap_or_else(|| backoff(attempt));
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
            result => return result,
        }
    }
}

/// Exponential backoff for a 0-based retry `attempt`: 1s, 2s, 4s.
fn backoff(attempt: u32) -> Duration {
    Duration::from_secs(1u64 << attempt)
}

impl sealed::Sealed for SeriesSearchResults {}
impl Page for SeriesSearchResults {
    type Item = Series;
    fn total(&self) -> u32 {
        self.count
    }
    fn items_len(&self) -> usize {
        self.series.len()
    }
    fn into_items(self) -> Vec<Self::Item> {
        self.series
    }
}

impl sealed::Sealed for TagsResults {}
impl Page for TagsResults {
    type Item = Tag;
    fn total(&self) -> u32 {
        self.count
    }
    fn items_len(&self) -> usize {
        self.tags.len()
    }
    fn into_items(self) -> Vec<Self::Item> {
        self.tags
    }
}

impl sealed::Sealed for ReleasesResults {}
impl Page for ReleasesResults {
    type Item = Release;
    fn total(&self) -> u32 {
        self.count
    }
    fn items_len(&self) -> usize {
        self.releases.len()
    }
    fn into_items(self) -> Vec<Self::Item> {
        self.releases
    }
}

impl sealed::Sealed for ReleaseDatesResults {}
impl Page for ReleaseDatesResults {
    type Item = ReleaseDate;
    fn total(&self) -> u32 {
        self.count
    }
    fn items_len(&self) -> usize {
        self.release_dates.len()
    }
    fn into_items(self) -> Vec<Self::Item> {
        self.release_dates
    }
}

impl sealed::Sealed for SourcesResults {}
impl Page for SourcesResults {
    type Item = Source;
    fn total(&self) -> u32 {
        self.count
    }
    fn items_len(&self) -> usize {
        self.sources.len()
    }
    fn into_items(self) -> Vec<Self::Item> {
        self.sources
    }
}

impl sealed::Sealed for VintageDates {}
impl Page for VintageDates {
    type Item = NaiveDate;
    fn total(&self) -> u32 {
        self.count
    }
    fn items_len(&self) -> usize {
        self.vintage_dates.len()
    }
    fn into_items(self) -> Vec<Self::Item> {
        self.vintage_dates
    }
}
