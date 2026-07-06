//! Live integration tests for the observations endpoint. Ignored by default so
//! `cargo test` stays offline; run explicitly with a valid `FRED_API_KEY`:
//!
//! ```sh
//! cargo nextest run -p ferric-fred --run-ignored all
//! ```

use chrono::NaiveDate;
use ferric_fred::{Client, SeriesId, SortOrder, Units};

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn fetches_gnpca_observations() {
    let client = Client::from_env().expect("FRED_API_KEY should be set for the live test");

    let observations = client
        .observations(&SeriesId::new("GNPCA"))
        .send()
        .await
        .expect("observations request should succeed");

    assert!(!observations.is_empty(), "GNPCA should return observations");
    assert!(
        observations.iter().any(|o| o.value.is_some()),
        "at least one observation should have a value"
    );
}

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn honors_request_parameters() {
    let client = Client::from_env().expect("FRED_API_KEY should be set for the live test");

    let observations = client
        .observations(&SeriesId::new("GNPCA"))
        .units(Units::PercentChange)
        .sort_order(SortOrder::Descending)
        .limit(5)
        .send()
        .await
        .expect("parameterized observations request should succeed");

    assert_eq!(observations.len(), 5, "limit should cap the result count");
    for pair in observations.windows(2) {
        assert!(
            pair[0].date >= pair[1].date,
            "sort_order=desc should yield newest-first dates"
        );
    }
}

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn point_in_time_reflects_the_as_of_date() {
    let client = Client::from_env().expect("FRED_API_KEY should be set for the live test");
    let as_of = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();

    // GNPCA (annual real GNP) is revised, so it exercises ALFRED well.
    let snapshot = client
        .observations(&SeriesId::new("GNPCA"))
        .realtime(as_of, as_of)
        .send()
        .await
        .expect("point-in-time observations request should succeed");

    assert!(!snapshot.is_empty());
    // Every row carries the requested real-time period...
    assert!(
        snapshot
            .iter()
            .all(|o| o.realtime_start == as_of && o.realtime_end == as_of),
        "each row should report the requested as-of period"
    );
    // ...and nothing postdates the snapshot — no look-ahead.
    assert!(
        snapshot.iter().all(|o| o.date < as_of),
        "as of {as_of}, no future observation should be known"
    );

    // The point-in-time value for a revised year differs from today's latest.
    let latest = client
        .observations(&SeriesId::new("GNPCA"))
        .send()
        .await
        .expect("latest observations request should succeed");
    let year = NaiveDate::from_ymd_opt(2017, 1, 1).unwrap();
    let then = snapshot
        .iter()
        .find(|o| o.date == year)
        .and_then(|o| o.value);
    let now = latest.iter().find(|o| o.date == year).and_then(|o| o.value);
    assert!(
        then.is_some() && now.is_some(),
        "2017 should be present in both"
    );
    assert_ne!(then, now, "GNPCA 2017 was revised between 2020 and now");
}
