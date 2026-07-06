//! MCP-facing tool-input enums: `Deserialize` + `JsonSchema` mirrors of the
//! library's request enums, each converting into its `ferric_fred` counterpart.
//!
//! These are MCP-facing *input* enums, so they live in the MCP crate regardless
//! (the library models FRED's returned values, not its request codes). The
//! `rename_all` attributes make the generated tool schemas advertise FRED's own
//! value codes. The library return types derive `JsonSchema` too, but only
//! behind its optional, default-off `schemars` feature, for tool *output*
//! schemas (ADR-0023) — plain library consumers still pay nothing.

use ferric_fred::{AggregationMethod, Frequency, OrderBy, SortOrder, Units, UpdatesFilter};
use schemars::JsonSchema;
use serde::Deserialize;

/// Units transformation to apply to observations (FRED's short codes).
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub(crate) enum UnitsArg {
    Lin,
    Chg,
    Ch1,
    Pch,
    Pc1,
    Pca,
    Cch,
    Cca,
    Log,
}

impl From<UnitsArg> for Units {
    fn from(value: UnitsArg) -> Self {
        match value {
            UnitsArg::Lin => Self::Levels,
            UnitsArg::Chg => Self::Change,
            UnitsArg::Ch1 => Self::ChangeFromYearAgo,
            UnitsArg::Pch => Self::PercentChange,
            UnitsArg::Pc1 => Self::PercentChangeFromYearAgo,
            UnitsArg::Pca => Self::CompoundedAnnualRateOfChange,
            UnitsArg::Cch => Self::ContinuouslyCompoundedRateOfChange,
            UnitsArg::Cca => Self::ContinuouslyCompoundedAnnualRateOfChange,
            UnitsArg::Log => Self::NaturalLog,
        }
    }
}

/// Sort direction.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SortOrderArg {
    Asc,
    Desc,
}

impl From<SortOrderArg> for SortOrder {
    fn from(value: SortOrderArg) -> Self {
        match value {
            SortOrderArg::Asc => Self::Ascending,
            SortOrderArg::Desc => Self::Descending,
        }
    }
}

/// Field to order search results by.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OrderByArg {
    SearchRank,
    SeriesId,
    Title,
    Units,
    Frequency,
    SeasonalAdjustment,
    Popularity,
    GroupPopularity,
    LastUpdated,
    ObservationStart,
    ObservationEnd,
}

impl From<OrderByArg> for OrderBy {
    fn from(value: OrderByArg) -> Self {
        match value {
            OrderByArg::SearchRank => Self::SearchRank,
            OrderByArg::SeriesId => Self::SeriesId,
            OrderByArg::Title => Self::Title,
            OrderByArg::Units => Self::Units,
            OrderByArg::Frequency => Self::Frequency,
            OrderByArg::SeasonalAdjustment => Self::SeasonalAdjustment,
            OrderByArg::Popularity => Self::Popularity,
            OrderByArg::GroupPopularity => Self::GroupPopularity,
            OrderByArg::LastUpdated => Self::LastUpdated,
            OrderByArg::ObservationStart => Self::ObservationStart,
            OrderByArg::ObservationEnd => Self::ObservationEnd,
        }
    }
}

/// Frequency to aggregate observations down to.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub(crate) enum FrequencyArg {
    Daily,
    Weekly,
    Biweekly,
    Monthly,
    Quarterly,
    Semiannual,
    Annual,
}

impl From<FrequencyArg> for Frequency {
    fn from(value: FrequencyArg) -> Self {
        match value {
            FrequencyArg::Daily => Self::Daily,
            FrequencyArg::Weekly => Self::Weekly,
            FrequencyArg::Biweekly => Self::Biweekly,
            FrequencyArg::Monthly => Self::Monthly,
            FrequencyArg::Quarterly => Self::Quarterly,
            FrequencyArg::Semiannual => Self::Semiannual,
            FrequencyArg::Annual => Self::Annual,
        }
    }
}

/// Aggregation method used together with a frequency.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AggregationArg {
    Avg,
    Sum,
    Eop,
}

impl From<AggregationArg> for AggregationMethod {
    fn from(value: AggregationArg) -> Self {
        match value {
            AggregationArg::Avg => Self::Average,
            AggregationArg::Sum => Self::Sum,
            AggregationArg::Eop => Self::EndOfPeriod,
        }
    }
}

/// Which series `series/updates` returns.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub(crate) enum UpdatesFilterArg {
    All,
    Macro,
    Regional,
}

impl From<UpdatesFilterArg> for UpdatesFilter {
    fn from(value: UpdatesFilterArg) -> Self {
        match value {
            UpdatesFilterArg::All => Self::All,
            UpdatesFilterArg::Macro => Self::Macro,
            UpdatesFilterArg::Regional => Self::Regional,
        }
    }
}
