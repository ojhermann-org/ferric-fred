use crate::{Client, Result, SeriesSearchResults, UpdatesFilter};

/// A builder for a `series/updates` request, returned by
/// [`Client::series_updates`]. Lists the series updated most recently (ordered
/// by last-updated time), optionally narrowed to a class of series. Finish with
/// [`send`](SeriesUpdatesRequest::send).
///
/// ```no_run
/// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
/// use ferric_fred::UpdatesFilter;
/// let results = client
///     .series_updates()
///     .filter(UpdatesFilter::Macro)
///     .limit(20)
///     .send()
///     .await?;
/// println!("{} series recently updated", results.count);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
#[must_use = "a SeriesUpdatesRequest does nothing until you call `.send()`"]
pub struct SeriesUpdatesRequest<'a> {
    client: &'a Client,
    filter: Option<UpdatesFilter>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl<'a> SeriesUpdatesRequest<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            filter: None,
            limit: None,
            offset: None,
        }
    }

    /// Narrow the results to a class of series (`filter_value`); defaults to all.
    pub fn filter(mut self, filter: UpdatesFilter) -> Self {
        self.filter = Some(filter);
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

    /// Run the request and return the recently-updated series with pagination
    /// metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn send(self) -> Result<SeriesSearchResults> {
        self.client.execute_series_updates(&self).await
    }

    /// Serialize the set parameters to FRED query key/value pairs. `api_key` and
    /// `file_type` are added by the client, not here.
    pub(crate) fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params: Vec<(&'static str, String)> = Vec::new();
        if let Some(filter) = self.filter {
            params.push(("filter_value", filter.query_code().to_owned()));
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
