use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::{Frequency, SeasonalAdjustment, SeriesId};

/// Metadata describing a FRED series (the `fred/series` endpoint).
///
/// This `series` metadata endpoint's own ALFRED fields (`realtime_start` /
/// `realtime_end`) are still ignored on the wire (ADR-0005); point-in-time
/// *observations* are supported via [`Observation`](crate::Observation) and
/// [`ObservationsRequest::realtime`](crate::ObservationsRequest::realtime)
/// (ADR-0024). `last_updated` is kept as FRED's raw string for now — FRED
/// encodes it with a non-standard timezone offset (e.g. `2024-03-28 07:56:03-05`);
/// a typed datetime is a later refinement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Series {
    /// The series identifier.
    pub id: SeriesId,

    /// Human-readable title, e.g. `"Real Gross National Product"`.
    pub title: String,

    /// Date of the earliest available observation.
    pub observation_start: NaiveDate,

    /// Date of the latest available observation.
    pub observation_end: NaiveDate,

    /// The series' native reporting frequency.
    pub frequency: Frequency,

    /// Whether/how the series is seasonally adjusted.
    pub seasonal_adjustment: SeasonalAdjustment,

    /// Free-form units description, e.g. `"Billions of Chained 2017 Dollars"`.
    /// This is descriptive text, *not* the closed-vocabulary units transform
    /// used in observation requests (modelled separately, later).
    pub units: String,

    /// FRED popularity score (0–100).
    pub popularity: u32,

    /// Editorial notes, when present.
    #[serde(default)]
    pub notes: Option<String>,

    /// When FRED last updated the series, as FRED's raw timestamp string.
    pub last_updated: String,
}

/// A page of `series/search` results: the matching series plus FRED's
/// pagination metadata. `count` is the total number of matches across all
/// pages, not just this one — use it with `offset`/`limit` to page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SeriesSearchResults {
    /// The matching series on this page. FRED names the array `seriess` (sic) on
    /// the wire; we read that but emit the correctly-spelled `series` on output.
    #[serde(rename(deserialize = "seriess", serialize = "series"))]
    pub series: Vec<Series>,

    /// Total number of matches across all pages.
    pub count: u32,

    /// Offset of this page into the full result set.
    pub offset: u32,

    /// Page-size limit that FRED applied.
    pub limit: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    // A representative `seriess[0]` object, including fields we intentionally
    // ignore (`realtime_*`, the `*_short` codes) to prove they don't break
    // deserialization.
    const GNPCA_JSON: &str = r#"{
        "id": "GNPCA",
        "realtime_start": "2024-01-01",
        "realtime_end": "2024-01-01",
        "title": "Real Gross National Product",
        "observation_start": "1929-01-01",
        "observation_end": "2023-01-01",
        "frequency": "Annual",
        "frequency_short": "A",
        "units": "Billions of Chained 2017 Dollars",
        "units_short": "Bil. of Chn. 2017 $",
        "seasonal_adjustment": "Not Seasonally Adjusted",
        "seasonal_adjustment_short": "NSA",
        "last_updated": "2024-03-28 07:56:03-05",
        "popularity": 76,
        "notes": "BEA Account Code: A001RX"
    }"#;

    #[test]
    fn deserializes_series_metadata() {
        let series: Series = serde_json::from_str(GNPCA_JSON).unwrap();

        assert_eq!(series.id, SeriesId::new("GNPCA"));
        assert_eq!(series.title, "Real Gross National Product");
        assert_eq!(
            series.observation_start,
            NaiveDate::from_ymd_opt(1929, 1, 1).unwrap()
        );
        assert_eq!(series.frequency, Frequency::Annual);
        assert_eq!(
            series.seasonal_adjustment,
            SeasonalAdjustment::NotSeasonallyAdjusted
        );
        assert_eq!(series.units, "Billions of Chained 2017 Dollars");
        assert_eq!(series.popularity, 76);
        assert_eq!(series.notes.as_deref(), Some("BEA Account Code: A001RX"));
    }

    #[test]
    fn notes_default_to_none_when_absent() {
        let json = r#"{
            "id": "X",
            "title": "T",
            "observation_start": "2000-01-01",
            "observation_end": "2001-01-01",
            "frequency": "Monthly",
            "seasonal_adjustment": "Seasonally Adjusted",
            "units": "Percent",
            "popularity": 0,
            "last_updated": "2024-01-01 00:00:00-05"
        }"#;
        let series: Series = serde_json::from_str(json).unwrap();
        assert_eq!(series.notes, None);
    }

    #[test]
    fn deserializes_search_results_with_pagination() {
        let json = format!(
            r#"{{
                "order_by": "search_rank",
                "sort_order": "desc",
                "count": 1,
                "offset": 0,
                "limit": 1000,
                "seriess": [{GNPCA_JSON}]
            }}"#
        );
        let results: SeriesSearchResults = serde_json::from_str(&json).unwrap();
        assert_eq!(results.count, 1);
        assert_eq!(results.offset, 0);
        assert_eq!(results.limit, 1000);
        assert_eq!(results.series.len(), 1);
        assert_eq!(results.series[0].id, SeriesId::new("GNPCA"));
    }

    #[test]
    fn serializes_with_typed_enum_labels_and_clean_keys() {
        let series: Series = serde_json::from_str(GNPCA_JSON).unwrap();
        let value = serde_json::to_value(&series).unwrap();
        assert_eq!(value["id"], "GNPCA");
        assert_eq!(value["frequency"], "Annual");
        assert_eq!(value["seasonal_adjustment"], "Not Seasonally Adjusted");

        let results = SeriesSearchResults {
            series: vec![series],
            count: 1,
            offset: 0,
            limit: 1,
        };
        let results_value = serde_json::to_value(&results).unwrap();
        // The output uses the correctly-spelled `series`, not FRED's `seriess`.
        assert!(results_value.get("series").is_some());
        assert!(results_value.get("seriess").is_none());
    }
}
