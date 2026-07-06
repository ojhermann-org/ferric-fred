//! CLI-facing argument enums that mirror the library's request enums but derive
//! [`clap::ValueEnum`], keeping `clap` out of the library crate. Each converts
//! into its `ferric_fred` counterpart via `From`. `clap` renders multi-word
//! variants as kebab-case values (e.g. `SearchRank` → `search-rank`).

use clap::ValueEnum;
use ferric_fred::{
    AggregationMethod, Frequency, OrderBy, RegionType, SeasonalAdjustment, ShapeType, SortOrder,
    Units, UpdatesFilter,
};

/// `--units` transformation (values are FRED's short codes).
#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum UnitsArg {
    /// Levels — data as reported.
    Lin,
    /// Change from the previous period.
    Chg,
    /// Change from a year ago.
    Ch1,
    /// Percent change from the previous period.
    Pch,
    /// Percent change from a year ago.
    Pc1,
    /// Compounded annual rate of change.
    Pca,
    /// Continuously compounded rate of change.
    Cch,
    /// Continuously compounded annual rate of change.
    Cca,
    /// Natural log.
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

/// `--sort` order.
#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum SortOrderArg {
    /// Oldest first.
    Asc,
    /// Newest first.
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

/// `--order-by` field for search results.
#[derive(Clone, Copy, ValueEnum)]
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

/// `--frequency` to aggregate observations down to.
#[derive(Clone, Copy, ValueEnum)]
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

/// `--aggregation` method used together with `--frequency`.
#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum AggregationArg {
    /// Average of the higher-frequency values.
    Avg,
    /// Sum of the higher-frequency values.
    Sum,
    /// End-of-period value.
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

/// `--region-type` granularity for GeoFRED `regional` (the region level to break
/// the data down to). The library also supports region types beyond these named
/// ones; the CLI exposes the common set.
#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum RegionTypeArg {
    State,
    County,
    Msa,
    Country,
    Bea,
}

impl From<RegionTypeArg> for RegionType {
    fn from(value: RegionTypeArg) -> Self {
        match value {
            RegionTypeArg::State => Self::State,
            RegionTypeArg::County => Self::County,
            RegionTypeArg::Msa => Self::Msa,
            RegionTypeArg::Country => Self::Country,
            RegionTypeArg::Bea => Self::Bea,
        }
    }
}

/// `--shape` boundary set for GeoFRED `shapes`. As with `--region-type`, the
/// library supports more; the CLI exposes the common set.
#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum ShapeTypeArg {
    State,
    County,
    Msa,
    Country,
    Bea,
}

impl From<ShapeTypeArg> for ShapeType {
    fn from(value: ShapeTypeArg) -> Self {
        match value {
            ShapeTypeArg::State => Self::State,
            ShapeTypeArg::County => Self::County,
            ShapeTypeArg::Msa => Self::Msa,
            ShapeTypeArg::Country => Self::Country,
            ShapeTypeArg::Bea => Self::Bea,
        }
    }
}

/// `--season` seasonal adjustment for GeoFRED `regional` (values are FRED's
/// short codes).
#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum SeasonArg {
    /// Not seasonally adjusted.
    Nsa,
    /// Seasonally adjusted.
    Sa,
    /// Seasonally adjusted annual rate.
    Saar,
}

impl From<SeasonArg> for SeasonalAdjustment {
    fn from(value: SeasonArg) -> Self {
        match value {
            SeasonArg::Nsa => Self::NotSeasonallyAdjusted,
            SeasonArg::Sa => Self::SeasonallyAdjusted,
            SeasonArg::Saar => Self::SeasonallyAdjustedAnnualRate,
        }
    }
}

/// `--filter` class for the `updates` command.
#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum UpdatesFilterArg {
    /// All updated series.
    All,
    /// Macroeconomic series only.
    Macro,
    /// Regional series only.
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
