use std::time::Duration;

use chrono::NaiveDate;
use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::{
    Category, CategoryId, Error, Frequency, Observation, ObservationsRequest, RegionType,
    RegionalData, Release, ReleaseDatesRequest, ReleaseDatesResults, ReleaseId, ReleaseTable,
    ReleaseTablesRequest, ReleasesRequest, ReleasesResults, Result, SeasonalAdjustment, Series,
    SeriesDataRequest, SeriesGroup, SeriesGroupId, SeriesId, SeriesListRequest,
    SeriesSearchRequest, SeriesSearchResults, SeriesUpdatesRequest, ShapeFile, ShapeType, Source,
    SourceId, SourcesRequest, SourcesResults, TagsRequest, TagsResults, VintageDates,
    VintageDatesRequest,
};

/// Base URL for the FRED REST API.
const FRED_BASE_URL: &str = "https://api.stlouisfed.org/fred";

/// Base URL for the GeoFRED / Maps API — a separate surface on the same host,
/// under `/geofred` instead of `/fred` (ADR-0025).
const GEOFRED_BASE_URL: &str = "https://api.stlouisfed.org/geofred";

/// An async client for the FRED API.
///
/// Cheap to clone — the underlying `reqwest::Client` holds a connection pool
/// behind an `Arc`, so clones share it.
#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
    geofred_base_url: String,
}

