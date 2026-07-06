use serde::{Deserialize, Serialize};

/// A FRED tag — a keyword used to classify series (e.g. `gdp`, `quarterly`,
/// `nsa`), from the `fred/tags`, `fred/series/tags`, and related endpoints.
///
/// Tags are identified by [`name`](Tag::name); there is no numeric id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Tag {
    /// The tag's name, e.g. `"gdp"`.
    pub name: String,

    /// The id of the group the tag belongs to (e.g. `"gen"` general, `"geo"`
    /// geography, `"freq"` frequency, `"seas"` seasonal adjustment).
    pub group_id: String,

    /// Descriptive notes, when FRED provides them (may be absent or null).
    #[serde(default)]
    pub notes: Option<String>,

    /// Relative popularity, 0–100.
    pub popularity: u32,

    /// Number of series carrying this tag.
    pub series_count: u64,
}

/// A page of tags with pagination metadata, from the `fred/tags` and
/// `fred/series/tags` endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct TagsResults {
    /// Total number of tags available (across all pages).
    pub count: u32,

    /// The offset (number of tags skipped) for this page.
    pub offset: u32,

    /// The page-size limit that was applied.
    pub limit: u32,

    /// The tags on this page.
    pub tags: Vec<Tag>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_a_tag() {
        let tag: Tag = serde_json::from_str(
            r#"{"name":"gdp","group_id":"gen","notes":"Gross Domestic Product",
                "created":"2012-02-27 10:18:19-06","popularity":80,"series_count":12345}"#,
        )
        .unwrap();
        assert_eq!(tag.name, "gdp");
        assert_eq!(tag.group_id, "gen");
        assert_eq!(tag.notes.as_deref(), Some("Gross Domestic Product"));
        assert_eq!(tag.popularity, 80);
        assert_eq!(tag.series_count, 12345);
    }

    #[test]
    fn tag_notes_may_be_null_or_absent() {
        let with_null: Tag = serde_json::from_str(
            r#"{"name":"nsa","group_id":"seas","notes":null,"popularity":90,"series_count":9}"#,
        )
        .unwrap();
        assert!(with_null.notes.is_none());

        let absent: Tag = serde_json::from_str(
            r#"{"name":"nsa","group_id":"seas","popularity":90,"series_count":9}"#,
        )
        .unwrap();
        assert!(absent.notes.is_none());
    }

    #[test]
    fn tags_results_carry_pagination() {
        let results: TagsResults = serde_json::from_str(
            r#"{"count":2,"offset":0,"limit":1000,"tags":[
                {"name":"gdp","group_id":"gen","popularity":80,"series_count":12345},
                {"name":"quarterly","group_id":"freq","popularity":75,"series_count":6789}
            ]}"#,
        )
        .unwrap();
        assert_eq!(results.count, 2);
        assert_eq!(results.tags.len(), 2);
        assert_eq!(results.tags[1].name, "quarterly");
    }
}
