//! `ferric-fred` — a strongly-typed async Rust client for the [FRED] API
//! (Federal Reserve Economic Data, from the Federal Reserve Bank of St. Louis).
//!
//! [FRED]: https://fred.stlouisfed.org/
//!
//! This crate is in early construction. The first implemented slice fetches a
//! series' observations end-to-end; see the design ADRs under `docs/adr/` for
//! the decisions that shape the API (async-first client, typed error taxonomy,
//! newtype identifiers, and forward-compatible domain modelling).
//!
//! ```no_run
//! # async fn run() -> ferric_fred::Result<()> {
//! use ferric_fred::{Client, SeriesId};
//!
//! let client = Client::from_env()?; // reads FRED_API_KEY
//! let observations = client.observations(&SeriesId::new("GNPCA")).send().await?;
//! println!("{} observations", observations.len());
//! # Ok(())
//! # }
//! ```

mod aggregation_method;
mod category;
mod client;
mod error;
mod frequency;
mod ids;
mod observation;
mod observations_request;
mod order_by;
mod release;
mod release_date;
mod release_dates_request;
mod releases_request;
mod search_type;
mod seasonal_adjustment;
mod series;
mod series_list_request;
mod series_search_request;
mod series_updates_request;
mod sort_order;
mod source;
mod sources_request;
mod tag;
mod tags_request;
mod units;
mod updates_filter;
mod vintage_dates;
mod vintage_dates_request;

pub use aggregation_method::AggregationMethod;
pub use category::Category;
pub use client::Client;
pub use error::Error;
pub use frequency::Frequency;
pub use ids::{CategoryId, ReleaseId, SeriesId, SourceId};
pub use observation::Observation;
pub use observations_request::ObservationsRequest;
pub use order_by::OrderBy;
pub use release::{Release, ReleasesResults};
pub use release_date::{ReleaseDate, ReleaseDatesResults};
pub use release_dates_request::ReleaseDatesRequest;
pub use releases_request::ReleasesRequest;
pub use search_type::SearchType;
pub use seasonal_adjustment::SeasonalAdjustment;
pub use series::{Series, SeriesSearchResults};
pub use series_list_request::SeriesListRequest;
pub use series_search_request::SeriesSearchRequest;
pub use series_updates_request::SeriesUpdatesRequest;
pub use sort_order::SortOrder;
pub use source::{Source, SourcesResults};
pub use sources_request::SourcesRequest;
pub use tag::{Tag, TagsResults};
pub use tags_request::TagsRequest;
pub use units::Units;
pub use updates_filter::UpdatesFilter;
pub use vintage_dates::VintageDates;
pub use vintage_dates_request::VintageDatesRequest;

/// A `Result` whose error type is this crate's [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