impl Client {
    /// Build a client with the given FRED API key.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying HTTP client cannot be built.
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        let http = reqwest::Client::builder().build()?;
        Ok(Self {
            http,
            api_key: api_key.into(),
            base_url: FRED_BASE_URL.to_owned(),
            geofred_base_url: GEOFRED_BASE_URL.to_owned(),
        })
    }

    /// Build a client pointed at a custom base URL. A test seam for aiming the
    /// client at a local mock HTTP server (ADR-0011); deliberately not public.
    #[cfg(test)]
    pub(crate) fn with_base_url(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Result<Self> {
        // Point both the core and GeoFRED bases at the same mock so path-only
        // matching works for either surface (ADR-0025).
        let base_url = base_url.into();
        Ok(Self {
            http: reqwest::Client::builder().build()?,
            api_key: api_key.into(),
            geofred_base_url: base_url.clone(),
            base_url,
        })
    }

    /// Build a client, reading the API key from the `FRED_API_KEY` environment
    /// variable.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInput`] if `FRED_API_KEY` is unset, or an error if
    /// the underlying HTTP client cannot be built.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("FRED_API_KEY").map_err(|_| {
            Error::InvalidInput("FRED_API_KEY environment variable is not set".to_owned())
        })?;
        Self::new(api_key)
    }

    /// Begin an observations request for a series.
    ///
    /// Returns a builder; set optional parameters (date range, units transform,
    /// frequency aggregation, sort order, paging) and call
    /// [`ObservationsRequest::send`] to run it. With nothing set, FRED's
    /// defaults apply (full history, levels, ascending by date).
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// use ferric_fred::{SeriesId, Units};
    /// let obs = client
    ///     .observations(&SeriesId::new("GNPCA"))
    ///     .units(Units::PercentChange)
    ///     .limit(10)
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn observations(&self, series_id: &SeriesId) -> ObservationsRequest<'_> {
        ObservationsRequest::new(self, series_id.clone())
    }

    /// Run an observations request (invoked by [`ObservationsRequest::send`]).
    pub(crate) async fn execute_observations(
        &self,
        request: &ObservationsRequest<'_>,
    ) -> Result<Vec<Observation>> {
        let response: ObservationsResponse = self
            .get("/series/observations", &request.query_params())
            .await?;
        Ok(response.observations)
    }

    /// Fetch metadata for a series (the `fred/series` endpoint).
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn series(&self, series_id: &SeriesId) -> Result<Series> {
        let response: SeriesResponse = self
            .get("/series", &[("series_id", series_id.as_str().to_owned())])
            .await?;
        response
            .seriess
            .into_iter()
            .next()
            .ok_or_else(|| Error::Api {
                status: 200,
                code: None,
                message: format!("FRED returned no series for id `{series_id}`"),
            })
    }

    /// Begin a search over series (the `fred/series/search` endpoint).
    ///
    /// Returns a builder; set optional parameters (search type, ordering, sort,
    /// paging) and call [`SeriesSearchRequest::send`] to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// use ferric_fred::OrderBy;
    /// let results = client
    ///     .search("industrial production")
    ///     .order_by(OrderBy::Popularity)
    ///     .limit(5)
    ///     .send()
    ///     .await?;
    /// println!("{} matches", results.count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn search(&self, search_text: impl Into<String>) -> SeriesSearchRequest<'_> {
        SeriesSearchRequest::new(self, search_text.into())
    }

    /// Run a search request (invoked by [`SeriesSearchRequest::send`]).
    pub(crate) async fn execute_search(
        &self,
        request: &SeriesSearchRequest<'_>,
    ) -> Result<SeriesSearchResults> {
        self.get("/series/search", &request.query_params()).await
    }

    /// Begin a request for the tags on the series matching a search (the
    /// `fred/series/search/tags` endpoint) — the tag facets of a full-text
    /// search, for narrowing it down.
    ///
    /// Returns a [`TagsRequest`] builder; set optional tag-filter text (sent as
    /// FRED's `tag_search_text`), sort, and paging, then call
    /// [`send`](TagsRequest::send) to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// let results = client.series_search_tags("unemployment").limit(10).send().await?;
    /// println!("{} tags", results.count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn series_search_tags(&self, search_text: impl Into<String>) -> TagsRequest<'_> {
        TagsRequest::scoped(
            self,
            "/series/search/tags",
            ("series_search_text", search_text.into()),
            None,
            "tag_search_text",
        )
    }

    /// Begin a request for the tags that co-occur, among the series matching a
    /// search, with a seed set of tags (the `fred/series/search/related_tags`
    /// endpoint).
    ///
    /// Accepts any iterable of seed tag names (joined with `;` for FRED).
    /// Returns a [`TagsRequest`] builder; set optional tag-filter text (sent as
    /// `tag_search_text`), sort, and paging, then call
    /// [`send`](TagsRequest::send) to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// let results = client
    ///     .series_search_related_tags("unemployment", ["monthly"])
    ///     .send()
    ///     .await?;
    /// println!("{} related tags", results.count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn series_search_related_tags<I, S>(
        &self,
        search_text: impl Into<String>,
        tag_names: I,
    ) -> TagsRequest<'_>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        TagsRequest::scoped(
            self,
            "/series/search/related_tags",
            ("series_search_text", search_text.into()),
            Some(join_tag_names(tag_names)),
            "tag_search_text",
        )
    }

    /// Fetch a single category by id (the `fred/category` endpoint). Use
    /// [`CategoryId::ROOT`] for the top of the tree.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn category(&self, category_id: CategoryId) -> Result<Category> {
        let response: CategoriesResponse = self
            .get(
                "/category",
                &[("category_id", category_id.get().to_string())],
            )
            .await?;
        response
            .categories
            .into_iter()
            .next()
            .ok_or_else(|| Error::Api {
                status: 200,
                code: None,
                message: format!("FRED returned no category for id `{category_id}`"),
            })
    }

    /// Fetch the child categories of a category (the `fred/category/children`
    /// endpoint) — the primary way to walk the category tree downward.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn category_children(&self, category_id: CategoryId) -> Result<Vec<Category>> {
        let response: CategoriesResponse = self
            .get(
                "/category/children",
                &[("category_id", category_id.get().to_string())],
            )
            .await?;
        Ok(response.categories)
    }

    /// Fetch the categories related to a category (the `fred/category/related`
    /// endpoint) — cross-links to sibling topics elsewhere in the tree, distinct
    /// from the parent/child hierarchy. FRED returns the full list unpaginated,
    /// so this yields a plain `Vec<Category>` (often empty).
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn category_related(&self, category_id: CategoryId) -> Result<Vec<Category>> {
        let response: CategoriesResponse = self
            .get(
                "/category/related",
                &[("category_id", category_id.get().to_string())],
            )
            .await?;
        Ok(response.categories)
    }

    /// Begin a request for the series in a category (the `fred/category/series`
    /// endpoint).
    ///
    /// Returns a [`SeriesListRequest`] builder; set optional ordering/paging and
    /// call [`send`](SeriesListRequest::send) to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// use ferric_fred::CategoryId;
    /// let results = client
    ///     .category_series(CategoryId::new(125))
    ///     .limit(5)
    ///     .send()
    ///     .await?;
    /// println!("{} series", results.count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn category_series(&self, category_id: CategoryId) -> SeriesListRequest<'_> {
        SeriesListRequest::new(
            self,
            "/category/series",
            "category_id",
            category_id.get().to_string(),
        )
    }

    /// Begin a request for the tags used by the series in a category (the
    /// `fred/category/tags` endpoint) — the tag facets available when browsing
    /// a category.
    ///
    /// Returns a [`TagsRequest`] builder; set optional tag-filter text/sort/
    /// paging and call [`send`](TagsRequest::send) to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// use ferric_fred::CategoryId;
    /// let results = client.category_tags(CategoryId::new(125)).limit(10).send().await?;
    /// println!("{} tags", results.count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn category_tags(&self, category_id: CategoryId) -> TagsRequest<'_> {
        TagsRequest::scoped(
            self,
            "/category/tags",
            ("category_id", category_id.get().to_string()),
            None,
            "search_text",
        )
    }

    /// Begin a request for the tags that co-occur, within a category, with a
    /// seed set of tags (the `fred/category/related_tags` endpoint) — refine a
    /// category browse by discovering adjacent tags.
    ///
    /// Accepts any iterable of seed tag names (joined with `;` for FRED).
    /// Returns a [`TagsRequest`] builder; set optional tag-filter text/sort/
    /// paging and call [`send`](TagsRequest::send) to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// use ferric_fred::CategoryId;
    /// let results = client.category_related_tags(CategoryId::new(125), ["gdp"]).send().await?;
    /// println!("{} related tags", results.count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn category_related_tags<I, S>(
        &self,
        category_id: CategoryId,
        tag_names: I,
    ) -> TagsRequest<'_>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        TagsRequest::scoped(
            self,
            "/category/related_tags",
            ("category_id", category_id.get().to_string()),
            Some(join_tag_names(tag_names)),
            "search_text",
        )
    }

    /// Begin a request listing all FRED data releases (the `fred/releases`
    /// endpoint) — a browse axis parallel to categories.
    ///
    /// Returns a builder; set optional sort/paging and call
    /// [`ReleasesRequest::send`] to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// let results = client.releases().limit(20).send().await?;
    /// println!("{} releases", results.count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn releases(&self) -> ReleasesRequest<'_> {
        ReleasesRequest::new(self, "/releases")
    }

    /// Run a releases request — `releases` or `source/releases` (invoked by
    /// [`ReleasesRequest::send`]).
    pub(crate) async fn execute_releases(
        &self,
        request: &ReleasesRequest<'_>,
    ) -> Result<ReleasesResults> {
        self.get(request.path(), &request.query_params()).await
    }

    /// Begin a request for the publication dates of *all* releases (the
    /// `fred/releases/dates` endpoint) — a release calendar across FRED,
    /// newest first by default.
    ///
    /// Returns a builder; set optional sort/paging (and
    /// [`include_dates_with_no_data`](ReleaseDatesRequest::include_dates_with_no_data))
    /// and call [`ReleaseDatesRequest::send`] to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// let calendar = client.releases_dates().limit(20).send().await?;
    /// println!("{} release dates", calendar.count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn releases_dates(&self) -> ReleaseDatesRequest<'_> {
        ReleaseDatesRequest::new(self, "/releases/dates")
    }

    /// Run a release-dates request — `releases/dates` or `release/dates`
    /// (invoked by [`ReleaseDatesRequest::send`]).
    pub(crate) async fn execute_release_dates(
        &self,
        request: &ReleaseDatesRequest<'_>,
    ) -> Result<ReleaseDatesResults> {
        self.get(request.path(), &request.query_params()).await
    }

    /// Fetch a single release by id (the `fred/release` endpoint).
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn release(&self, release_id: ReleaseId) -> Result<Release> {
        let response: ReleaseResponse = self
            .get("/release", &[("release_id", release_id.get().to_string())])
            .await?;
        response
            .releases
            .into_iter()
            .next()
            .ok_or_else(|| Error::Api {
                status: 200,
                code: None,
                message: format!("FRED returned no release for id `{release_id}`"),
            })
    }

    /// Begin a request for the series in a release (the `fred/release/series`
    /// endpoint).
    ///
    /// Returns a [`SeriesListRequest`] builder; set optional ordering/paging and
    /// call [`send`](SeriesListRequest::send) to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// use ferric_fred::ReleaseId;
    /// let results = client
    ///     .release_series(ReleaseId::new(53))
    ///     .limit(5)
    ///     .send()
    ///     .await?;
    /// println!("{} series", results.count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn release_series(&self, release_id: ReleaseId) -> SeriesListRequest<'_> {
        SeriesListRequest::new(
            self,
            "/release/series",
            "release_id",
            release_id.get().to_string(),
        )
    }

    /// Fetch the sources for a release (the `fred/release/sources` endpoint) —
    /// the reverse of [`source_releases`](Client::source_releases). FRED returns
    /// the full list unpaginated, so this yields a plain `Vec<Source>`.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn release_sources(&self, release_id: ReleaseId) -> Result<Vec<Source>> {
        let response: SourceResponse = self
            .get(
                "/release/sources",
                &[("release_id", release_id.get().to_string())],
            )
            .await?;
        Ok(response.sources)
    }

    /// Begin a request for the publication dates of *one* release (the
    /// `fred/release/dates` endpoint) — that release's calendar, oldest first
    /// by default.
    ///
    /// Returns a [`ReleaseDatesRequest`] builder; set optional sort/paging (and
    /// [`include_dates_with_no_data`](ReleaseDatesRequest::include_dates_with_no_data))
    /// and call [`send`](ReleaseDatesRequest::send) to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// use ferric_fred::ReleaseId;
    /// let dates = client.release_dates(ReleaseId::new(82)).limit(10).send().await?;
    /// println!("{} release dates", dates.count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn release_dates(&self, release_id: ReleaseId) -> ReleaseDatesRequest<'_> {
        ReleaseDatesRequest::with_release(self, "/release/dates", release_id.get().to_string())
    }

    /// Begin a request for a release's table tree (the `fred/release/tables`
    /// endpoint) — the nested layout (sections, tables, and series rows) a
    /// release uses to present its series.
    ///
    /// Returns a builder; optionally scope to a subtree with
    /// [`element`](ReleaseTablesRequest::element), then call
    /// [`ReleaseTablesRequest::send`] to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// use ferric_fred::ReleaseId;
    /// let table = client.release_tables(ReleaseId::new(10)).send().await?;
    /// println!("{} root elements", table.roots.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn release_tables(&self, release_id: ReleaseId) -> ReleaseTablesRequest<'_> {
        ReleaseTablesRequest::new(self, release_id.get())
    }

    /// Run a release/tables request (invoked by [`ReleaseTablesRequest::send`]).
    pub(crate) async fn execute_release_tables(
        &self,
        request: &ReleaseTablesRequest<'_>,
    ) -> Result<ReleaseTable> {
        self.get("/release/tables", &request.query_params()).await
    }

    /// Begin a request for the tags used by the series in a release (the
    /// `fred/release/tags` endpoint) — the tag facets available when browsing a
    /// release.
    ///
    /// Returns a [`TagsRequest`] builder; set optional tag-filter text/sort/
    /// paging and call [`send`](TagsRequest::send) to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// use ferric_fred::ReleaseId;
    /// let results = client.release_tags(ReleaseId::new(53)).limit(10).send().await?;
    /// println!("{} tags", results.count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn release_tags(&self, release_id: ReleaseId) -> TagsRequest<'_> {
        TagsRequest::scoped(
            self,
            "/release/tags",
            ("release_id", release_id.get().to_string()),
            None,
            "search_text",
        )
    }

    /// Begin a request for the tags that co-occur, within a release, with a
    /// seed set of tags (the `fred/release/related_tags` endpoint).
    ///
    /// Accepts any iterable of seed tag names (joined with `;` for FRED).
    /// Returns a [`TagsRequest`] builder; set optional tag-filter text/sort/
    /// paging and call [`send`](TagsRequest::send) to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// use ferric_fred::ReleaseId;
    /// let results = client.release_related_tags(ReleaseId::new(53), ["gdp"]).send().await?;
    /// println!("{} related tags", results.count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn release_related_tags<I, S>(&self, release_id: ReleaseId, tag_names: I) -> TagsRequest<'_>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        TagsRequest::scoped(
            self,
            "/release/related_tags",
            ("release_id", release_id.get().to_string()),
            Some(join_tag_names(tag_names)),
            "search_text",
        )
    }

    /// Begin a request to browse or search FRED's tag vocabulary (the
    /// `fred/tags` endpoint).
    ///
    /// Returns a builder; set optional search text/sort/paging and call
    /// [`TagsRequest::send`] to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// let results = client.tags().search_text("gdp").limit(10).send().await?;
    /// println!("{} tags", results.count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn tags(&self) -> TagsRequest<'_> {
        TagsRequest::new(self, "/tags")
    }

    /// Begin a request for the tags that co-occur with a seed set of tags (the
    /// `fred/related_tags` endpoint) — refine a faceted search by discovering
    /// adjacent tags.
    ///
    /// Accepts any iterable of tag names (they are joined with `;` for FRED).
    /// Returns a [`TagsRequest`] builder; set optional search text/sort/paging
    /// and call [`send`](TagsRequest::send) to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// let results = client.related_tags(["gdp"]).limit(10).send().await?;
    /// println!("{} tags related to gdp", results.count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn related_tags<I, S>(&self, tag_names: I) -> TagsRequest<'_>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        TagsRequest::with_tag_names(self, "/related_tags", join_tag_names(tag_names))
    }

    /// Run a tags request — `tags` or `related_tags` (invoked by
    /// [`TagsRequest::send`]).
    pub(crate) async fn execute_tags(&self, request: &TagsRequest<'_>) -> Result<TagsResults> {
        self.get(request.path(), &request.query_params()).await
    }

    /// Begin a request for the series carrying *all* of the given tags (the
    /// `fred/tags/series` endpoint) — faceted discovery.
    ///
    /// Accepts any iterable of tag names (they are joined with `;` for FRED).
    /// Returns a [`SeriesListRequest`] builder; set optional ordering/paging and
    /// call [`send`](SeriesListRequest::send) to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// let results = client.tags_series(["gdp", "quarterly"]).limit(5).send().await?;
    /// println!("{} series", results.count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn tags_series<I, S>(&self, tag_names: I) -> SeriesListRequest<'_>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        SeriesListRequest::new(self, "/tags/series", "tag_names", join_tag_names(tag_names))
    }

    /// Run a series-list request — `category/series`, `release/series`, or
    /// `tags/series` (invoked by [`SeriesListRequest::send`]).
    pub(crate) async fn execute_series_list(
        &self,
        request: &SeriesListRequest<'_>,
    ) -> Result<SeriesSearchResults> {
        self.get(request.path(), &request.query_params()).await
    }

    /// Fetch the tags attached to a series (the `fred/series/tags` endpoint) —
    /// the reverse of [`tags_series`](Client::tags_series).
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn series_tags(&self, series_id: &SeriesId) -> Result<TagsResults> {
        self.get(
            "/series/tags",
            &[("series_id", series_id.as_str().to_owned())],
        )
        .await
    }

    /// Begin a request for the most recently updated series (the
    /// `fred/series/updates` endpoint) — a "what changed" feed, ordered by
    /// last-updated time.
    ///
    /// Returns a builder; set an optional class filter/paging and call
    /// [`SeriesUpdatesRequest::send`] to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// let results = client.series_updates().limit(20).send().await?;
    /// println!("{} recently updated", results.count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn series_updates(&self) -> SeriesUpdatesRequest<'_> {
        SeriesUpdatesRequest::new(self)
    }

    /// Run a series/updates request (invoked by [`SeriesUpdatesRequest::send`]).
    pub(crate) async fn execute_series_updates(
        &self,
        request: &SeriesUpdatesRequest<'_>,
    ) -> Result<SeriesSearchResults> {
        self.get("/series/updates", &request.query_params()).await
    }

    /// Begin a request for a series' vintage dates (the
    /// `fred/series/vintagedates` endpoint) — the dates on which the series was
    /// revised or newly released.
    ///
    /// Returns a builder; set optional sort/paging and call
    /// [`VintageDatesRequest::send`] to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// use ferric_fred::SeriesId;
    /// let dates = client
    ///     .series_vintagedates(&SeriesId::new("GNPCA"))
    ///     .limit(10)
    ///     .send()
    ///     .await?;
    /// println!("{} vintage dates", dates.count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn series_vintagedates(&self, series_id: &SeriesId) -> VintageDatesRequest<'_> {
        VintageDatesRequest::new(self, series_id.clone())
    }

    /// Run a series/vintagedates request (invoked by
    /// [`VintageDatesRequest::send`]).
    pub(crate) async fn execute_vintage_dates(
        &self,
        request: &VintageDatesRequest<'_>,
    ) -> Result<VintageDates> {
        self.get("/series/vintagedates", &request.query_params())
            .await
    }

    /// Fetch the categories a series belongs to (the `fred/series/categories`
    /// endpoint) — the reverse of [`category_series`](Client::category_series).
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn series_categories(&self, series_id: &SeriesId) -> Result<Vec<Category>> {
        let response: CategoriesResponse = self
            .get(
                "/series/categories",
                &[("series_id", series_id.as_str().to_owned())],
            )
            .await?;
        Ok(response.categories)
    }

    /// Fetch the release a series belongs to (the `fred/series/release`
    /// endpoint) — the reverse of [`release_series`](Client::release_series).
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn series_release(&self, series_id: &SeriesId) -> Result<Release> {
        let response: ReleaseResponse = self
            .get(
                "/series/release",
                &[("series_id", series_id.as_str().to_owned())],
            )
            .await?;
        response
            .releases
            .into_iter()
            .next()
            .ok_or_else(|| Error::Api {
                status: 200,
                code: None,
                message: format!("FRED returned no release for series `{series_id}`"),
            })
    }

    /// Begin a request listing all FRED data sources (the `fred/sources`
    /// endpoint) — the organizations that produce releases.
    ///
    /// Returns a builder; set optional sort/paging and call
    /// [`SourcesRequest::send`] to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// let results = client.sources().limit(20).send().await?;
    /// println!("{} sources", results.count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn sources(&self) -> SourcesRequest<'_> {
        SourcesRequest::new(self)
    }

    /// Run a sources request (invoked by [`SourcesRequest::send`]).
    pub(crate) async fn execute_sources(
        &self,
        request: &SourcesRequest<'_>,
    ) -> Result<SourcesResults> {
        self.get("/sources", &request.query_params()).await
    }

    /// Fetch a single source by id (the `fred/source` endpoint).
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn source(&self, source_id: SourceId) -> Result<Source> {
        let response: SourceResponse = self
            .get("/source", &[("source_id", source_id.get().to_string())])
            .await?;
        response
            .sources
            .into_iter()
            .next()
            .ok_or_else(|| Error::Api {
                status: 200,
                code: None,
                message: format!("FRED returned no source for id `{source_id}`"),
            })
    }

    /// Begin a request for the releases produced by a source (the
    /// `fred/source/releases` endpoint).
    ///
    /// Returns a [`ReleasesRequest`] builder; set optional sort/paging and call
    /// [`send`](ReleasesRequest::send) to run it.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// use ferric_fred::SourceId;
    /// let results = client.source_releases(SourceId::new(18)).limit(5).send().await?;
    /// println!("{} releases", results.count);
    /// # Ok(())
    /// # }
    /// ```
    pub fn source_releases(&self, source_id: SourceId) -> ReleasesRequest<'_> {
        ReleasesRequest::with_source(self, "/source/releases", source_id.get().to_string())
    }

    // --- GeoFRED / Maps API (ADR-0025) ---------------------------------------

    /// Fetch a region cross-section for a series group (the GeoFRED
    /// `geofred/regional/data` endpoint) — the value in every region of
    /// `region_type` on `date`, for the given `units` label,
    /// `frequency`, and `season`.
    ///
    /// FRED requires **all** of these parameters (a live probe rejects any
    /// omission — ADR-0025), so this is a direct call rather than a builder.
    /// `units` is a free-form measurement label FRED echoes into the result
    /// title (e.g. `"Dollars"`), not a transformation code.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// use ferric_fred::{Frequency, RegionType, SeasonalAdjustment, SeriesGroupId};
    /// let date = chrono::NaiveDate::from_ymd_opt(2013, 1, 1).unwrap();
    /// let data = client
    ///     .regional_data(
    ///         &SeriesGroupId::new("882"),
    ///         RegionType::State,
    ///         date,
    ///         "Dollars",
    ///         Frequency::Annual,
    ///         SeasonalAdjustment::NotSeasonallyAdjusted,
    ///     )
    ///     .await?;
    /// println!("{}", data.meta.title);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn regional_data(
        &self,
        series_group: &SeriesGroupId,
        region_type: RegionType,
        date: NaiveDate,
        units: impl Into<String>,
        frequency: Frequency,
        season: SeasonalAdjustment,
    ) -> Result<RegionalData> {
        self.get_geofred(
            "/regional/data",
            &[
                ("series_group", series_group.as_str().to_owned()),
                ("region_type", region_type.query_code().to_owned()),
                ("date", date.to_string()),
                ("units", units.into()),
                ("frequency", frequency.query_code().to_owned()),
                ("season", season.query_code().to_owned()),
            ],
        )
        .await
    }

    /// Begin a request for one regional series' values across regions (the
    /// GeoFRED `geofred/series/data` endpoint).
    ///
    /// Returns a builder; set an optional [`date`](SeriesDataRequest::date) or
    /// [`start_date`](SeriesDataRequest::start_date) and call
    /// [`send`](SeriesDataRequest::send). With neither set, FRED returns the most
    /// recent date.
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// use ferric_fred::SeriesId;
    /// let data = client
    ///     .series_data(&SeriesId::new("SMU56000000500000001"))
    ///     .send()
    ///     .await?;
    /// println!("{} dates", data.meta.data.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn series_data(&self, series_id: &SeriesId) -> SeriesDataRequest<'_> {
        SeriesDataRequest::new(self, series_id.clone())
    }

    /// Run a GeoFRED series/data request (invoked by
    /// [`SeriesDataRequest::send`]).
    pub(crate) async fn execute_series_data(
        &self,
        request: &SeriesDataRequest<'_>,
    ) -> Result<RegionalData> {
        self.get_geofred("/series/data", &request.query_params())
            .await
    }

    /// Fetch the series-group metadata for a regional series (the GeoFRED
    /// `geofred/series/group` endpoint) — pass a regional `series_id` and get
    /// back the group it belongs to (title, region type, date span).
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn series_group(&self, series_id: &SeriesId) -> Result<SeriesGroup> {
        let response: SeriesGroupResponse = self
            .get_geofred(
                "/series/group",
                &[("series_id", series_id.as_str().to_owned())],
            )
            .await?;
        Ok(response.series_group)
    }

    /// Fetch the region boundary polygons for a shape type (the GeoFRED
    /// `geofred/shapes/file` endpoint) — a GeoJSON [`ShapeFile`] this crate
    /// transports without interpreting (ADR-0025).
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn shape_file(&self, shape: ShapeType) -> Result<ShapeFile> {
        self.get_geofred("/shapes/file", &[("shape", shape.query_code().to_owned())])
            .await
    }

    /// GET a core-FRED `path` with `params`; see [`get_from`](Self::get_from).
    async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&'static str, String)],
    ) -> Result<T> {
        self.get_from(&self.base_url, path, params).await
    }

    /// GET a GeoFRED / Maps `path` with `params` (the `/geofred` base);
    /// see [`get_from`](Self::get_from).
    async fn get_geofred<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&'static str, String)],
    ) -> Result<T> {
        self.get_from(&self.geofred_base_url, path, params).await
    }

    /// GET `base_url` + `path` with `params` plus `api_key`/`file_type`, then
    /// deserialize the JSON body as `T`. A non-success status becomes
    /// [`Error::Api`] (or [`Error::RateLimited`]); a body that doesn't match `T`
    /// becomes [`Error::Deserialize`].
    async fn get_from<T: DeserializeOwned>(
        &self,
        base_url: &str,
        path: &str,
        params: &[(&'static str, String)],
    ) -> Result<T> {
        let mut query: Vec<(&str, String)> = Vec::with_capacity(params.len() + 2);
        query.push(("api_key", self.api_key.clone()));
        query.push(("file_type", "json".to_owned()));
        query.extend(params.iter().cloned());

        let response = self
            .http
            .get(format!("{base_url}{path}"))
            .query(&query)
            .send()
            .await?;

        let status = response.status();
        // Read `Retry-After` before consuming the response into its body.
        let retry_after = parse_retry_after(response.headers());
        let body = response.bytes().await?;

        if !status.is_success() {
            return Err(api_error(status, retry_after, &body));
        }

        serde_json::from_slice(&body).map_err(Error::from)
    }
}

/// Join tag names with `;`, FRED's multi-value separator for the `tag_names`
/// parameter (shared by every endpoint that takes a seed tag set).
fn join_tag_names<I, S>(tag_names: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    tag_names
        .into_iter()
        .map(|name| name.as_ref().to_owned())
        .collect::<Vec<_>>()
        .join(";")
}

/// Build an [`Error`] from a non-success FRED response, decoding FRED's error
/// body (`{"error_code": N, "error_message": "..."}`) when present. `retry_after`
/// is the parsed `Retry-After` header, carried through on a `429`.
fn api_error(status: reqwest::StatusCode, retry_after: Option<Duration>, body: &[u8]) -> Error {
    let fred: Option<FredErrorBody> = serde_json::from_slice(body).ok();
    let code = fred.as_ref().and_then(|e| e.error_code);
    let message = fred.and_then(|e| e.error_message).unwrap_or_else(|| {
        status
            .canonical_reason()
            .unwrap_or("unknown error")
            .to_owned()
    });

    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Error::RateLimited { retry_after };
    }

    Error::Api {
        status: status.as_u16(),
        code,
        message,
    }
}

