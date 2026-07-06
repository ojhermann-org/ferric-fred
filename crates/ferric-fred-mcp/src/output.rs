//! Output shapes for the tools that don't return a single library type.
//!
//! Most tools serialize a `ferric_fred` domain type straight through, so their
//! output schema comes from that type. A handful, though, wrap a bare list in a
//! small envelope (a `count` plus the items, sometimes echoing the request id).
//! Those envelopes are defined here as real types — deriving `Serialize` +
//! `JsonSchema` — so the value a tool emits and the schema it advertises come
//! from one definition and can't drift (issue #30 / ADR-0023).

use ferric_fred::{Category, Observation, Source};
use schemars::JsonSchema;
use serde::Serialize;

/// `get_observations` — a series' observations, with its id and a count echoed
/// alongside.
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct ObservationsOutput {
    /// The series the observations belong to.
    pub series_id: String,
    /// The number of observations returned.
    pub count: usize,
    /// The observations, in the requested sort order.
    pub observations: Vec<Observation>,
}

/// `get_category_children` — the child categories of a category.
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct CategoryChildrenOutput {
    /// The parent category whose children these are.
    pub category_id: u32,
    /// The number of child categories returned.
    pub count: usize,
    /// The child categories.
    pub children: Vec<Category>,
}

/// `get_category_related` — the categories cross-linked to a category.
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct CategoryRelatedOutput {
    /// The category whose related categories these are.
    pub category_id: u32,
    /// The number of related categories returned.
    pub count: usize,
    /// The related categories (often empty).
    pub related: Vec<Category>,
}

/// `get_release_sources` — the data sources a release draws from.
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct ReleaseSourcesOutput {
    /// The number of sources returned.
    pub count: usize,
    /// The sources.
    pub sources: Vec<Source>,
}

/// `get_series_categories` — the categories a series belongs to.
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct SeriesCategoriesOutput {
    /// The number of categories returned.
    pub count: usize,
    /// The categories the series belongs to.
    pub categories: Vec<Category>,
}
