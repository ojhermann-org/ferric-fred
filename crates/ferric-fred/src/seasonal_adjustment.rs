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

    /// The short code used by GeoFRED / Maps requests (the `season` parameter):
    /// `SA`, `NSA`, `SAAR`. The core FRED endpoints only ever *return* seasonal
    /// adjustment (as a [`label`](Self::label)); GeoFRED is the one surface that
    /// takes it as a request parameter, and it wants these codes. For
    /// [`SeasonalAdjustment::Other`] this returns the raw label, which may not be
    /// a valid code.
    pub fn query_code(&self) -> &str {
        match self {
            Self::SeasonallyAdjusted => "SA",
            Self::NotSeasonallyAdjusted => "NSA",
            Self::SeasonallyAdjustedAnnualRate => "SAAR",
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

// A `SeasonalAdjustment` is carried on the wire as its long-form label (see
// `Serialize`), so its JSON Schema is that of a string. The custom serde impls
// rule out deriving `JsonSchema`, so mirror them by hand.
#[cfg(feature = "schemars")]
impl schemars::JsonSchema for SeasonalAdjustment {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "SeasonalAdjustment".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        <String as schemars::JsonSchema>::json_schema(generator)
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

    #[test]
    fn query_codes_match_geofred() {
        assert_eq!(SeasonalAdjustment::SeasonallyAdjusted.query_code(), "SA");
        assert_eq!(
            SeasonalAdjustment::NotSeasonallyAdjusted.query_code(),
            "NSA"
        );
        assert_eq!(
            SeasonalAdjustment::SeasonallyAdjustedAnnualRate.query_code(),
            "SAAR"
        );
    }

    /// Every named (non-`Other`) [`SeasonalAdjustment`], hand-maintained. The
    /// crate declines a `strum`/`EnumIter` dependency (ADR-0030), so this list is
    /// the drift anchor; [`known_list_is_exhaustive`] keeps it complete.
    const KNOWN: &[SeasonalAdjustment] = &[
        SeasonalAdjustment::SeasonallyAdjusted,
        SeasonalAdjustment::NotSeasonallyAdjusted,
        SeasonalAdjustment::SeasonallyAdjustedAnnualRate,
    ];

    #[test]
    fn every_known_variant_round_trips_through_serde() {
        for variant in KNOWN {
            let json = serde_json::to_string(variant).unwrap();
            let back: SeasonalAdjustment = serde_json::from_str(&json).unwrap();
            assert_eq!(
                &back, variant,
                "{variant:?} serialized to {json} but deserialized back to {back:?} — a \
                 named variant whose label is not wired into `from_label` is silently \
                 swallowed by `Other`, the exact bug ADR-0030 pins"
            );
        }
    }

    #[test]
    fn known_list_is_exhaustive() {
        // Compile-time drift tripwire: adding a variant makes this match
        // non-exhaustive and the crate stops compiling. The fix is to add the new
        // variant to `KNOWN` above, which pulls it into the round-trip test.
        fn account_for_every_variant(seasonal_adjustment: &SeasonalAdjustment) {
            match seasonal_adjustment {
                SeasonalAdjustment::SeasonallyAdjusted
                | SeasonalAdjustment::NotSeasonallyAdjusted
                | SeasonalAdjustment::SeasonallyAdjustedAnnualRate
                | SeasonalAdjustment::Other(_) => {}
            }
        }
        account_for_every_variant(&SeasonalAdjustment::SeasonallyAdjusted);
        assert!(
            KNOWN
                .iter()
                .all(|v| !matches!(v, SeasonalAdjustment::Other(_))),
            "`KNOWN` must list only named variants, never `Other`"
        );
    }
}
