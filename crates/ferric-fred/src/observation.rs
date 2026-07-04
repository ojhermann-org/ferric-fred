use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serialize};

/// A single observation in a FRED series: a calendar date and its value.
///
/// FRED transmits the value as a string and encodes a *missing* value as the
/// sentinel `"."`, which maps to `None`. Any other value parses to `Some(f64)`;
/// a non-`"."` value that fails to parse is a deserialization error, not a
/// silent `None` (see ADR-0004 and ADR-0005).
///
/// On *serialization* the value is emitted as a JSON number or `null` — typed
/// JSON for consumers, not FRED's stringly-typed `"."` wire format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    /// The observation date. FRED dates are calendar dates with no time or zone,
    /// which [`NaiveDate`] models exactly.
    pub date: NaiveDate,

    /// The observation value; `None` when FRED reports it as missing (`"."`).
    #[serde(deserialize_with = "deserialize_value")]
    pub value: Option<f64>,
}

/// Deserialize a FRED observation value: `"."` → `None`, otherwise parse the
/// string as `f64`.
fn deserialize_value<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    if raw == "." {
        return Ok(None);
    }
    raw.parse::<f64>()
        .map(Some)
        .map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_value_maps_to_none() {
        let obs: Observation =
            serde_json::from_str(r#"{"date":"1930-01-01","value":"."}"#).unwrap();
        assert_eq!(obs.value, None);
        assert_eq!(obs.date, NaiveDate::from_ymd_opt(1930, 1, 1).unwrap());
    }

    #[test]
    fn numeric_value_parses() {
        let obs: Observation =
            serde_json::from_str(r#"{"date":"1929-01-01","value":"1065.9"}"#).unwrap();
        assert_eq!(obs.value, Some(1065.9));
    }

    #[test]
    fn unparseable_value_is_an_error() {
        let parsed: Result<Observation, _> =
            serde_json::from_str(r#"{"date":"1929-01-01","value":"not a number"}"#);
        assert!(parsed.is_err());
    }

    #[test]
    fn serializes_value_as_number_or_null() {
        let present = Observation {
            date: NaiveDate::from_ymd_opt(1929, 1, 1).unwrap(),
            value: Some(1065.9),
        };
        assert_eq!(
            serde_json::to_value(&present).unwrap(),
            serde_json::json!({"date": "1929-01-01", "value": 1065.9})
        );

        let missing = Observation {
            date: NaiveDate::from_ymd_opt(1930, 1, 1).unwrap(),
            value: None,
        };
        assert_eq!(
            serde_json::to_value(&missing).unwrap(),
            serde_json::json!({"date": "1930-01-01", "value": null})
        );
    }
}
