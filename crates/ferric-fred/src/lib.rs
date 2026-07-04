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
mod category_series_request;
mod client;
mod error;
mod frequency;
mod ids;
mod observation;
mod observations_request;
mod order_by;
mod search_type;
mod seasonal_adjustment;
mod series;
mod series_search_request;
mod sort_order;
mod units;

pub use aggregation_method::AggregationMethod;
pub use category::Category;
pub use category_series_request::CategorySeriesRequest;
pub use client::Client;
pub use error::Error;
pub use frequency::Frequency;
pub use ids::{CategoryId, SeriesId};
pub use observation::Observation;
pub use observations_request::ObservationsRequest;
pub use order_by::OrderBy;
pub use search_type::SearchType;
pub use seasonal_adjustment::SeasonalAdjustment;
pub use series::{Series, SeriesSearchResults};
pub use series_search_request::SeriesSearchRequest;
pub use sort_order::SortOrder;
pub use units::Units;

/// A `Result` whose error type is this crate's [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
