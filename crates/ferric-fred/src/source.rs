use serde::{Deserialize, Serialize};

use crate::SourceId;

/// A FRED data source — the organization that produces releases (e.g. the
/// Bureau of Economic Analysis), from the `fred/source` and `fred/sources`
/// endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    /// The source's identifier.
    pub id: SourceId,

    /// Human-readable name, e.g. `"U.S. Bureau of Economic Analysis"`.
    pub name: String,

    /// A link to the source's site, when FRED provides one.
    #[serde(default)]
    pub link: Option<String>,
}

/// A page of sources with pagination metadata, from the `fred/sources`
/// endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcesResults {
    /// Total number of sources available (across all pages).
    pub count: u32,

    /// The offset (number of sources skipped) for this page.
    pub offset: u32,

    /// The page-size limit that was applied.
    pub limit: u32,

    /// The sources on this page.
    pub sources: Vec<Source>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_a_source_with_a_link() {
        let source: Source = serde_json::from_str(
            r#"{"id":18,"realtime_start":"2013-08-14","realtime_end":"2013-08-14",
                "name":"U.S. Bureau of Economic Analysis","link":"http://www.bea.gov/"}"#,
        )
        .unwrap();
        assert_eq!(source.id, SourceId::new(18));
        assert_eq!(source.name, "U.S. Bureau of Economic Analysis");
        assert_eq!(source.link.as_deref(), Some("http://www.bea.gov/"));
    }

    #[test]
    fn source_without_link_defaults_to_none() {
        let source: Source =
            serde_json::from_str(r#"{"id":3,"name":"Federal Reserve Board"}"#).unwrap();
        assert!(source.link.is_none());
    }

    #[test]
    fn sources_results_carry_pagination() {
        let results: SourcesResults = serde_json::from_str(
            r#"{"count":2,"offset":0,"limit":1000,"sources":[
                {"id":1,"name":"Board of Governors of the Federal Reserve System (US)"},
                {"id":3,"name":"Federal Reserve Bank of Philadelphia"}
            ]}"#,
        )
        .unwrap();
        assert_eq!(results.count, 2);
        assert_eq!(results.sources.len(), 2);
        assert_eq!(results.sources[1].id, SourceId::new(3));
    }
}
