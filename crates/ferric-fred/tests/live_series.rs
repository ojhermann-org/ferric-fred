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

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn series_reverse_lookups_resolve() {
    let client = Client::from_env().expect("FRED_API_KEY should be set for the live test");
    let gnpca = SeriesId::new("GNPCA");

    // The categories GNPCA belongs to.
    let categories = client
        .series_categories(&gnpca)
        .await
        .expect("series/categories");
    assert!(!categories.is_empty());

    // The release GNPCA belongs to.
    let release = client.series_release(&gnpca).await.expect("series/release");
    assert!(!release.name.is_empty());
}

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn series_updates_returns_recently_updated() {
    use ferric_fred::UpdatesFilter;

    let client = Client::from_env().expect("FRED_API_KEY should be set for the live test");

    let results = client
        .series_updates()
        .filter(UpdatesFilter::Macro)
        .limit(3)
        .send()
        .await
        .expect("series/updates");
    assert!(results.count > 0);
    assert!(!results.series.is_empty());
}

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn series_vintagedates_resolve() {
    let client = Client::from_env().expect("FRED_API_KEY should be set for the live test");

    let dates = client
        .series_vintagedates(&SeriesId::new("GNPCA"))
        .limit(5)
        .send()
        .await
        .expect("series/vintagedates");
    assert!(dates.count > 0);
    assert!(!dates.vintage_dates.is_empty());
}
