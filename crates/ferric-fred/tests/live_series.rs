//! Live integration test for the `series` metadata endpoint. Ignored by default;
//! run explicitly with a valid `FRED_API_KEY`:
//!
//! ```sh
//! cargo nextest run -p ferric-fred --run-ignored all
//! ```

use ferric_fred::{Client, Frequency, SeasonalAdjustment, SeriesId};

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn fetches_gnpca_series_metadata() {
    let client = Client::from_env().expect("FRED_API_KEY should be set for the live test");

    let series = client
        .series(&SeriesId::new("GNPCA"))
        .await
        .expect("series request should succeed");

    assert_eq!(series.id, SeriesId::new("GNPCA"));
    assert!(
        series.title.contains("Gross National Product"),
        "unexpected title: {}",
        series.title
    );
    assert_eq!(series.frequency, Frequency::Annual);
    assert_eq!(
        series.seasonal_adjustment,
        SeasonalAdjustment::NotSeasonallyAdjusted
    );
    assert!(series.observation_start <= series.observation_end);
}
