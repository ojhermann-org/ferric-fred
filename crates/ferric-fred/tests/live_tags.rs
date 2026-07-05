//! Live tests for the tag endpoints; hit the real FRED API and require
//! `FRED_API_KEY`. Run with `--run-ignored all` (nextest) inside the dev shell.

use ferric_fred::{CategoryId, Client, ReleaseId, SeriesId};

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

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn scoped_tag_facets_resolve() {
    let client = Client::from_env().expect("FRED_API_KEY set");

    // Tags on a category (125 = Trade Balance) and those co-occurring with a seed.
    let cat_tags = client
        .category_tags(CategoryId::new(125))
        .limit(5)
        .send()
        .await
        .expect("category/tags");
    assert!(cat_tags.count > 0);
    let cat_related = client
        .category_related_tags(CategoryId::new(125), ["trade"])
        .limit(5)
        .send()
        .await
        .expect("category/related_tags");
    assert!(cat_related.count > 0);

    // Tags on a release (53 = Gross Domestic Product) and related.
    let rel_tags = client
        .release_tags(ReleaseId::new(53))
        .limit(5)
        .send()
        .await
        .expect("release/tags");
    assert!(rel_tags.count > 0);
    let rel_related = client
        .release_related_tags(ReleaseId::new(53), ["gdp"])
        .limit(5)
        .send()
        .await
        .expect("release/related_tags");
    assert!(rel_related.count > 0);

    // Tag facets of a full-text search, and related tags within it. The tag
    // filter goes out as `tag_search_text` (a live 400 would prove otherwise).
    let search_tags = client
        .series_search_tags("unemployment")
        .search_text("rate")
        .limit(5)
        .send()
        .await
        .expect("series/search/tags");
    assert!(search_tags.count > 0);
    let search_related = client
        .series_search_related_tags("unemployment", ["monthly"])
        .limit(5)
        .send()
        .await
        .expect("series/search/related_tags");
    assert!(search_related.count > 0);
}
