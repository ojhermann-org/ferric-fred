/// How observations are aggregated when down-sampling to a lower `frequency`
/// (the `aggregation_method` request parameter). Only meaningful alongside
/// [`ObservationsRequest::frequency`](crate::ObservationsRequest::frequency).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AggregationMethod {
    /// Average of the higher-frequency values (`avg`, FRED's default).
    Average,
    /// Sum of the higher-frequency values (`sum`).
    Sum,
    /// The end-of-period value (`eop`).
    EndOfPeriod,
}

impl AggregationMethod {
    /// The FRED query code for this aggregation method.
    pub fn query_code(self) -> &'static str {
        match self {
            Self::Average => "avg",
            Self::Sum => "sum",
            Self::EndOfPeriod => "eop",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_codes_match_fred() {
        assert_eq!(AggregationMethod::Average.query_code(), "avg");
        assert_eq!(AggregationMethod::Sum.query_code(), "sum");
        assert_eq!(AggregationMethod::EndOfPeriod.query_code(), "eop");
    }
}
