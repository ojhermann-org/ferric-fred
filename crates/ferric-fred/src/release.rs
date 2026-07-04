use serde::{Deserialize, Serialize};

use crate::ReleaseId;

/// A FRED data release — a publication such as "Gross Domestic Product", from
/// the `fred/release` and `fred/releases` endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Release {
    /// The release's identifier.
    pub id: ReleaseId,

    /// Human-readable name, e.g. `"Gross Domestic Product"`.
    pub name: String,

    /// Whether the release is accompanied by a press release.
    pub press_release: bool,

    /// A link to the release on the source's site, when FRED provides one.
    #[serde(default)]
    pub link: Option<String>,
}

/// A page of releases with pagination metadata, from the `fred/releases`
/// endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleasesResults {
    /// Total number of releases available (across all pages).
    pub count: u32,

    /// The offset (number of releases skipped) for this page.
    pub offset: u32,

    /// The page-size limit that was applied.
    pub limit: u32,

    /// The releases on this page.
    pub releases: Vec<Release>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_a_release_with_a_link() {
        let release: Release = serde_json::from_str(
            r#"{"id":53,"realtime_start":"2013-08-13","realtime_end":"2013-08-13",
                "name":"Gross Domestic Product","press_release":true,
                "link":"http://www.bea.gov/national/index.htm"}"#,
        )
        .unwrap();
        assert_eq!(release.id, ReleaseId::new(53));
        assert_eq!(release.name, "Gross Domestic Product");
        assert!(release.press_release);
        assert_eq!(
            release.link.as_deref(),
            Some("http://www.bea.gov/national/index.htm")
        );
    }

    #[test]
    fn release_without_link_defaults_to_none() {
        let release: Release = serde_json::from_str(
            r#"{"id":9,"name":"Advance Monthly Sales","press_release":false}"#,
        )
        .unwrap();
        assert!(release.link.is_none());
    }

    #[test]
    fn releases_results_carry_pagination() {
        let results: ReleasesResults = serde_json::from_str(
            r#"{"count":2,"offset":0,"limit":1000,"releases":[
                {"id":9,"name":"Advance Monthly Sales","press_release":false},
                {"id":10,"name":"Consumer Price Index","press_release":true}
            ]}"#,
        )
        .unwrap();
        assert_eq!(results.count, 2);
        assert_eq!(results.releases.len(), 2);
        assert_eq!(results.releases[1].id, ReleaseId::new(10));
    }
}