/// Parse a `Retry-After` header, if present, as a whole number of seconds
/// (FRED's form). The HTTP-date form is not used by FRED and is treated as
/// absent.
fn parse_retry_after(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    let seconds: u64 = headers
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse()
        .ok()?;
    Some(Duration::from_secs(seconds))
}

/// The `series/observations` response envelope. Metadata fields (realtime range,
/// units, paging) are ignored for this slice; serde drops unknown fields.
#[derive(Deserialize)]
struct ObservationsResponse {
    observations: Vec<Observation>,
}

/// The `series` response envelope. FRED pluralizes the array key as `seriess`
/// (sic); other metadata fields are ignored for this slice.
#[derive(Deserialize)]
struct SeriesResponse {
    seriess: Vec<Series>,
}

/// The `category` / `category/children` response envelope.
#[derive(Deserialize)]
struct CategoriesResponse {
    categories: Vec<Category>,
}

/// The single-`release` response envelope. The `releases` list endpoint
/// deserializes into [`ReleasesResults`] directly (it carries pagination);
/// `fred/release` returns only the array.
#[derive(Deserialize)]
struct ReleaseResponse {
    releases: Vec<Release>,
}

/// The `source` / `release/sources` response envelope: a bare `sources` array
/// (the paginated `sources` list endpoint deserializes into [`SourcesResults`]
/// directly; `release/sources` is unpaginated, so it uses this).
#[derive(Deserialize)]
struct SourceResponse {
    sources: Vec<Source>,
}

