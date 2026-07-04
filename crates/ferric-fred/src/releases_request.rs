use crate::{Client, ReleasesResults, Result, SortOrder};

/// A builder for a `releases` request, returned by [`Client::releases`]. Lists
/// all FRED data releases, with optional sort and paging. Finish with
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
    sort_order: Option<SortOrder>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl<'a> ReleasesRequest<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            sort_order: None,
            limit: None,
            offset: None,
        }
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
