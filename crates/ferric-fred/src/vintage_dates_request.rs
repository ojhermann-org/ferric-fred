use crate::{Client, Result, SeriesId, SortOrder, VintageDates};

/// A builder for a `series/vintagedates` request, returned by
/// [`Client::series_vintagedates`]. Lists the dates on which a series was
/// revised or newly released, with optional sort and paging. Finish with
/// [`send`](VintageDatesRequest::send).
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
#[derive(Debug, Clone)]
#[must_use = "a VintageDatesRequest does nothing until you call `.send()`"]
pub struct VintageDatesRequest<'a> {
    client: &'a Client,
    series_id: SeriesId,
    sort_order: Option<SortOrder>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl<'a> VintageDatesRequest<'a> {
    pub(crate) fn new(client: &'a Client, series_id: SeriesId) -> Self {
        Self {
            client,
            series_id,
            sort_order: None,
            limit: None,
            offset: None,
        }
    }

    /// Sort order of the dates (`sort_order`); oldest first by default.
    pub fn sort_order(mut self, order: SortOrder) -> Self {
        self.sort_order = Some(order);
        self
    }

    /// Maximum number of results to return (`limit`).
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Number of results to skip from the start (`offset`), for paging.
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Run the request and return the vintage dates with pagination metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn send(self) -> Result<VintageDates> {
        self.client.execute_vintage_dates(&self).await
    }

    /// Serialize the set parameters to FRED query key/value pairs. `api_key` and
    /// `file_type` are added by the client, not here.
    pub(crate) fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params: Vec<(&'static str, String)> =
            vec![("series_id", self.series_id.as_str().to_owned())];
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

impl crate::paginate::sealed::Sealed for VintageDatesRequest<'_> {}
impl crate::paginate::Paginate for VintageDatesRequest<'_> {
    type Page = VintageDates;
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
