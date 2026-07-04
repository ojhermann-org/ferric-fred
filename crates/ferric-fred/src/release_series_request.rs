use crate::{Client, OrderBy, ReleaseId, Result, SeriesSearchResults, SortOrder};

/// A builder for a `release/series` request, returned by
/// [`Client::release_series`]. Lists the series belonging to a release, with
/// optional ordering and paging. Finish with
/// [`send`](ReleaseSeriesRequest::send).
///
/// ```no_run
/// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
/// use ferric_fred::{OrderBy, ReleaseId};
/// let results = client
///     .release_series(ReleaseId::new(53))
///     .order_by(OrderBy::Popularity)
///     .limit(10)
///     .send()
///     .await?;
/// println!("{} series in this release", results.count);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
#[must_use = "a ReleaseSeriesRequest does nothing until you call `.send()`"]
pub struct ReleaseSeriesRequest<'a> {
    client: &'a Client,
    release_id: ReleaseId,
    order_by: Option<OrderBy>,
    sort_order: Option<SortOrder>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl<'a> ReleaseSeriesRequest<'a> {
    pub(crate) fn new(client: &'a Client, release_id: ReleaseId) -> Self {
        Self {
            client,
            release_id,
            order_by: None,
            sort_order: None,
            limit: None,
            offset: None,
        }
    }

    /// Field to order results by (`order_by`).
    pub fn order_by(mut self, order_by: OrderBy) -> Self {
        self.order_by = Some(order_by);
        self
    }

    /// Sort order of the results (`sort_order`).
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

    /// Run the request and return the series in the release with pagination
    /// metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn send(self) -> Result<SeriesSearchResults> {
        self.client.execute_release_series(&self).await
    }

    /// Serialize the set parameters to FRED query key/value pairs. `api_key` and
    /// `file_type` are added by the client, not here.
    pub(crate) fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params: Vec<(&'static str, String)> =
            vec![("release_id", self.release_id.get().to_string())];
        if let Some(order_by) = self.order_by {
            params.push(("order_by", order_by.query_code().to_owned()));
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
