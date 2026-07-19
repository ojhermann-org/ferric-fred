use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The kind of geographic boundary set to fetch from GeoFRED / Maps (the
/// `shape` parameter of `geofred/shapes/file`) — which regions' polygons the
/// returned [`ShapeFile`](crate::ShapeFile) covers.
///
/// Carried on the wire as a lowercase token (e.g. `"bea"`). Tokens this version
/// does not name round-trip verbatim through [`ShapeType::Other`] rather than
/// failing (ADR-0005), and the enum is `#[non_exhaustive]` so new variants can
/// be promoted out of `Other` later without breaking callers' `match` arms.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ShapeType {
    /// U.S. state boundaries.
    State,
    /// U.S. county boundaries.
    County,
    /// Metropolitan Statistical Area boundaries.
    Msa,
    /// Country boundaries.
    Country,
    /// Bureau of Economic Analysis region boundaries.
    Bea,
    /// A shape type FRED accepts that this version does not name; holds the raw
    /// token verbatim.
    Other(String),
}

impl ShapeType {
    /// Map a GeoFRED shape token to a [`ShapeType`].
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

    /// The GeoFRED query token for this shape (the `shape` parameter): `state`,
    /// `county`, `msa`, `country`, `bea`. For [`ShapeType::Other`] this returns
    /// the raw token.
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

impl fmt::Display for ShapeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.query_code())
    }
}

impl<'de> Deserialize<'de> for ShapeType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let token = String::deserialize(deserializer)?;
        Ok(Self::from_token(&token))
    }
}

impl Serialize for ShapeType {
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
            serde_json::from_str::<ShapeType>("\"bea\"").unwrap(),
            ShapeType::Bea
        );
    }

    #[test]
    fn unknown_token_is_preserved_verbatim() {
        assert_eq!(
            serde_json::from_str::<ShapeType>("\"necta\"").unwrap(),
            ShapeType::Other("necta".to_owned())
        );
    }

    #[test]
    fn query_codes_match_fred() {
        assert_eq!(ShapeType::Bea.query_code(), "bea");
        assert_eq!(ShapeType::State.query_code(), "state");
        assert_eq!(ShapeType::Other("necta".to_owned()).query_code(), "necta");
    }

    /// Every named (non-`Other`) [`ShapeType`], hand-maintained. The crate
    /// declines a `strum`/`EnumIter` dependency (ADR-0030), so this list is the
    /// drift anchor; [`known_list_is_exhaustive`] keeps it complete.
    const KNOWN: &[ShapeType] = &[
        ShapeType::State,
        ShapeType::County,
        ShapeType::Msa,
        ShapeType::Country,
        ShapeType::Bea,
    ];

    #[test]
    fn every_known_variant_round_trips_through_serde() {
        for variant in KNOWN {
            let json = serde_json::to_string(variant).unwrap();
            let back: ShapeType = serde_json::from_str(&json).unwrap();
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
        fn account_for_every_variant(shape_type: &ShapeType) {
            match shape_type {
                ShapeType::State
                | ShapeType::County
                | ShapeType::Msa
                | ShapeType::Country
                | ShapeType::Bea
                | ShapeType::Other(_) => {}
            }
        }
        account_for_every_variant(&ShapeType::State);
        assert!(
            KNOWN.iter().all(|v| !matches!(v, ShapeType::Other(_))),
            "`KNOWN` must list only named variants, never `Other`"
        );
    }
}
