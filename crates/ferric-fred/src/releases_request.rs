use crate::{Client, ReleasesResults, Result, SortOrder};

/// A builder for the release-listing endpoints, returned by
/// [`Client::releases`] (`fred/releases`, all data releases) and
/// [`Client::source_releases`] (`fred/source/releases`, the releases of one
/// source). Both share optional sort and paging and return [`ReleasesResults`];
/// `source_releases` additionally carries the `source_id`. Finish with
/// [`send`](ReleasesRequest::send).
///
/// ```no_run
/// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
/// let results = client.releases().limit(20).send().await?;
/// println!("{} releases", results.count);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
#[must_use = "a ReleasesRequest does nothing until you call `.send()`"]
pub struct ReleasesRequest<'a> {
    client: &'a Client,
    /// The endpoint path, `/releases` or `/source/releases`.
    path: &'static str,
    /// FRED's `source_id`; required by `/source/releases`, absent for
    /// `/releases`.
    source_id: Option<String>,
    sort_order: Option<SortOrder>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl<'a> ReleasesRequest<'a> {
    pub(crate) fn new(client: &'a Client, path: &'static str) -> Self {
        Self {
            client,
            path,
            source_id: None,
            sort_order: None,
            limit: None,
            offset: None,
        }
    }

    /// Construct a request for `/source/releases` filtered to one source.
    pub(crate) fn with_source(client: &'a Client, path: &'static str, source_id: String) -> Self {
        Self {
            source_id: Some(source_id),
            ..Self::new(client, path)
        }
    }

    /// The endpoint path this request targets (used by the client to dispatch).
    pub(crate) fn path(&self) -> &'static str {
        self.path
    }

    /// Sort order of the results by release id (`sort_order`).
    pub fn sort_order(mut self, order: SortOrder) -> Self {
        self.sort_order = Some(order);
        self
    }

    /// Maximum number of results to return, `1..=1000` (`limit`).
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Number of results to skip from the start (`offset`), for paging.
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Run the request and return a page of releases with pagination metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn send(self) -> Result<ReleasesResults> {
        self.client.execute_releases(&self).await
    }

    /// Serialize the set parameters to FRED query key/value pairs. `api_key` and
    /// `file_type` are added by the client, not here.
    pub(crate) fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params: Vec<(&'static str, String)> = Vec::new();
        if let Some(source_id) = &self.source_id {
            params.push(("source_id", source_id.clone()));
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
        params
    }
}

impl crate::paginate::sealed::Sealed for ReleasesRequest<'_> {}
impl crate::paginate::Paginate for ReleasesRequest<'_> {
    type Page = ReleasesResults;
    const MAX_PAGE: u32 = 1000;
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
