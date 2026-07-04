use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Whether, and how, a FRED series is seasonally adjusted.
///
/// Deserialized from FRED's long-form `seasonal_adjustment` label. Unmodelled
/// labels are preserved verbatim in [`SeasonalAdjustment::Other`] rather than
/// failing to deserialize (ADR-0005). Also `#[non_exhaustive]`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SeasonalAdjustment {
    /// Seasonally adjusted.
    SeasonallyAdjusted,
    /// Not seasonally adjusted.
    NotSeasonallyAdjusted,
    /// Seasonally adjusted annual rate.
    SeasonallyAdjustedAnnualRate,
    /// A value FRED reported that this version does not model; holds the raw
    /// label verbatim.
    Other(String),
}

impl SeasonalAdjustment {
    /// Map FRED's long-form seasonal-adjustment label to a variant.
    fn from_label(label: &str) -> Self {
        match label {
            "Seasonally Adjusted" => Self::SeasonallyAdjusted,
            "Not Seasonally Adjusted" => Self::NotSeasonallyAdjusted,
            "Seasonally Adjusted Annual Rate" => Self::SeasonallyAdjustedAnnualRate,
            other => Self::Other(other.to_owned()),
        }
    }

    /// The label, as FRED presents it.
    pub fn label(&self) -> &str {
        match self {
            Self::SeasonallyAdjusted => "Seasonally Adjusted",
            Self::NotSeasonallyAdjusted => "Not Seasonally Adjusted",
            Self::SeasonallyAdjustedAnnualRate => "Seasonally Adjusted Annual Rate",
            Self::Other(label) => label,
        }
    }
}

impl fmt::Display for SeasonalAdjustment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl<'de> Deserialize<'de> for SeasonalAdjustment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let label = String::deserialize(deserializer)?;
        Ok(Self::from_label(&label))
    }
}

impl Serialize for SeasonalAdjustment {
    /// Serializes as FRED's long-form label — symmetric with [`Deserialize`], so
    /// the value round-trips.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_labels_map_to_variants() {
        assert_eq!(
            serde_json::from_str::<SeasonalAdjustment>("\"Not Seasonally Adjusted\"").unwrap(),
            SeasonalAdjustment::NotSeasonallyAdjusted
        );
        assert_eq!(
            serde_json::from_str::<SeasonalAdjustment>("\"Seasonally Adjusted Annual Rate\"")
                .unwrap(),
            SeasonalAdjustment::SeasonallyAdjustedAnnualRate
        );
    }

    #[test]
    fn unknown_label_is_preserved_verbatim() {
        assert_eq!(
            serde_json::from_str::<SeasonalAdjustment>("\"Smoothed\"").unwrap(),
            SeasonalAdjustment::Other("Smoothed".to_owned())
        );
    }

    #[test]
    fn serializes_to_its_label() {
        assert_eq!(
            serde_json::to_string(&SeasonalAdjustment::SeasonallyAdjustedAnnualRate).unwrap(),
            "\"Seasonally Adjusted Annual Rate\""
        );
    }
}
