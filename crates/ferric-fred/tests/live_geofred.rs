//! Live tests for the GeoFRED / Maps endpoints; hit the real FRED API and
//! require `FRED_API_KEY`. Run with `--run-ignored all` (nextest) inside the dev
//! shell.

use chrono::NaiveDate;
use ferric_fred::{
    Client, Error, Frequency, RegionType, SeasonalAdjustment, SeriesGroupId, SeriesId, ShapeType,
};

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn regional_data_cross_section() {
    let client = Client::from_env().expect("FRED_API_KEY set");

    // Series group 882 = Per Capita Personal Income; a state cross-section.
    let data = client
        .regional_data(
            &SeriesGroupId::new("882"),
            RegionType::State,
            NaiveDate::from_ymd_opt(2013, 1, 1).unwrap(),
            "Dollars",
            Frequency::Annual,
            SeasonalAdjustment::NotSeasonallyAdjusted,
        )
        .await
        .expect("regional data");

    assert_eq!(data.meta.region, "state");
    let day = data
        .meta
        .data
        .get("2013-01-01")
        .expect("the requested date");
    // The 50 states plus D.C.
    assert!(
        day.len() >= 50,
        "expected a value per state, got {}",
        day.len()
    );
    assert!(day.iter().all(|p| !p.region.is_empty()));
}

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn series_data_over_time() {
    let client = Client::from_env().expect("FRED_API_KEY set");

    // A Wyoming total-private-employment regional series.
    let data = client
        .series_data(&SeriesId::new("SMU56000000500000001"))
        .send()
        .await
        .expect("series data");

    assert!(!data.meta.data.is_empty());
    // Every date bucket has at least one region row.
    assert!(data.meta.data.values().all(|rows| !rows.is_empty()));
}

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn series_group_metadata() {
    let client = Client::from_env().expect("FRED_API_KEY set");

    let group = client
        .series_group(&SeriesId::new("SMU56000000500000001"))
        .await
        .expect("series group");

    assert!(!group.title.is_empty());
    assert_eq!(group.region_type, "state");
    assert!(group.min_date <= group.max_date);
}

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn non_regional_series_group_gives_actionable_error() {
    let client = Client::from_env().expect("FRED_API_KEY set");

    // GNPCA is a valid *macro* series but has no regional data; FRED answers the
    // GeoFRED endpoint with a bare HTTP 500. We rewrite it into an actionable
    // message naming the id and pointing at the non-regional cause (#56).
    let error = client
        .series_group(&SeriesId::new("GNPCA"))
        .await
        .expect_err("a non-regional series should error");

    match error {
        Error::Api {
            status, message, ..
        } => {
            assert_eq!(status, 500);
            assert_ne!(message, "Internal Server Error");
            assert!(
                message.contains("series_id=GNPCA"),
                "message was {message:?}"
            );
            assert!(message.contains("regional"), "message was {message:?}");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn shape_file_is_geojson() {
    let client = Client::from_env().expect("FRED_API_KEY set");

    let shapes = client.shape_file(ShapeType::Bea).await.expect("shape file");

    assert_eq!(shapes.kind, "FeatureCollection");
    assert!(!shapes.features.is_empty());
    assert!(shapes.features.iter().all(|f| !f.geometry.kind.is_empty()));
}
