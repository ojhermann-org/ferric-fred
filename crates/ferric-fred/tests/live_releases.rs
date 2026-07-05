//! Live tests for the release endpoints; hit the real FRED API and require
//! `FRED_API_KEY`. Run with `--run-ignored all` (nextest) inside the dev shell.

use ferric_fred::{Client, ReleaseId};

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn releases_list_and_single_and_series() {
    let client = Client::from_env().expect("FRED_API_KEY set");

    // The full list of releases is non-empty.
    let results = client.releases().limit(5).send().await.expect("releases");
    assert!(results.count > 0);
    assert!(!results.releases.is_empty());

    // 53 = "Gross Domestic Product", a stable release.
    let release = client
        .release(ReleaseId::new(53))
        .await
        .expect("release 53");
    assert_eq!(release.id, ReleaseId::new(53));
    assert!(!release.name.is_empty());

    // Its series.
    let series = client
        .release_series(ReleaseId::new(53))
        .limit(1)
        .send()
        .await
        .expect("release 53 series");
    assert!(series.count > 0);

    // Its sources (unpaginated) — GDP is produced by at least one source.
    let sources = client
        .release_sources(ReleaseId::new(53))
        .await
        .expect("release 53 sources");
    assert!(!sources.is_empty());
    assert!(sources.iter().all(|source| !source.name.is_empty()));
}

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn release_dates_all_and_single() {
    let client = Client::from_env().expect("FRED_API_KEY set");

    // The cross-FRED release calendar is non-empty and names each release.
    let calendar = client
        .releases_dates()
        .limit(5)
        .send()
        .await
        .expect("releases/dates");
    assert!(calendar.count > 0);
    assert!(calendar
        .release_dates
        .iter()
        .all(|d| d.release_name.as_deref().is_some_and(|n| !n.is_empty())));

    // A single release's dates (53 = "Gross Domestic Product") omit the name.
    let dates = client
        .release_dates(ReleaseId::new(53))
        .limit(5)
        .send()
        .await
        .expect("release/dates");
    assert!(dates.count > 0);
    assert!(dates
        .release_dates
        .iter()
        .all(|d| d.release_id == ReleaseId::new(53)));
}
