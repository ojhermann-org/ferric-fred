use crate::{Client, OrderBy, Result, SeriesSearchResults, SortOrder};

/// A builder for the FRED endpoints that return a page of series filtered by a
/// single facet: `category/series`, `release/series`, and `tags/series`. They
/// share the same optional ordering and paging and all return
/// [`SeriesSearchResults`]; they differ only in the endpoint path and the one
/// facet parameter (`category_id` / `release_id` / `tag_names`).
///
/// Construct one via [`Client::category_series`], [`Client::release_series`], or
/// [`Client::tags_series`]; finish with [`send`](SeriesListRequest::send).
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
/// println!("{} series", results.count);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
#[must_use = "a SeriesListRequest does nothing until you call `.send()`"]
pub struct SeriesListRequest<'a> {
    client: &'a Client,
    /// The endpoint path, e.g. `/category/series`.
    path: &'static str,
    /// The facet filter as a `(key, value)` query pair, e.g.
    /// `("category_id", "125")` or `("tag_names", "gdp;quarterly")`.
    facet: (&'static str, String),
    order_by: Option<OrderBy>,
    sort_order: Option<SortOrder>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl<'a> SeriesListRequest<'a> {
    pub(crate) fn new(
        client: &'a Client,
        path: &'static str,
        facet_key: &'static str,
        facet_value: String,
    ) -> Self {
        Self {
            client,
            path,
            facet: (facet_key, facet_value),
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

    /// Run the request and return the matching series with pagination metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn send(self) -> Result<SeriesSearchResults> {
        self.client.execute_series_list(&self).await
    }

    /// The endpoint path this request targets (used by the client to dispatch).
    pub(crate) fn path(&self) -> &'static str {
        self.path
    }

    /// Serialize the facet plus set options to FRED query key/value pairs.
    /// `api_key` and `file_type` are added by the client, not here.
    pub(crate) fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params: Vec<(&'static str, String)> = vec![(self.facet.0, self.facet.1.clone())];
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

impl crate::paginate::sealed::Sealed for SeriesListRequest<'_> {}
impl crate::paginate::Paginate for SeriesListRequest<'_> {
    type Page = SeriesSearchResults;
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
