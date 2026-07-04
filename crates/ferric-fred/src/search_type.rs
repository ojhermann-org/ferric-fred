/// How `series/search` interprets the search text (the `search_type` request
/// parameter).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SearchType {
    /// Match against series attributes and full text (`full_text`, FRED's
    /// default).
    FullText,
    /// Match against the series ID (`series_id`).
    SeriesId,
}

impl SearchType {
    /// The FRED query code for this search type.
    pub fn query_code(self) -> &'static str {
        match self {
            Self::FullText => "full_text",
            Self::SeriesId => "series_id",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_codes_match_fred() {
        assert_eq!(SearchType::FullText.query_code(), "full_text");
        assert_eq!(SearchType::SeriesId.query_code(), "series_id");
    }
}
