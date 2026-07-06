use crate::{Client, ReleaseDatesResults, Result, SortOrder};

/// A builder for the release-date endpoints, returned by
/// [`Client::releases_dates`] (`fred/releases/dates`, the publication dates of
/// *every* release) and [`Client::release_dates`] (`fred/release/dates`, the
/// dates of *one* release). Both share optional sort, paging, and the
/// "include dates with no data" toggle and return [`ReleaseDatesResults`];
/// `release_dates` additionally carries the `release_id`. Finish with
/// [`send`](ReleaseDatesRequest::send).
///
/// ```no_run
/// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
/// let calendar = client.releases_dates().limit(20).send().await?;
/// println!("{} release dates", calendar.count);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
#[must_use = "a ReleaseDatesRequest does nothing until you call `.send()`"]
pub struct ReleaseDatesRequest<'a> {
    client: &'a Client,
    /// The endpoint path, `/releases/dates` or `/release/dates`.
    path: &'static str,
    /// FRED's `release_id`; required by `/release/dates`, absent for
    /// `/releases/dates`.
    release_id: Option<String>,
    sort_order: Option<SortOrder>,
    limit: Option<u32>,
    offset: Option<u32>,
    include_dates_with_no_data: Option<bool>,
}

impl<'a> ReleaseDatesRequest<'a> {
    pub(crate) fn new(client: &'a Client, path: &'static str) -> Self {
        Self {
            client,
            path,
            release_id: None,
            sort_order: None,
            limit: None,
            offset: None,
            include_dates_with_no_data: None,
        }
    }

    /// Construct a request for `/release/dates` scoped to one release.
    pub(crate) fn with_release(client: &'a Client, path: &'static str, release_id: String) -> Self {
        Self {
            release_id: Some(release_id),
            ..Self::new(client, path)
        }
    }

    /// The endpoint path this request targets (used by the client to dispatch).
    pub(crate) fn path(&self) -> &'static str {
        self.path
    }

    /// Sort order of the dates (`sort_order`).
    pub fn sort_order(mut self, order: SortOrder) -> Self {
        self.sort_order = Some(order);
        self
    }

    /// Maximum number of results to return, `1..=10000` (`limit`).
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Number of results to skip from the start (`offset`), for paging.
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Include release dates for which no observations have yet been released
    /// (`include_release_dates_with_no_data`) — e.g. a scheduled future date.
    /// FRED omits these by default.
    pub fn include_dates_with_no_data(mut self, include: bool) -> Self {
        self.include_dates_with_no_data = Some(include);
        self
    }

    /// Run the request and return a page of release dates with pagination
    /// metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn send(self) -> Result<ReleaseDatesResults> {
        self.client.execute_release_dates(&self).await
    }

    /// Serialize the set parameters to FRED query key/value pairs. `api_key` and
    /// `file_type` are added by the client, not here.
    pub(crate) fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params: Vec<(&'static str, String)> = Vec::new();
        if let Some(release_id) = &self.release_id {
            params.push(("release_id", release_id.clone()));
        }
        if let Some(order) = self.sort_order {
            params.push(("sort_order", order.query_code().to_owned()));
        }
        if let Some(limit) = self.limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(offset) = self.offset {
            params.push(("offset", offset.to_string()));
        }
        if let Some(include) = self.include_dates_with_no_data {
            params.push(("include_release_dates_with_no_data", include.to_string()));
        }
        params
    }
}

impl crate::paginate::sealed::Sealed for ReleaseDatesRequest<'_> {}
impl crate::paginate::Paginate for ReleaseDatesRequest<'_> {
    type Page = ReleaseDatesResults;
    const MAX_PAGE: u32 = 10_000;
    fn requested_limit(&self) -> Option<u32> {
        self.limit
    }
    fn requested_offset(&self) -> Option<u32> {
        self.offset
    }
    fn with_paging(self, limit: u32, offset: u32) -> Self {
        self.limit(limit).offset(offset)
    }
    fn send_page(self) -> impl std::future::Future<Output = Result<Self::Page>> + Send {
        self.send()
    }
}
