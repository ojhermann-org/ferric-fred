//! Live integration test against the real FRED API.
//!
//! Ignored by default so `cargo test` stays offline and deterministic. Run it
//! explicitly with a valid `FRED_API_KEY` in the environment:
//!
//! ```sh
//! cargo test -p ferric-fred --test live_observations -- --ignored
//! # or: cargo nextest run -p ferric-fred --run-ignored all
//! ```

use ferric_fred::{Client, SeriesId};

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn fetches_gnpca_observations() {
    let client = Client::from_env().expect("FRED_API_KEY should be set for the live test");

    let observations = client
        .observations(&SeriesId::new("GNPCA"))
        .await
        .expect("observations request should succeed");

    assert!(!observations.is_empty(), "GNPCA should return observations");
    assert!(
        observations.iter().any(|o| o.value.is_some()),
        "at least one observation should have a value"
    );
}
