/// Field to order `series/search` results by (the `order_by` request
/// parameter). Request-only, so it carries [`query_code`](OrderBy::query_code)
/// and is `#[non_exhaustive]` but has no `Other` variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum OrderBy {
    /// Full-text search relevance (`search_rank`, FRED's default for a
    /// full-text search).
    SearchRank,
    /// Series identifier (`series_id`).
    SeriesId,
    /// Series title (`title`).
    Title,
    /// Units description (`units`).
    Units,
    /// Native frequency (`frequency`).
    Frequency,
    /// Seasonal adjustment (`seasonal_adjustment`).
    SeasonalAdjustment,
    /// Popularity score (`popularity`).
    Popularity,
    /// Group popularity score (`group_popularity`).
    GroupPopularity,
    /// When the series was last updated (`last_updated`).
    LastUpdated,
    /// First observation date (`observation_start`).
    ObservationStart,
    /// Last observation date (`observation_end`).
    ObservationEnd,
}

impl OrderBy {
    /// The FRED query code for this ordering.
    pub fn query_code(self) -> &'static str {
        match self {
            Self::SearchRank => "search_rank",
            Self::SeriesId => "series_id",
            Self::Title => "title",
            Self::Units => "units",
            Self::Frequency => "frequency",
            Self::SeasonalAdjustment => "seasonal_adjustment",
            Self::Popularity => "popularity",
            Self::GroupPopularity => "group_popularity",
            Self::LastUpdated => "last_updated",
            Self::ObservationStart => "observation_start",
            Self::ObservationEnd => "observation_end",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_codes_match_fred() {
        assert_eq!(OrderBy::SearchRank.query_code(), "search_rank");
        assert_eq!(OrderBy::Popularity.query_code(), "popularity");
        assert_eq!(OrderBy::LastUpdated.query_code(), "last_updated");
    }
}
