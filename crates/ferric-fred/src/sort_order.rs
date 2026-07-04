/// Sort order for returned observations, by observation date (the `sort_order`
/// request parameter).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SortOrder {
    /// Oldest first (`asc`, FRED's default).
    Ascending,
    /// Newest first (`desc`).
    Descending,
}

impl SortOrder {
    /// The FRED query code for this sort order.
    pub fn query_code(self) -> &'static str {
        match self {
            Self::Ascending => "asc",
            Self::Descending => "desc",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_codes_match_fred() {
        assert_eq!(SortOrder::Ascending.query_code(), "asc");
        assert_eq!(SortOrder::Descending.query_code(), "desc");
    }
}
