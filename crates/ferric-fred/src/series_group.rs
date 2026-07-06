use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::SeriesGroupId;

/// Metadata for a GeoFRED / Maps series group (the `geofred/series/group`
/// endpoint) — the descriptive header for a group of regional series, and the
/// span of dates it covers.
///
/// `region_type`, `season`, `units`, and `frequency` are kept as `String`: FRED
/// returns them as display/code text here, and this crate mirrors the wire
/// rather than parsing them into enums (ADR-0025).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SeriesGroup {
    /// The group's descriptive title, e.g. `"All Employees: Total Private"`.
    pub title: String,

    /// The group's identifier (FRED's own `series_group` field), e.g. `1223`.
    #[serde(rename = "series_group")]
    pub id: SeriesGroupId,

    /// The region granularity as a display label, e.g. `"state"`.
    pub region_type: String,

    /// The seasonality, as FRED reports it here (a short code, e.g. `"NSA"`).
    pub season: String,

    /// The units as a display label, e.g. `"Thousands of Persons"`.
    pub units: String,

    /// The frequency as a display label, e.g. `"Monthly"`.
    pub frequency: String,

    /// The earliest date the group has data for.
    pub min_date: NaiveDate,

    /// The latest date the group has data for.
    pub max_date: NaiveDate,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SERIES_GROUP: &str = r#"{
        "title": "All Employees: Total Private",
        "region_type": "state",
        "series_group": "1223",
        "season": "NSA",
        "units": "Thousands of Persons",
        "frequency": "Monthly",
        "min_date": "1990-01-01",
        "max_date": "2026-05-01"
    }"#;

    #[test]
    fn parses_group_metadata() {
        let group: SeriesGroup = serde_json::from_str(SERIES_GROUP).expect("series group parses");
        assert_eq!(group.id, SeriesGroupId::new("1223"));
        assert_eq!(group.title, "All Employees: Total Private");
        assert_eq!(group.region_type, "state");
        assert_eq!(group.season, "NSA");
        assert_eq!(group.min_date, NaiveDate::from_ymd_opt(1990, 1, 1).unwrap());
        assert_eq!(group.max_date, NaiveDate::from_ymd_opt(2026, 5, 1).unwrap());
    }
}
