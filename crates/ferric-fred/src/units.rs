/// A units transformation FRED applies to a series' observations before
/// returning them (the `units` request parameter).
///
/// A request-only, closed vocabulary — [`query_code`](Units::query_code) is the
/// value sent to FRED. `#[non_exhaustive]` leaves room for FRED to add
/// transformations without a breaking change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Units {
    /// Levels — the data as reported (`lin`, FRED's default).
    Levels,
    /// Change from the previous period (`chg`).
    Change,
    /// Change from a year ago (`ch1`).
    ChangeFromYearAgo,
    /// Percent change from the previous period (`pch`).
    PercentChange,
    /// Percent change from a year ago (`pc1`).
    PercentChangeFromYearAgo,
    /// Compounded annual rate of change (`pca`).
    CompoundedAnnualRateOfChange,
    /// Continuously compounded rate of change (`cch`).
    ContinuouslyCompoundedRateOfChange,
    /// Continuously compounded annual rate of change (`cca`).
    ContinuouslyCompoundedAnnualRateOfChange,
    /// Natural log (`log`).
    NaturalLog,
}

impl Units {
    /// The FRED query code for this transformation.
    pub fn query_code(self) -> &'static str {
        match self {
            Self::Levels => "lin",
            Self::Change => "chg",
            Self::ChangeFromYearAgo => "ch1",
            Self::PercentChange => "pch",
            Self::PercentChangeFromYearAgo => "pc1",
            Self::CompoundedAnnualRateOfChange => "pca",
            Self::ContinuouslyCompoundedRateOfChange => "cch",
            Self::ContinuouslyCompoundedAnnualRateOfChange => "cca",
            Self::NaturalLog => "log",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_codes_match_fred() {
        assert_eq!(Units::Levels.query_code(), "lin");
        assert_eq!(Units::PercentChange.query_code(), "pch");
        assert_eq!(Units::NaturalLog.query_code(), "log");
    }
}
