/// Which series `series/updates` returns, by FRED's `filter_value` parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum UpdatesFilter {
    /// All updated series (`all`, FRED's default).
    All,
    /// Macroeconomic series only (`macro`).
    Macro,
    /// Regional series only (`regional`).
    Regional,
}

impl UpdatesFilter {
    /// The FRED query code for this filter.
    pub fn query_code(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Macro => "macro",
            Self::Regional => "regional",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_codes_match_fred() {
        assert_eq!(UpdatesFilter::All.query_code(), "all");
        assert_eq!(UpdatesFilter::Macro.query_code(), "macro");
        assert_eq!(UpdatesFilter::Regional.query_code(), "regional");
    }
}
