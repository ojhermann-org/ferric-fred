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

    /// Every named (non-`Other`) [`RegionType`], hand-maintained. The crate
    /// declines a `strum`/`EnumIter` dependency (ADR-0030), so this list is the
    /// drift anchor; [`known_list_is_exhaustive`] keeps it complete.
    const KNOWN: &[RegionType] = &[
        RegionType::State,
        RegionType::County,
        RegionType::Msa,
        RegionType::Country,
        RegionType::Bea,
    ];

    #[test]
    fn every_known_variant_round_trips_through_serde() {
        for variant in KNOWN {
            let json = serde_json::to_string(variant).unwrap();
            let back: RegionType = serde_json::from_str(&json).unwrap();
            assert_eq!(
                &back, variant,
                "{variant:?} serialized to {json} but deserialized back to {back:?} — a \
                 named variant whose token is not wired into `from_token` is silently \
                 swallowed by `Other`, the exact bug ADR-0030 pins"
            );
        }
    }

    #[test]
    fn known_list_is_exhaustive() {
        // Compile-time drift tripwire: adding a variant makes this match
        // non-exhaustive and the crate stops compiling. The fix is to add the new
        // variant to `KNOWN` above, which pulls it into the round-trip test.
        fn account_for_every_variant(region_type: &RegionType) {
            match region_type {
                RegionType::State
                | RegionType::County
                | RegionType::Msa
                | RegionType::Country
                | RegionType::Bea
                | RegionType::Other(_) => {}
            }
        }
        account_for_every_variant(&RegionType::State);
        assert!(
            KNOWN.iter().all(|v| !matches!(v, RegionType::Other(_))),
            "`KNOWN` must list only named variants, never `Other`"
        );
    }
}