/// The GeoFRED `series/group` response envelope: `{ "series_group": { … } }`.
#[derive(Deserialize)]
struct SeriesGroupResponse {
    series_group: SeriesGroup,
}

/// FRED's error response body.
#[derive(Deserialize)]
struct FredErrorBody {
    error_code: Option<u32>,
    error_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::Client;
    use std::time::Duration;

    use crate::{
        CategoryId, Error, Frequency, OrderBy, Paginate, RegionType, ReleaseElementId, ReleaseId,
        SeasonalAdjustment, SeriesGroupId, SeriesId, ShapeType, SortOrder, SourceId, Units,
        UpdatesFilter,
    };
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// A representative `seriess[0]` object, reused across the response bodies.
    const SERIES_OBJECT: &str = r#"{
        "id": "GNPCA",
        "title": "Real Gross National Product",
        "observation_start": "1929-01-01",
        "observation_end": "2023-01-01",
        "frequency": "Annual",
        "units": "Billions of Chained 2017 Dollars",
        "seasonal_adjustment": "Not Seasonally Adjusted",
        "last_updated": "2024-03-28 07:56:03-05",
        "popularity": 76,
        "notes": "BEA Account Code: A001RX"
    }"#;

    fn client_for(server: &MockServer) -> Client {
        Client::with_base_url("test-key", server.uri()).expect("client builds")
    }

    #[tokio::test]
    async fn series_parses_metadata() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/series"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(format!("{{\"seriess\":[{SERIES_OBJECT}]}}")),
            )
            .mount(&server)
            .await;

        let series = client_for(&server)
            .series(&SeriesId::new("GNPCA"))
            .await
            .expect("series parses");
        assert_eq!(series.id, SeriesId::new("GNPCA"));
        assert_eq!(series.frequency, Frequency::Annual);
        assert_eq!(
            series.seasonal_adjustment,
            SeasonalAdjustment::NotSeasonallyAdjusted
        );
        assert_eq!(series.popularity, 76);
    }

    #[tokio::test]
    async fn observations_parse_missing_and_present_values() {
        let server = MockServer::start().await;
        let body = r#"{"observations":[
            {"realtime_start":"2026-07-06","realtime_end":"2026-07-06","date":"1930-01-01","value":"."},
            {"realtime_start":"2026-07-06","realtime_end":"2026-07-06","date":"1929-01-01","value":"1065.9"}
        ]}"#;
        Mock::given(method("GET"))
            .and(path("/series/observations"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&server)
            .await;

        let observations = client_for(&server)
            .observations(&SeriesId::new("GNPCA"))
            .send()
            .await
            .expect("observations parse");
        assert_eq!(observations.len(), 2);
        assert_eq!(observations[0].value, None); // the "." sentinel
        assert_eq!(observations[1].value, Some(1065.9));
    }

    #[tokio::test]
    async fn observations_point_in_time_sends_realtime_and_parses_period() {
        let server = MockServer::start().await;
        // A point-in-time query: realtime_start == realtime_end must reach the
        // wire, and each row's archived real-time period must deserialize.
        Mock::given(method("GET"))
            .and(path("/series/observations"))
            .and(query_param("realtime_start", "2020-01-01"))
            .and(query_param("realtime_end", "2020-01-01"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"observations":[
                    {"realtime_start":"2020-01-01","realtime_end":"2020-01-01","date":"2017-01-01","value":"18344.563"}
                ]}"#,
            ))
            .mount(&server)
            .await;

        let as_of = chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let observations = client_for(&server)
            .observations(&SeriesId::new("GNPCA"))
            .realtime(as_of, as_of)
            .send()
            .await
            .expect("point-in-time observations parse");
        assert_eq!(observations[0].realtime_start, as_of);
        assert_eq!(observations[0].realtime_end, as_of);
        assert_eq!(observations[0].value, Some(18344.563));
    }

    #[tokio::test]
    async fn search_parses_results_with_pagination() {
        let server = MockServer::start().await;
        let body =
            format!("{{\"count\":1,\"offset\":0,\"limit\":1000,\"seriess\":[{SERIES_OBJECT}]}}");
        Mock::given(method("GET"))
            .and(path("/series/search"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&server)
            .await;

        let results = client_for(&server)
            .search("real gnp")
            .send()
            .await
            .expect("search parses");
        assert_eq!(results.count, 1);
        assert_eq!(results.series.len(), 1);
        assert_eq!(results.series[0].id, SeriesId::new("GNPCA"));
    }

    #[tokio::test]
    async fn error_status_with_body_maps_to_api_error() {
        let server = MockServer::start().await;
        let body = r#"{"error_code":400,"error_message":"Bad Request. Invalid value for variable series_id."}"#;
        Mock::given(method("GET"))
            .and(path("/series"))
            .respond_with(ResponseTemplate::new(400).set_body_string(body))
            .mount(&server)
            .await;

        let error = client_for(&server)
            .series(&SeriesId::new("BAD"))
            .await
            .expect_err("a 400 should be an API error");
        match error {
            Error::Api {
                status,
                code,
                message,
            } => {
                assert_eq!(status, 400);
                assert_eq!(code, Some(400));
                assert!(message.contains("Invalid value"), "message was {message:?}");
            }
            other => panic!("expected Error::Api, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn too_many_requests_maps_to_rate_limited() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/series"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&server)
            .await;

        let error = client_for(&server)
            .series(&SeriesId::new("GNPCA"))
            .await
            .expect_err("a 429 should be rate-limited");
        assert!(matches!(error, Error::RateLimited { .. }), "got {error:?}");
    }

    #[tokio::test]
    async fn rate_limited_carries_retry_after_seconds() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/series"))
            .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "120"))
            .mount(&server)
            .await;

        let error = client_for(&server)
            .series(&SeriesId::new("GNPCA"))
            .await
            .expect_err("a 429 should be rate-limited");
        match error {
            Error::RateLimited { retry_after } => {
                assert_eq!(retry_after, Some(Duration::from_secs(120)));
            }
            other => panic!("expected Error::RateLimited, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn send_all_walks_every_page() {
        let server = MockServer::start().await;
        // First page (offset 0) reports a total of 3 and returns 2 sources; the
        // second page (offset 2) returns the last one. `send_all` should stitch
        // the two into one Vec and stop once it has walked past `count`.
        Mock::given(method("GET"))
            .and(path("/sources"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"count":3,"offset":0,"limit":1000,"sources":[
                    {"id":1,"name":"Source One"},
                    {"id":2,"name":"Source Two"}
                ]}"#,
            ))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/sources"))
            .and(query_param("offset", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"count":3,"offset":2,"limit":1000,"sources":[
                    {"id":3,"name":"Source Three"}
                ]}"#,
            ))
            .mount(&server)
            .await;

        let sources = client_for(&server)
            .sources()
            .send_all()
            .await
            .expect("send_all walks both pages");
        let ids: Vec<_> = sources.iter().map(|s| s.id).collect();
        assert_eq!(
            ids,
            vec![SourceId::new(1), SourceId::new(2), SourceId::new(3)]
        );
    }

    #[tokio::test]
    async fn send_all_treats_limit_as_a_ceiling() {
        let server = MockServer::start().await;
        // Only an offset-0 page is mocked, and only for a limit of 2. `count` is
        // 5, but a `.limit(2)` ceiling must stop `send_all` after one request of
        // exactly two — a second page request would 404 and fail the test.
        Mock::given(method("GET"))
            .and(path("/sources"))
            .and(query_param("offset", "0"))
            .and(query_param("limit", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"count":5,"offset":0,"limit":2,"sources":[
                    {"id":1,"name":"Source One"},
                    {"id":2,"name":"Source Two"}
                ]}"#,
            ))
            .mount(&server)
            .await;

        let sources = client_for(&server)
            .sources()
            .limit(2)
            .send_all()
            .await
            .expect("send_all stops at the ceiling");
        let ids: Vec<_> = sources.iter().map(|s| s.id).collect();
        assert_eq!(ids, vec![SourceId::new(1), SourceId::new(2)]);
    }

    #[tokio::test]
    async fn send_all_retries_after_a_429() {
        let server = MockServer::start().await;
        // The first request is rate-limited with `Retry-After: 0` (so the retry
        // sleeps for no real time); the retry then succeeds. Priority + a
        // one-shot cap make the 429 fire first, then fall through to the 200.
        Mock::given(method("GET"))
            .and(path("/sources"))
            .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "0"))
            .up_to_n_times(1)
            .with_priority(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/sources"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"count":1,"offset":0,"limit":1000,"sources":[
                    {"id":1,"name":"Source One"}
                ]}"#,
            ))
            .with_priority(2)
            .mount(&server)
            .await;

        let sources = client_for(&server)
            .sources()
            .send_all()
            .await
            .expect("send_all retries the 429 and then succeeds");
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].id, SourceId::new(1));
    }

    #[tokio::test]
    async fn stream_walks_every_page() {
        use futures_util::TryStreamExt;

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/sources"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"count":3,"offset":0,"limit":1000,"sources":[
                    {"id":1,"name":"Source One"},
                    {"id":2,"name":"Source Two"}
                ]}"#,
            ))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/sources"))
            .and(query_param("offset", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"count":3,"offset":2,"limit":1000,"sources":[
                    {"id":3,"name":"Source Three"}
                ]}"#,
            ))
            .mount(&server)
            .await;

        let sources: Vec<_> = client_for(&server)
            .sources()
            .stream()
            .try_collect()
            .await
            .expect("stream walks both pages");
        let ids: Vec<_> = sources.iter().map(|s| s.id).collect();
        assert_eq!(
            ids,
            vec![SourceId::new(1), SourceId::new(2), SourceId::new(3)]
        );
    }

    #[tokio::test]
    async fn stream_treats_limit_as_a_ceiling() {
        use futures_util::TryStreamExt;

        let server = MockServer::start().await;
        // Only an offset-0, limit-2 page is mocked; a `.limit(2)` ceiling must
        // stop the stream after it, without ever requesting a second page.
        Mock::given(method("GET"))
            .and(path("/sources"))
            .and(query_param("offset", "0"))
            .and(query_param("limit", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"count":5,"offset":0,"limit":2,"sources":[
                    {"id":1,"name":"Source One"},
                    {"id":2,"name":"Source Two"}
                ]}"#,
            ))
            .mount(&server)
            .await;

        let sources: Vec<_> = client_for(&server)
            .sources()
            .limit(2)
            .stream()
            .try_collect()
            .await
            .expect("stream stops at the ceiling");
        let ids: Vec<_> = sources.iter().map(|s| s.id).collect();
        assert_eq!(ids, vec![SourceId::new(1), SourceId::new(2)]);
    }

    #[tokio::test]
    async fn stream_surfaces_a_mid_stream_error() {
        use futures_util::StreamExt;

        let server = MockServer::start().await;
        // Page one succeeds; page two (offset 2) fails. The items from page one
        // arrive as `Ok`, then the error arrives as a final `Err` item.
        Mock::given(method("GET"))
            .and(path("/sources"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"count":4,"offset":0,"limit":1000,"sources":[
                    {"id":1,"name":"Source One"},
                    {"id":2,"name":"Source Two"}
                ]}"#,
            ))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/sources"))
            .and(query_param("offset", "2"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let results: Vec<_> = client_for(&server).sources().stream().collect().await;
        assert_eq!(results.len(), 3);
        assert_eq!(
            results[0].as_ref().expect("first item is Ok").id,
            SourceId::new(1)
        );
        assert_eq!(
            results[1].as_ref().expect("second item is Ok").id,
            SourceId::new(2)
        );
        assert!(
            matches!(results[2], Err(Error::Api { status: 500, .. })),
            "third item should be the page-two error, got {:?}",
            results[2]
        );
    }

    #[tokio::test]
    async fn malformed_body_maps_to_deserialize_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/series"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"unexpected":true}"#))
            .mount(&server)
            .await;

        let error = client_for(&server)
            .series(&SeriesId::new("GNPCA"))
            .await
            .expect_err("an unexpected body should fail to deserialize");
        assert!(matches!(error, Error::Deserialize(_)), "got {error:?}");
    }

    #[tokio::test]
    async fn request_carries_api_key_file_type_and_params() {
        let server = MockServer::start().await;
        // This mock only matches when every expected query parameter is present;
        // an unmatched request 404s and the call fails. So a *successful* call
        // proves the client sent api_key, file_type, and the builder's params.
        Mock::given(method("GET"))
            .and(path("/series/observations"))
            .and(query_param("api_key", "test-key"))
            .and(query_param("file_type", "json"))
            .and(query_param("units", "pch"))
            .and(query_param("limit", "5"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"observations":[]}"#))
            .mount(&server)
            .await;

        let observations = client_for(&server)
            .observations(&SeriesId::new("GNPCA"))
            .units(Units::PercentChange)
            .limit(5)
            .send()
            .await
            .expect("request with the expected params should match the mock");
        assert!(observations.is_empty());
    }

    #[tokio::test]
    async fn category_parses() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/category"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"categories":[{"id":125,"name":"Trade Balance","parent_id":13}]}"#,
            ))
            .mount(&server)
            .await;

        let category = client_for(&server)
            .category(CategoryId::new(125))
            .await
            .expect("category parses");
        assert_eq!(category.id, CategoryId::new(125));
        assert_eq!(category.name, "Trade Balance");
        assert_eq!(category.parent_id, CategoryId::new(13));
    }

    #[tokio::test]
    async fn category_children_parse() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/category/children"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"categories":[
                    {"id":16,"name":"Exports","parent_id":13},
                    {"id":17,"name":"Imports","parent_id":13}
                ]}"#,
            ))
            .mount(&server)
            .await;

        let children = client_for(&server)
            .category_children(CategoryId::new(13))
            .await
            .expect("children parse");
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].name, "Exports");
        assert_eq!(children[1].id, CategoryId::new(17));
    }

    #[tokio::test]
    async fn category_related_parse() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/category/related"))
            .and(query_param("category_id", "32073"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"categories":[
                    {"id":149,"name":"Arkansas","parent_id":27281},
                    {"id":150,"name":"Illinois","parent_id":27281}
                ]}"#,
            ))
            .mount(&server)
            .await;

        let related = client_for(&server)
            .category_related(CategoryId::new(32073))
            .await
            .expect("category/related parse");
        assert_eq!(related.len(), 2);
        assert_eq!(related[0].name, "Arkansas");
        assert_eq!(related[1].id, CategoryId::new(150));
    }

    #[tokio::test]
    async fn category_series_sends_params_and_parses() {
        let server = MockServer::start().await;
        // Matches only when the builder's params reach the wire.
        Mock::given(method("GET"))
            .and(path("/category/series"))
            .and(query_param("category_id", "125"))
            .and(query_param("order_by", "popularity"))
            .and(query_param("limit", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_string(format!(
                "{{\"count\":1,\"offset\":0,\"limit\":2,\"seriess\":[{SERIES_OBJECT}]}}"
            )))
            .mount(&server)
            .await;

        let results = client_for(&server)
            .category_series(CategoryId::new(125))
            .order_by(OrderBy::Popularity)
            .limit(2)
            .send()
            .await
            .expect("category series parse");
        assert_eq!(results.count, 1);
        assert_eq!(results.series[0].id, SeriesId::new("GNPCA"));
    }

    #[tokio::test]
    async fn releases_parse_with_pagination() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/releases"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"count":2,"offset":0,"limit":1000,"releases":[
                    {"id":9,"name":"Advance Monthly Sales","press_release":false},
                    {"id":53,"name":"Gross Domestic Product","press_release":true,"link":"http://bea.gov"}
                ]}"#,
            ))
            .mount(&server)
            .await;

        let results = client_for(&server)
            .releases()
            .send()
            .await
            .expect("releases parse");
        assert_eq!(results.count, 2);
        assert_eq!(results.releases[1].id, ReleaseId::new(53));
        assert_eq!(results.releases[1].link.as_deref(), Some("http://bea.gov"));
    }

    #[tokio::test]
    async fn release_parses() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/release"))
            .and(query_param("release_id", "53"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"releases":[{"id":53,"name":"Gross Domestic Product","press_release":true}]}"#,
            ))
            .mount(&server)
            .await;

        let release = client_for(&server)
            .release(ReleaseId::new(53))
            .await
            .expect("release parses");
        assert_eq!(release.id, ReleaseId::new(53));
        assert_eq!(release.name, "Gross Domestic Product");
        assert!(release.press_release);
    }

    #[tokio::test]
    async fn release_series_sends_params_and_parses() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/release/series"))
            .and(query_param("release_id", "53"))
            .and(query_param("limit", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_string(format!(
                "{{\"count\":1,\"offset\":0,\"limit\":2,\"seriess\":[{SERIES_OBJECT}]}}"
            )))
            .mount(&server)
            .await;

        let results = client_for(&server)
            .release_series(ReleaseId::new(53))
            .limit(2)
            .send()
            .await
            .expect("release series parse");
        assert_eq!(results.count, 1);
        assert_eq!(results.series[0].id, SeriesId::new("GNPCA"));
    }

    #[tokio::test]
    async fn tags_search_sends_text_and_parses() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/tags"))
            .and(query_param("search_text", "gdp"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"count":1,"offset":0,"limit":1000,"tags":[
                    {"name":"gdp","group_id":"gen","popularity":80,"series_count":12345}
                ]}"#,
            ))
            .mount(&server)
            .await;

        let results = client_for(&server)
            .tags()
            .search_text("gdp")
            .send()
            .await
            .expect("tags parse");
        assert_eq!(results.count, 1);
        assert_eq!(results.tags[0].name, "gdp");
        assert_eq!(results.tags[0].series_count, 12345);
    }

    #[tokio::test]
    async fn related_tags_send_seed_names_and_parses() {
        let server = MockServer::start().await;
        // The seed tags reach `/related_tags` joined by `;`.
        Mock::given(method("GET"))
            .and(path("/related_tags"))
            .and(query_param("tag_names", "gdp;quarterly"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"count":1,"offset":0,"limit":1000,"tags":[
                    {"name":"nsa","group_id":"seas","popularity":90,"series_count":42}
                ]}"#,
            ))
            .mount(&server)
            .await;

        let results = client_for(&server)
            .related_tags(["gdp", "quarterly"])
            .send()
            .await
            .expect("related_tags parse");
        assert_eq!(results.count, 1);
        assert_eq!(results.tags[0].name, "nsa");
    }

    /// A minimal single-tag `tags` response body, reused by the scoped-tag tests.
    const ONE_TAG_BODY: &str = r#"{"count":1,"offset":0,"limit":1000,"tags":[
        {"name":"gdp","group_id":"gen","popularity":80,"series_count":42}
    ]}"#;

    #[tokio::test]
    async fn category_tags_send_scope_and_parse() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/category/tags"))
            .and(query_param("category_id", "125"))
            .and(query_param("search_text", "gdp"))
            .respond_with(ResponseTemplate::new(200).set_body_string(ONE_TAG_BODY))
            .mount(&server)
            .await;

        let results = client_for(&server)
            .category_tags(CategoryId::new(125))
            .search_text("gdp")
            .send()
            .await
            .expect("category/tags parse");
        assert_eq!(results.count, 1);
        assert_eq!(results.tags[0].name, "gdp");
    }

    #[tokio::test]
    async fn category_related_tags_send_scope_and_seed_and_parse() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/category/related_tags"))
            .and(query_param("category_id", "125"))
            .and(query_param("tag_names", "gdp;quarterly"))
            .respond_with(ResponseTemplate::new(200).set_body_string(ONE_TAG_BODY))
            .mount(&server)
            .await;

        let results = client_for(&server)
            .category_related_tags(CategoryId::new(125), ["gdp", "quarterly"])
            .send()
            .await
            .expect("category/related_tags parse");
        assert_eq!(results.count, 1);
    }

    #[tokio::test]
    async fn release_tags_send_scope_and_parse() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/release/tags"))
            .and(query_param("release_id", "53"))
            .respond_with(ResponseTemplate::new(200).set_body_string(ONE_TAG_BODY))
            .mount(&server)
            .await;

        let results = client_for(&server)
            .release_tags(ReleaseId::new(53))
            .send()
            .await
            .expect("release/tags parse");
        assert_eq!(results.count, 1);
    }

    #[tokio::test]
    async fn release_related_tags_send_scope_and_seed_and_parse() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/release/related_tags"))
            .and(query_param("release_id", "53"))
            .and(query_param("tag_names", "gdp"))
            .respond_with(ResponseTemplate::new(200).set_body_string(ONE_TAG_BODY))
            .mount(&server)
            .await;

        let results = client_for(&server)
            .release_related_tags(ReleaseId::new(53), ["gdp"])
            .send()
            .await
            .expect("release/related_tags parse");
        assert_eq!(results.count, 1);
    }

    #[tokio::test]
    async fn series_search_tags_send_scope_and_tag_search_text() {
        let server = MockServer::start().await;
        // series/search/* sends the tag filter under `tag_search_text`, not
        // `search_text`; the mock only matches if that key is used.
        Mock::given(method("GET"))
            .and(path("/series/search/tags"))
            .and(query_param("series_search_text", "unemployment"))
            .and(query_param("tag_search_text", "rate"))
            .respond_with(ResponseTemplate::new(200).set_body_string(ONE_TAG_BODY))
            .mount(&server)
            .await;

        let results = client_for(&server)
            .series_search_tags("unemployment")
            .search_text("rate")
            .send()
            .await
            .expect("series/search/tags parse");
        assert_eq!(results.count, 1);
    }

    #[tokio::test]
    async fn series_search_related_tags_send_scope_and_seed() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/series/search/related_tags"))
            .and(query_param("series_search_text", "unemployment"))
            .and(query_param("tag_names", "monthly"))
            .respond_with(ResponseTemplate::new(200).set_body_string(ONE_TAG_BODY))
            .mount(&server)
            .await;

        let results = client_for(&server)
            .series_search_related_tags("unemployment", ["monthly"])
            .send()
            .await
            .expect("series/search/related_tags parse");
        assert_eq!(results.count, 1);
    }

    #[tokio::test]
    async fn tags_series_joins_names_and_parses() {
        let server = MockServer::start().await;
        // The two tag names must reach the wire joined by `;`.
        Mock::given(method("GET"))
            .and(path("/tags/series"))
            .and(query_param("tag_names", "gdp;quarterly"))
            .and(query_param("limit", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_string(format!(
                "{{\"count\":1,\"offset\":0,\"limit\":2,\"seriess\":[{SERIES_OBJECT}]}}"
            )))
            .mount(&server)
            .await;

        let results = client_for(&server)
            .tags_series(["gdp", "quarterly"])
            .limit(2)
            .send()
            .await
            .expect("tags/series parse");
        assert_eq!(results.count, 1);
        assert_eq!(results.series[0].id, SeriesId::new("GNPCA"));
    }

    #[tokio::test]
    async fn series_tags_parses() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/series/tags"))
            .and(query_param("series_id", "GNPCA"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"count":2,"offset":0,"limit":1000,"tags":[
                    {"name":"gnp","group_id":"gen","popularity":50,"series_count":10},
                    {"name":"usa","group_id":"geo","notes":null,"popularity":100,"series_count":500}
                ]}"#,
            ))
            .mount(&server)
            .await;

        let results = client_for(&server)
            .series_tags(&SeriesId::new("GNPCA"))
            .await
            .expect("series/tags parse");
        assert_eq!(results.count, 2);
        assert_eq!(results.tags[0].name, "gnp");
        assert!(results.tags[1].notes.is_none());
    }

    #[tokio::test]
    async fn sources_parse_with_pagination() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/sources"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"count":2,"offset":0,"limit":1000,"sources":[
                    {"id":1,"name":"Board of Governors of the Federal Reserve System (US)"},
                    {"id":18,"name":"U.S. Bureau of Economic Analysis","link":"http://bea.gov"}
                ]}"#,
            ))
            .mount(&server)
            .await;

        let results = client_for(&server)
            .sources()
            .send()
            .await
            .expect("sources parse");
        assert_eq!(results.count, 2);
        assert_eq!(results.sources[1].id, SourceId::new(18));
        assert_eq!(results.sources[1].link.as_deref(), Some("http://bea.gov"));
    }

    #[tokio::test]
    async fn source_parses() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/source"))
            .and(query_param("source_id", "18"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"sources":[{"id":18,"name":"U.S. Bureau of Economic Analysis","link":"http://bea.gov"}]}"#,
            ))
            .mount(&server)
            .await;

        let source = client_for(&server)
            .source(SourceId::new(18))
            .await
            .expect("source parses");
        assert_eq!(source.id, SourceId::new(18));
        assert_eq!(source.name, "U.S. Bureau of Economic Analysis");
    }

    #[tokio::test]
    async fn source_releases_send_source_id_and_parse() {
        let server = MockServer::start().await;
        // The source_id reaches `/source/releases`, which returns releases.
        Mock::given(method("GET"))
            .and(path("/source/releases"))
            .and(query_param("source_id", "18"))
            .and(query_param("limit", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"count":1,"offset":0,"limit":2,"releases":[
                    {"id":53,"name":"Gross Domestic Product","press_release":true}
                ]}"#,
            ))
            .mount(&server)
            .await;

        let results = client_for(&server)
            .source_releases(SourceId::new(18))
            .limit(2)
            .send()
            .await
            .expect("source/releases parse");
        assert_eq!(results.count, 1);
        assert_eq!(results.releases[0].id, ReleaseId::new(53));
    }

    #[tokio::test]
    async fn release_sources_send_release_id_and_parse() {
        let server = MockServer::start().await;
        // The release_id reaches `/release/sources`, which returns a bare
        // (unpaginated) `sources` array wrapped alongside realtime fields.
        Mock::given(method("GET"))
            .and(path("/release/sources"))
            .and(query_param("release_id", "51"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"realtime_start":"2013-08-14","realtime_end":"2013-08-14","sources":[
                    {"id":18,"name":"U.S. Bureau of Economic Analysis","link":"http://www.bea.gov/"},
                    {"id":19,"name":"U.S. Census Bureau"}
                ]}"#,
            ))
            .mount(&server)
            .await;

        let sources = client_for(&server)
            .release_sources(ReleaseId::new(51))
            .await
            .expect("release/sources parse");
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].id, SourceId::new(18));
        assert_eq!(sources[0].link.as_deref(), Some("http://www.bea.gov/"));
        assert!(sources[1].link.is_none());
    }

    #[tokio::test]
    async fn releases_dates_send_params_and_parse() {
        let server = MockServer::start().await;
        // The `/releases/dates` calendar carries a release_name per entry.
        Mock::given(method("GET"))
            .and(path("/releases/dates"))
            .and(query_param("sort_order", "desc"))
            .and(query_param("limit", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"count":2,"offset":0,"limit":2,"release_dates":[
                    {"release_id":9,"release_name":"Advance Monthly Sales","date":"2013-08-13"},
                    {"release_id":10,"release_name":"Consumer Price Index","date":"2013-08-15"}
                ]}"#,
            ))
            .mount(&server)
            .await;

        let results = client_for(&server)
            .releases_dates()
            .sort_order(SortOrder::Descending)
            .limit(2)
            .send()
            .await
            .expect("releases/dates parse");
        assert_eq!(results.count, 2);
        assert_eq!(results.release_dates[0].release_id, ReleaseId::new(9));
        assert_eq!(
            results.release_dates[0].release_name.as_deref(),
            Some("Advance Monthly Sales")
        );
    }

    #[tokio::test]
    async fn release_dates_send_release_id_and_include_flag_and_parse() {
        let server = MockServer::start().await;
        // `/release/dates` fixes the release, so entries omit release_name; the
        // request must carry release_id and the include-no-data toggle.
        Mock::given(method("GET"))
            .and(path("/release/dates"))
            .and(query_param("release_id", "82"))
            .and(query_param("include_release_dates_with_no_data", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"count":2,"offset":0,"limit":10000,"release_dates":[
                    {"release_id":82,"date":"1997-02-10"},
                    {"release_id":82,"date":"1998-02-10"}
                ]}"#,
            ))
            .mount(&server)
            .await;

        let results = client_for(&server)
            .release_dates(ReleaseId::new(82))
            .include_dates_with_no_data(true)
            .send()
            .await
            .expect("release/dates parse");
        assert_eq!(results.count, 2);
        assert_eq!(results.release_dates[0].release_id, ReleaseId::new(82));
        assert!(results.release_dates[0].release_name.is_none());
    }

    #[tokio::test]
    async fn release_tables_send_element_and_parse_tree() {
        let server = MockServer::start().await;
        // The element_id (subtree scope) must reach the wire, and the nested
        // tree — a section containing a series row — must deserialize.
        Mock::given(method("GET"))
            .and(path("/release/tables"))
            .and(query_param("release_id", "10"))
            .and(query_param("element_id", "34483"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"name":"Monthly, SA","element_id":34483,"release_id":"10","elements":{
                    "34484":{"element_id":34484,"release_id":10,"parent_id":34483,
                        "series_id":"","type":"series","name":"All items","line":"1","level":"0",
                        "children":[
                            {"element_id":34485,"release_id":10,"parent_id":34484,
                             "series_id":"CPIFABSL","type":"series","name":"Food",
                             "line":"2","level":"1","children":[]}
                        ]}
                }}"#,
            ))
            .mount(&server)
            .await;

        let table = client_for(&server)
            .release_tables(ReleaseId::new(10))
            .element(ReleaseElementId::new(34483))
            .send()
            .await
            .expect("release/tables parse");
        assert_eq!(table.name.as_deref(), Some("Monthly, SA"));
        assert_eq!(table.roots.len(), 1);
        let leaf = &table.roots[0].children[0];
        assert_eq!(
            leaf.series_id.as_ref().map(|s| s.as_str()),
            Some("CPIFABSL")
        );
    }

    #[tokio::test]
    async fn release_tables_observation_values_reach_wire_and_parse() {
        let server = MockServer::start().await;
        // `.observation_date(..)` must send both `observation_date` (ISO) and
        // `include_observation_values=true`, and the per-element value/date
        // fields must deserialize onto the returned series row.
        Mock::given(method("GET"))
            .and(path("/release/tables"))
            .and(query_param("release_id", "10"))
            .and(query_param("include_observation_values", "true"))
            .and(query_param("observation_date", "2023-06-01"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"release_id":"10","elements":{
                    "36715":{"element_id":36715,"release_id":10,"parent_id":36714,
                        "series_id":"CUSR0000SA0L5","type":"series","name":"All items",
                        "level":"1","observation_value":"292.260","observation_date":"Jun 2023",
                        "children":[]}
                }}"#,
            ))
            .mount(&server)
            .await;

        let table = client_for(&server)
            .release_tables(ReleaseId::new(10))
            .observation_date(chrono::NaiveDate::from_ymd_opt(2023, 6, 1).unwrap())
            .send()
            .await
            .expect("release/tables with values parse");
        let row = &table.roots[0];
        assert_eq!(row.observation_value, Some(292.260));
        assert_eq!(row.observation_date.as_deref(), Some("Jun 2023"));
    }

    #[tokio::test]
    async fn series_categories_parse() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/series/categories"))
            .and(query_param("series_id", "GNPCA"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"categories":[
                    {"id":106,"name":"Gross National Product","parent_id":18},
                    {"id":18,"name":"National Income & Product Accounts","parent_id":13}
                ]}"#,
            ))
            .mount(&server)
            .await;

        let categories = client_for(&server)
            .series_categories(&SeriesId::new("GNPCA"))
            .await
            .expect("series/categories parse");
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0].id, CategoryId::new(106));
    }

    #[tokio::test]
    async fn series_release_parses() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/series/release"))
            .and(query_param("series_id", "GNPCA"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"releases":[{"id":53,"name":"Gross Domestic Product","press_release":true}]}"#,
            ))
            .mount(&server)
            .await;

        let release = client_for(&server)
            .series_release(&SeriesId::new("GNPCA"))
            .await
            .expect("series/release parse");
        assert_eq!(release.id, ReleaseId::new(53));
        assert_eq!(release.name, "Gross Domestic Product");
    }

    #[tokio::test]
    async fn series_updates_sends_filter_and_parses() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/series/updates"))
            .and(query_param("filter_value", "macro"))
            .and(query_param("limit", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_string(format!(
                "{{\"count\":5,\"offset\":0,\"limit\":2,\"seriess\":[{SERIES_OBJECT}]}}"
            )))
            .mount(&server)
            .await;

        let results = client_for(&server)
            .series_updates()
            .filter(UpdatesFilter::Macro)
            .limit(2)
            .send()
            .await
            .expect("series/updates parse");
        assert_eq!(results.count, 5);
        assert_eq!(results.series[0].id, SeriesId::new("GNPCA"));
    }

    #[tokio::test]
    async fn series_updates_sends_time_window_as_yyyymmddhhmm() {
        use chrono::NaiveDate;
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/series/updates"))
            .and(query_param("start_time", "201803021420"))
            .and(query_param("end_time", "201803030905"))
            .respond_with(ResponseTemplate::new(200).set_body_string(format!(
                "{{\"count\":1,\"offset\":0,\"limit\":1,\"seriess\":[{SERIES_OBJECT}]}}"
            )))
            .mount(&server)
            .await;

        let start = NaiveDate::from_ymd_opt(2018, 3, 2)
            .unwrap()
            .and_hms_opt(14, 20, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2018, 3, 3)
            .unwrap()
            .and_hms_opt(9, 5, 0)
            .unwrap();
        let results = client_for(&server)
            .series_updates()
            .time_window(start, end)
            .send()
            .await
            .expect("series/updates time-window parse");
        assert_eq!(results.count, 1);
    }

    #[tokio::test]
    async fn series_vintagedates_send_id_and_parse() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/series/vintagedates"))
            .and(query_param("series_id", "GNPCA"))
            .and(query_param("limit", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"count":3,"offset":0,"limit":2,"vintage_dates":["1958-12-21","1959-02-19"]}"#,
            ))
            .mount(&server)
            .await;

        let dates = client_for(&server)
            .series_vintagedates(&SeriesId::new("GNPCA"))
            .limit(2)
            .send()
            .await
            .expect("series/vintagedates parse");
        assert_eq!(dates.count, 3);
        assert_eq!(dates.vintage_dates.len(), 2);
        assert_eq!(
            dates.vintage_dates[0],
            chrono::NaiveDate::from_ymd_opt(1958, 12, 21).unwrap()
        );
    }

    // --- GeoFRED / Maps (ADR-0025) -------------------------------------------

    #[tokio::test]
    async fn geofred_regional_data_sends_all_params_and_parses() {
        let server = MockServer::start().await;
        // The mock matches only when every required param — including the enum
        // query codes (region_type=state, frequency=a, season=NSA) — reaches the
        // wire on the `/geofred` base.
        Mock::given(method("GET"))
            .and(path("/regional/data"))
            .and(query_param("series_group", "882"))
            .and(query_param("region_type", "state"))
            .and(query_param("date", "2013-01-01"))
            .and(query_param("units", "Dollars"))
            .and(query_param("frequency", "a"))
            .and(query_param("season", "NSA"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"meta":{"title":"t","region":"state","seasonality":"Not Seasonally Adjusted",
                    "units":"Dollars","frequency":"Annual","data":{"2013-01-01":[
                        {"region":"Alabama","code":"01","value":35706,"series_id":"ALPCPI"}
                    ]}}}"#,
            ))
            .mount(&server)
            .await;

        let data = client_for(&server)
            .regional_data(
                &SeriesGroupId::new("882"),
                RegionType::State,
                chrono::NaiveDate::from_ymd_opt(2013, 1, 1).unwrap(),
                "Dollars",
                Frequency::Annual,
                SeasonalAdjustment::NotSeasonallyAdjusted,
            )
            .await
            .expect("regional data parses");
        let day = &data.meta.data["2013-01-01"];
        assert_eq!(day[0].region, "Alabama");
        assert_eq!(day[0].value, Some(35706.0));
        assert_eq!(day[0].series_id, SeriesId::new("ALPCPI"));
    }

    #[tokio::test]
    async fn geofred_series_data_sends_optional_date_and_parses() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/series/data"))
            .and(query_param("series_id", "SMU56000000500000001"))
            .and(query_param("date", "2013-01-01"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"meta":{"title":"t","region":"state","seasonality":"Not Seasonally Adjusted",
                    "units":"Thousands of Persons","frequency":"Monthly","data":{"2013-01-01":[
                        {"region":"Alabama","code":"01","value":1506.5,"series_id":"SMU01000000500000001"}
                    ]}}}"#,
            ))
            .mount(&server)
            .await;

        let data = client_for(&server)
            .series_data(&SeriesId::new("SMU56000000500000001"))
            .date(chrono::NaiveDate::from_ymd_opt(2013, 1, 1).unwrap())
            .send()
            .await
            .expect("series data parses");
        assert_eq!(data.meta.data["2013-01-01"][0].value, Some(1506.5));
    }

    #[tokio::test]
    async fn geofred_series_group_unwraps_envelope_and_parses() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/series/group"))
            .and(query_param("series_id", "SMU56000000500000001"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"series_group":{"title":"All Employees: Total Private","region_type":"state",
                    "series_group":"1223","season":"NSA","units":"Thousands of Persons",
                    "frequency":"Monthly","min_date":"1990-01-01","max_date":"2026-05-01"}}"#,
            ))
            .mount(&server)
            .await;

        let group = client_for(&server)
            .series_group(&SeriesId::new("SMU56000000500000001"))
            .await
            .expect("series group parses");
        assert_eq!(group.id, SeriesGroupId::new("1223"));
        assert_eq!(group.region_type, "state");
    }

    #[tokio::test]
    async fn geofred_shape_file_sends_shape_and_parses_geojson() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/shapes/file"))
            .and(query_param("shape", "bea"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"type":"FeatureCollection","name":"state_bea_region","features":[
                    {"type":"Feature","properties":{"bea_region":8},
                     "geometry":{"type":"MultiPolygon","coordinates":[[[[1485,2651]]]]}}
                ]}"#,
            ))
            .mount(&server)
            .await;

        let shapes = client_for(&server)
            .shape_file(ShapeType::Bea)
            .await
            .expect("shape file parses");
        assert_eq!(shapes.kind, "FeatureCollection");
        assert_eq!(shapes.features[0].geometry.kind, "MultiPolygon");
    }
}
