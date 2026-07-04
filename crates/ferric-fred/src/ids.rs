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

/// A FRED category identifier — a numeric node in the category tree (the root is
/// [`CategoryId::ROOT`], id `0`).
///
/// A `Copy` newtype over `u32` so a category id can't be silently swapped for a
/// parent id, a count, or an arbitrary number (ADR-0005). `#[serde(transparent)]`
/// carries it on the wire as the bare integer FRED sends.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct CategoryId(u32);

impl CategoryId {
    /// The root of the FRED category tree (id `0`).
    pub const ROOT: Self = Self(0);

    /// Wrap a numeric id as a [`CategoryId`].
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// The underlying numeric id.
    pub fn get(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for CategoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for CategoryId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

/// A FRED release identifier — the numeric id of a data release (a publication
/// such as "Gross Domestic Product").
///
/// A `Copy` newtype over `u32`, mirroring [`CategoryId`]; `#[serde(transparent)]`
/// carries it as the bare integer FRED sends (ADR-0005).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct ReleaseId(u32);

impl ReleaseId {
    /// Wrap a numeric id as a [`ReleaseId`].
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// The underlying numeric id.
    pub fn get(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for ReleaseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for ReleaseId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}
