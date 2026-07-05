use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::{
    Category, CategoryId, Error, Observation, ObservationsRequest, Release, ReleaseId,
    ReleasesRequest, ReleasesResults, Result, Series, SeriesId, SeriesListRequest,
    SeriesSearchRequest, SeriesSearchResults, SeriesUpdatesRequest, Source, SourceId,
    SourcesRequest, SourcesResults, TagsRequest, TagsResults, VintageDates, VintageDatesRequest,
};

/// Base URL for the FRED REST API.
const FRED_BASE_URL: &str = "https://api.stlouisfed.org/fred";

/// An async client for the FRED API.
///
/// Cheap to clone — the underlying `reqwest::Client` holds a connection pool
/// behind an `Arc`, so clones share it.
#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
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
        })
    }

    /// Build a client pointed at a custom base URL. A test seam for aiming the
    /// client at a local mock HTTP server (ADR-0011); deliberately not public.
    #[cfg(test)]
    pub(crate) fn with_base_url(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            http: reqwest::Client::builder().build()?,
            api_key: api_key.into(),
            base_url: base_url.into(),
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
        let joined = tag_names
            .into_iter()
            .map(|name| name.as_ref().to_owned())
            .collect::<Vec<_>>()
            .join(";");
        TagsRequest::with_tag_names(self, "/related_tags", joined)
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
        let joined = tag_names
            .into_iter()
            .map(|name| name.as_ref().to_owned())
            .collect::<Vec<_>>()
            .join(";");
        SeriesListRequest::new(self, "/tags/series", "tag_names", joined)
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

    /// GET `path` with `params` plus `api_key`/`file_type`, then deserialize the
    /// JSON body as `T`. A non-success status becomes [`Error::Api`] (or
    /// [`Error::RateLimited`]); a body that doesn't match `T` becomes
    /// [`Error::Deserialize`].
    async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&'static str, String)],
    ) -> Result<T> {
        let mut query: Vec<(&str, String)> = Vec::with_capacity(params.len() + 2);
        query.push(("api_key", self.api_key.clone()));
        query.push(("file_type", "json".to_owned()));
        query.extend(params.iter().cloned());

        let response = self
            .http
            .get(format!("{}{}", self.base_url, path))
            .query(&query)
            .send()
            .await?;

        let status = response.status();
        let body = response.bytes().await?;

        if !status.is_success() {
            return Err(api_error(status, &body));
        }

        serde_json::from_slice(&body).map_err(Error::from)
    }
}

/// Build an [`Error`] from a non-success FRED response, decoding FRED's error
/// body (`{"error_code": N, "error_message": "..."}`) when present.
fn api_error(status: reqwest::StatusCode, body: &[u8]) -> Error {
    let fred: Option<FredErrorBody> = serde_json::from_slice(body).ok();
    let code = fred.as_ref().and_then(|e| e.error_code);
    let message = fred.and_then(|e| e.error_message).unwrap_or_else(|| {
        status
            .canonical_reason()
            .unwrap_or("unknown error")
            .to_owned()
    });

    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Error::RateLimited { retry_after: None };
    }

    Error::Api {
        status: status.as_u16(),
        code,
        message,
    }
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

/// FRED's error response body.
#[derive(Deserialize)]
struct FredErrorBody {
    error_code: Option<u32>,
    error_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::Client;
    use crate::{
        CategoryId, Error, Frequency, OrderBy, ReleaseId, SeasonalAdjustment, SeriesId, SourceId,
        Units, UpdatesFilter,
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
            {"date":"1930-01-01","value":"."},
            {"date":"1929-01-01","value":"1065.9"}
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
}
