use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// The vintage dates of a series with pagination metadata, from the
/// `fred/series/vintagedates` endpoint.
///
/// A vintage date is a date on which the series' data were revised or new
/// observations were released — the series "as it looked" on that date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VintageDates {
    /// Total number of vintage dates available (across all pages).
    pub count: u32,

    /// The offset (number of dates skipped) for this page.
    pub offset: u32,

    /// The page-size limit that was applied.
    pub limit: u32,

    /// The vintage dates on this page, oldest first by default.
    pub vintage_dates: Vec<NaiveDate>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_vintage_dates() {
        let dates: VintageDates = serde_json::from_str(
            r#"{"count":3,"offset":0,"limit":10000,
                "vintage_dates":["1958-12-21","1959-02-19","2013-07-31"]}"#,
        )
        .unwrap();
        assert_eq!(dates.count, 3);
        assert_eq!(dates.vintage_dates.len(), 3);
        assert_eq!(
            dates.vintage_dates[0],
            NaiveDate::from_ymd_opt(1958, 12, 21).unwrap()
        );
        assert_eq!(
            dates.vintage_dates[2],
            NaiveDate::from_ymd_opt(2013, 7, 31).unwrap()
        );
    }
}
