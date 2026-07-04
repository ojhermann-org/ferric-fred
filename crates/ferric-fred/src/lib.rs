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
//! let observations = client.observations(&SeriesId::new("GNPCA")).await?;
//! println!("{} observations", observations.len());
//! # Ok(())
//! # }
//! ```

mod client;
mod error;
mod frequency;
mod ids;
mod observation;
mod seasonal_adjustment;
mod series;

pub use client::Client;
pub use error::Error;
pub use frequency::Frequency;
pub use ids::SeriesId;
pub use observation::Observation;
pub use seasonal_adjustment::SeasonalAdjustment;
pub use series::Series;

/// A `Result` whose error type is this crate's [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
