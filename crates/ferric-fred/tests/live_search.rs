//! Live integration test for the `series/search` endpoint. Ignored by default;
//! run explicitly with a valid `FRED_API_KEY`:
//!
//! ```sh
//! cargo nextest run -p ferric-fred --run-ignored all
//! ```

use ferric_fred::{Client, OrderBy, SortOrder};

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn searches_for_unemployment_series() {
    let client = Client::from_env().expect("FRED_API_KEY should be set for the live test");

    let results = client
        .search("unemployment rate")
        .order_by(OrderBy::Popularity)
        .sort_order(SortOrder::Descending)
        .limit(5)
        .send()
        .await
        .expect("search request should succeed");

    assert!(results.count > 0, "there should be matches");
    assert!(
        !results.series.is_empty(),
        "the results page should not be empty"
    );
    assert!(results.series.len() <= 5, "limit should cap the page size");
    assert!(
        results
            .series
            .iter()
            .any(|s| s.title.to_lowercase().contains("unemployment")),
        "a popularity-ranked search for 'unemployment rate' should surface an unemployment series"
    );
}
