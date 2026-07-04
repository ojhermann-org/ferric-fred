use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The native reporting frequency of a FRED series.
///
/// Deserialized from FRED's long-form `frequency` label (e.g. `"Monthly"`).
/// Labels this version does not model — for instance the week-ending variants
/// like `"Weekly, Ending Friday"` — are preserved verbatim in
/// [`Frequency::Other`] rather than failing to deserialize (ADR-0005:
/// forward-compatibility over strictness). The enum is also `#[non_exhaustive]`
/// so new named variants can be promoted out of `Other` later without breaking
/// callers' `match` arms.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Frequency {
    /// Daily.
    Daily,
    /// Weekly.
    Weekly,
    /// Biweekly.
    Biweekly,
    /// Monthly.
    Monthly,
    /// Quarterly.
    Quarterly,
    /// Semiannual.
    Semiannual,
    /// Annual.
    Annual,
    /// A frequency FRED reported that this version does not model; holds the raw
    /// label verbatim.
    Other(String),
}

impl Frequency {
    /// Map FRED's long-form frequency label to a [`Frequency`].
    fn from_label(label: &str) -> Self {
        match label {
            "Daily" => Self::Daily,
            "Weekly" => Self::Weekly,
            "Biweekly" => Self::Biweekly,
            "Monthly" => Self::Monthly,
            "Quarterly" => Self::Quarterly,
            "Semiannual" => Self::Semiannual,
            "Annual" => Self::Annual,
            other => Self::Other(other.to_owned()),
        }
    }

    /// The frequency label, as FRED presents it.
    pub fn label(&self) -> &str {
        match self {
            Self::Daily => "Daily",
            Self::Weekly => "Weekly",
            Self::Biweekly => "Biweekly",
            Self::Monthly => "Monthly",
            Self::Quarterly => "Quarterly",
            Self::Semiannual => "Semiannual",
            Self::Annual => "Annual",
            Self::Other(label) => label,
        }
    }

    /// The FRED query code for requesting aggregation to this frequency (the
    /// observations `frequency` parameter): `d`, `w`, `bw`, `m`, `q`, `sa`,
    /// `a`. For [`Frequency::Other`] this returns the raw label, which may not
    /// be a valid FRED code.
    pub fn query_code(&self) -> &str {
        match self {
            Self::Daily => "d",
            Self::Weekly => "w",
            Self::Biweekly => "bw",
            Self::Monthly => "m",
            Self::Quarterly => "q",
            Self::Semiannual => "sa",
            Self::Annual => "a",
            Self::Other(label) => label,
        }
    }
}

impl fmt::Display for Frequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl<'de> Deserialize<'de> for Frequency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let label = String::deserialize(deserializer)?;
        Ok(Self::from_label(&label))
    }
}

impl Serialize for Frequency {
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
    fn known_label_maps_to_variant() {
        assert_eq!(
            serde_json::from_str::<Frequency>("\"Monthly\"").unwrap(),
            Frequency::Monthly
        );
    }

    #[test]
    fn unknown_label_is_preserved_verbatim() {
        assert_eq!(
            serde_json::from_str::<Frequency>("\"Weekly, Ending Friday\"").unwrap(),
            Frequency::Other("Weekly, Ending Friday".to_owned())
        );
    }

    #[test]
    fn display_round_trips_the_label() {
        assert_eq!(Frequency::Annual.to_string(), "Annual");
        assert_eq!(
            Frequency::Other("Weekly, Ending Friday".to_owned()).to_string(),
            "Weekly, Ending Friday"
        );
    }

    #[test]
    fn query_codes_match_fred() {
        assert_eq!(Frequency::Monthly.query_code(), "m");
        assert_eq!(Frequency::Semiannual.query_code(), "sa");
        assert_eq!(Frequency::Daily.query_code(), "d");
    }

    #[test]
    fn serializes_to_its_label_and_round_trips() {
        assert_eq!(
            serde_json::to_string(&Frequency::Monthly).unwrap(),
            "\"Monthly\""
        );
        let other = Frequency::Other("Weekly, Ending Friday".to_owned());
        let json = serde_json::to_string(&other).unwrap();
        assert_eq!(json, "\"Weekly, Ending Friday\"");
        assert_eq!(serde_json::from_str::<Frequency>(&json).unwrap(), other);
    }
}
