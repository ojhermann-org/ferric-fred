use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::ReleaseId;

/// A single scheduled or historical release date, from the
/// `fred/releases/dates` and `fred/release/dates` endpoints — the date a
/// release was (or is scheduled to be) published.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseDate {
    /// The release this date belongs to.
    pub release_id: ReleaseId,

    /// The release's name. `releases/dates` (which spans every release)
    /// includes it; `release/dates` omits it, since the release is already
    /// fixed by the request.
    #[serde(default)]
    pub release_name: Option<String>,

    /// The date the release was, or is scheduled to be, published.
    pub date: NaiveDate,
}

/// A page of release dates with pagination metadata, shared by the
/// `fred/releases/dates` and `fred/release/dates` endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseDatesResults {
    /// Total number of release dates available (across all pages).
    pub count: u32,

    /// The offset (number of dates skipped) for this page.
    pub offset: u32,

    /// The page-size limit that was applied.
    pub limit: u32,

    /// The release dates on this page.
    pub release_dates: Vec<ReleaseDate>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_a_named_release_date() {
        // `releases/dates` entries carry the release name.
        let date: ReleaseDate = serde_json::from_str(
            r#"{"release_id":9,"release_name":"Advance Monthly Sales for Retail and Food Services","date":"2013-08-13"}"#,
        )
        .unwrap();
        assert_eq!(date.release_id, ReleaseId::new(9));
        assert_eq!(
            date.release_name.as_deref(),
            Some("Advance Monthly Sales for Retail and Food Services")
        );
        assert_eq!(date.date, NaiveDate::from_ymd_opt(2013, 8, 13).unwrap());
    }

    #[test]
    fn release_date_without_name_defaults_to_none() {
        // `release/dates` entries omit the name (the release is fixed).
        let date: ReleaseDate =
            serde_json::from_str(r#"{"release_id":82,"date":"1997-02-10"}"#).unwrap();
        assert_eq!(date.release_id, ReleaseId::new(82));
        assert!(date.release_name.is_none());
        assert_eq!(date.date, NaiveDate::from_ymd_opt(1997, 2, 10).unwrap());
    }

    #[test]
    fn results_carry_pagination() {
        let results: ReleaseDatesResults = serde_json::from_str(
            r#"{"count":2,"offset":0,"limit":1000,"release_dates":[
                {"release_id":9,"release_name":"Advance Monthly Sales","date":"2013-08-13"},
                {"release_id":10,"release_name":"Consumer Price Index","date":"2013-08-15"}
            ]}"#,
        )
        .unwrap();
        assert_eq!(results.count, 2);
        assert_eq!(results.release_dates.len(), 2);
        assert_eq!(results.release_dates[1].release_id, ReleaseId::new(10));
    }
}
