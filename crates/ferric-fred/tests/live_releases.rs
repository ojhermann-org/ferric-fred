//! Live tests for the release endpoints; hit the real FRED API and require
//! `FRED_API_KEY`. Run with `--run-ignored all` (nextest) inside the dev shell.

use chrono::NaiveDate;
use ferric_fred::{Client, ReleaseElementId, ReleaseId};

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

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn release_tables_tree_and_subtree() {
    let client = Client::from_env().expect("FRED_API_KEY set");

    // 10 = the Consumer Price Index release, which has a table tree.
    let table = client
        .release_tables(ReleaseId::new(10))
        .send()
        .await
        .expect("release/tables");
    assert!(!table.roots.is_empty());
    assert!(table.roots.iter().all(|element| !element.name.is_empty()));

    // Drilling into a root by element_id returns that subtree, now naming the
    // requested element.
    let root_id = table.roots[0].element_id;
    let subtree = client
        .release_tables(ReleaseId::new(10))
        .element(root_id)
        .send()
        .await
        .expect("release/tables subtree");
    assert_eq!(subtree.element_id, Some(root_id));
}

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn release_tables_observation_values_surface() {
    let client = Client::from_env().expect("FRED_API_KEY set");
    let date = NaiveDate::from_ymd_opt(2023, 6, 1).unwrap();

    // FRED returns one level of children per request for this release, so walk
    // down by element_id — carrying observation values the whole way — until a
    // `series` row surfaces its value. This both proves the request params reach
    // the wire and pins the response field names/format against API drift.
    let roots = client
        .release_tables(ReleaseId::new(10))
        .observation_date(date)
        .send()
        .await
        .expect("release/tables with values")
        .roots;

    // Depth-first drill with a bounded request budget; stops at the first value.
    let mut frontier: Vec<ReleaseElementId> = roots.iter().map(|e| e.element_id).collect();
    let mut value: Option<f64> = roots.iter().find_map(|e| e.observation_value);
    let mut budget = 24;
    while value.is_none() && budget > 0 {
        let Some(element_id) = frontier.pop() else {
            break;
        };
        budget -= 1;
        let children = client
            .release_tables(ReleaseId::new(10))
            .element(element_id)
            .observation_date(date)
            .send()
            .await
            .expect("release/tables subtree with values")
            .roots;
        value = children.iter().find_map(|e| e.observation_value);
        frontier.extend(children.iter().map(|e| e.element_id));
    }

    let value = value.expect("a series element should carry an observation value");
    assert!(value.is_finite());
}
