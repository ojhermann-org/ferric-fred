use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::SeriesId;

/// Regional economic data from GeoFRED / Maps — the shared response of both
/// `geofred/series/data` (one series' values across regions, over time) and
/// `geofred/regional/data` (a series group's cross-section at a date). The two
/// endpoints return the **same** shape (ADR-0025), so one type serves both.
///
/// FRED nests everything under a single `meta` object; this type mirrors that.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct RegionalData {
    /// The `meta` payload: the descriptive header plus the dated regional values.
    pub meta: RegionalDataMeta,
}

/// The `meta` payload of a [`RegionalData`] response: a descriptive header and
/// the values, keyed by date.
///
/// `title`, `region`, `seasonality`, `units`, and `frequency` are FRED **display
/// labels** ("state", "Not Seasonally Adjusted", "Dollars", "Annual"), so they
/// are kept as `String` rather than parsed into enums — the request side of the
/// API uses codes, but these come back as free text (ADR-0025).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct RegionalDataMeta {
    /// FRED's descriptive title for the result, e.g.
    /// `"2025 Per Capita Personal Income by State (Dollars)"`.
    pub title: String,

    /// The region granularity as a display label, e.g. `"state"`.
    pub region: String,

    /// The seasonality as a display label, e.g. `"Not Seasonally Adjusted"`.
    pub seasonality: String,

    /// The units as a display label, e.g. `"Dollars"` — echoed from the request's
    /// free-form `units` value (ADR-0025).
    pub units: String,

    /// The frequency as a display label, e.g. `"Annual"`.
    pub frequency: String,

    /// The regional values, keyed by observation date (`BTreeMap` for a
    /// deterministic date order). Each date maps to one [`RegionalDataPoint`]
    /// per region.
    pub data: BTreeMap<String, Vec<RegionalDataPoint>>,
}

/// A single region's value within a [`RegionalDataMeta`] date bucket.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct RegionalDataPoint {
    /// The region's display name, e.g. `"Alabama"`.
    pub region: String,

    /// FRED's region code, e.g. `"01"` for Alabama. A string because FRED zero-
    /// pads it and uses non-numeric codes for some region types.
    pub code: String,

    /// The value for this region on this date. Unlike core FRED observations
    /// (stringly-typed, `"."` for missing), GeoFRED sends a JSON number, so a
    /// plain `Option<f64>` suffices — `None` for a `null` or absent value.
    #[serde(default)]
    pub value: Option<f64>,

    /// The underlying FRED series this region's value comes from, e.g. `ALPCPI`.
    pub series_id: SeriesId,
}

#[cfg(test)]
mod tests {
    use super::*;

    const REGIONAL_DATA: &str = r#"{
        "meta": {
            "title": "2025 Per Capita Personal Income by State (Dollars)",
            "region": "state",
            "seasonality": "Not Seasonally Adjusted",
            "units": "Dollars",
            "frequency": "Annual",
            "data": {
                "2013-01-01": [
                    {"region": "Alabama", "code": "01", "value": 35706, "series_id": "ALPCPI"},
                    {"region": "Alaska", "code": "02", "value": 54012.5, "series_id": "AKPCPI"}
                ]
            }
        }
    }"#;

    #[test]
    fn parses_meta_and_dated_points() {
        let data: RegionalData = serde_json::from_str(REGIONAL_DATA).expect("regional data parses");
        assert_eq!(data.meta.region, "state");
        assert_eq!(data.meta.units, "Dollars");
        let day = &data.meta.data["2013-01-01"];
        assert_eq!(day.len(), 2);
        assert_eq!(day[0].region, "Alabama");
        assert_eq!(day[0].code, "01");
        assert_eq!(day[0].value, Some(35706.0));
        assert_eq!(day[0].series_id, SeriesId::new("ALPCPI"));
        assert_eq!(day[1].value, Some(54012.5));
    }

    #[test]
    fn missing_value_maps_to_none() {
        let point: RegionalDataPoint = serde_json::from_str(
            r#"{"region": "Nowhere", "code": "99", "value": null, "series_id": "NONE"}"#,
        )
        .expect("null value parses");
        assert_eq!(point.value, None);
    }
}
