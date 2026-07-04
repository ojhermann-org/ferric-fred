//! Live tests for the tag endpoints; hit the real FRED API and require
//! `FRED_API_KEY`. Run with `--run-ignored all` (nextest) inside the dev shell.

use ferric_fred::{Client, SeriesId};

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn tags_search_series_and_series_tags() {
    let client = Client::from_env().expect("FRED_API_KEY set");

    // Searching the tag vocabulary finds the "gdp" tag.
    let tags = client
        .tags()
        .search_text("gdp")
        .limit(5)
        .send()
        .await
        .expect("tags search");
    assert!(tags.count > 0);

    // Series carrying both "gdp" and "quarterly".
    let series = client
        .tags_series(["gdp", "quarterly"])
        .limit(1)
        .send()
        .await
        .expect("tags/series");
    assert!(series.count > 0);

    // The reverse: a series' own tags.
    let series_tags = client
        .series_tags(&SeriesId::new("GNPCA"))
        .await
        .expect("series/tags");
    assert!(!series_tags.tags.is_empty());

    // Tags co-occurring with "gdp".
    let related = client
        .related_tags(["gdp"])
        .limit(5)
        .send()
        .await
        .expect("related_tags");
    assert!(related.count > 0);
}
