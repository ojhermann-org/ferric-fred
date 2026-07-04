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
}
