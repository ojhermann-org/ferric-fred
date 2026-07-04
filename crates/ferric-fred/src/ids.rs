/// A FRED series identifier, e.g. `GNPCA` or `UNRATE`.
///
/// A newtype over `String` so a series id can't be silently swapped for another
/// kind of identifier or an arbitrary string (see ADR-0005). Construction does
/// no validation for now — FRED rejects malformed ids — but the newtype gives
/// us a place to add it later without changing call sites.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SeriesId(String);

impl SeriesId {
    /// Wrap a string as a [`SeriesId`].
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Borrow the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SeriesId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for SeriesId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl From<String> for SeriesId {
    fn from(s: String) -> Self {
        Self(s)
    }
}
