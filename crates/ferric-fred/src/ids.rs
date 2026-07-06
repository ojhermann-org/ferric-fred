/// A FRED series identifier, e.g. `GNPCA` or `UNRATE`.
///
/// A newtype over `String` so a series id can't be silently swapped for another
/// kind of identifier or an arbitrary string (see ADR-0005). Construction does
/// no validation for now — FRED rejects malformed ids — but the newtype gives
/// us a place to add it later without changing call sites.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
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

/// A GeoFRED / Maps series-group identifier, e.g. `1223` — the group of
/// regional series a `geofred/regional/data` request pulls a cross-section from.
///
/// A newtype over `String`: FRED transmits it as a string (`"series_group":
/// "1223"`) even though it reads as a number, so — unlike the numeric
/// [`CategoryId`]/[`ReleaseId`] `u32` newtypes — it stays string-backed to match
/// the wire (ADR-0005). Construction does no validation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SeriesGroupId(String);

impl SeriesGroupId {
    /// Wrap a string as a [`SeriesGroupId`].
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Borrow the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SeriesGroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for SeriesGroupId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl From<String> for SeriesGroupId {
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
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
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

/// A FRED release-table element identifier — the numeric id of a node in a
/// release's table tree (a section, table, or series row; see
/// `fred/release/tables`).
///
/// A `Copy` newtype over `u32`, mirroring [`ReleaseId`]; `#[serde(transparent)]`
/// carries it as the bare integer FRED sends (ADR-0005). `Ord` lets the table
/// deserializer order its roots deterministically by id.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(transparent)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ReleaseElementId(u32);

impl ReleaseElementId {
    /// Wrap a numeric id as a [`ReleaseElementId`].
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// The underlying numeric id.
    pub fn get(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for ReleaseElementId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for ReleaseElementId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

/// A FRED source identifier — the numeric id of a data source (the organization
/// that produces a release, e.g. the Bureau of Economic Analysis).
///
/// A `Copy` newtype over `u32`, mirroring [`ReleaseId`]; `#[serde(transparent)]`
/// carries it as the bare integer FRED sends (ADR-0005).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SourceId(u32);

impl SourceId {
    /// Wrap a numeric id as a [`SourceId`].
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// The underlying numeric id.
    pub fn get(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for SourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for SourceId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}
