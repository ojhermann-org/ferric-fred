use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A geographic region granularity for a GeoFRED / Maps request (the
/// `region_type` parameter of `geofred/regional/data`) — the level the data is
/// broken down to.
///
/// Carried on the wire as a lowercase token (e.g. `"state"`). Tokens this
/// version does not name — e.g. Federal Reserve districts (`"frb"`) or census
/// divisions — round-trip verbatim through [`RegionType::Other`] rather than
/// failing (ADR-0005: forward-compatibility over strictness), and the enum is
/// `#[non_exhaustive]` so new variants can be promoted out of `Other` later
/// without breaking callers' `match` arms.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RegionType {
    /// U.S. state.
    State,
    /// U.S. county.
    County,
    /// Metropolitan Statistical Area.
    Msa,
    /// Country.
    Country,
    /// Bureau of Economic Analysis region.
    Bea,
    /// A region type FRED accepts that this version does not name; holds the raw
    /// token verbatim.
    Other(String),
}

impl RegionType {
    /// Map a GeoFRED region-type token to a [`RegionType`].
    fn from_token(token: &str) -> Self {
        match token {
            "state" => Self::State,
            "county" => Self::County,
            "msa" => Self::Msa,
            "country" => Self::Country,
            "bea" => Self::Bea,
            other => Self::Other(other.to_owned()),
        }
    }

    /// The GeoFRED query token for this region type (the `region_type`
    /// parameter): `state`, `county`, `msa`, `country`, `bea`. For
    /// [`RegionType::Other`] this returns the raw token.
    pub fn query_code(&self) -> &str {
        match self {
            Self::State => "state",
            Self::County => "county",
            Self::Msa => "msa",
            Self::Country => "country",
            Self::Bea => "bea",
            Self::Other(token) => token,
        }
    }
}

impl fmt::Display for RegionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.query_code())
    }
}

impl<'de> Deserialize<'de> for RegionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let token = String::deserialize(deserializer)?;
        Ok(Self::from_token(&token))
    }
}

impl Serialize for RegionType {
    /// Serializes as the GeoFRED token — symmetric with [`Deserialize`].
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.query_code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_token_maps_to_variant() {
        assert_eq!(
            serde_json::from_str::<RegionType>("\"state\"").unwrap(),
            RegionType::State
        );
    }

    #[test]
    fn unknown_token_is_preserved_verbatim() {
        assert_eq!(
            serde_json::from_str::<RegionType>("\"frb\"").unwrap(),
            RegionType::Other("frb".to_owned())
        );
    }

    #[test]
    fn query_codes_match_fred() {
        assert_eq!(RegionType::State.query_code(), "state");
        assert_eq!(RegionType::Msa.query_code(), "msa");
        assert_eq!(RegionType::Other("frb".to_owned()).query_code(), "frb");
    }

    #[test]
    fn serializes_to_its_token_and_round_trips() {
        assert_eq!(
            serde_json::to_string(&RegionType::Country).unwrap(),
            "\"country\""
        );
        let other = RegionType::Other("censusregion".to_owned());
        let json = serde_json::to_string(&other).unwrap();
        assert_eq!(serde_json::from_str::<RegionType>(&json).unwrap(), other);
    }
}
