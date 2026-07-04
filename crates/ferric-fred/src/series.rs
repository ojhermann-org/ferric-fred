use chrono::NaiveDate;
use serde::Deserialize;

use crate::{Frequency, SeasonalAdjustment, SeriesId};

/// Metadata describing a FRED series (the `fred/series` endpoint).
///
/// ALFRED vintage fields (`realtime_start` / `realtime_end`) are deferred for v1
/// and ignored on the wire (ADR-0005). `last_updated` is kept as FRED's raw
/// string for now — FRED encodes it with a non-standard timezone offset (e.g.
/// `2024-03-28 07:56:03-05`); a typed datetime is a later refinement.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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
}
