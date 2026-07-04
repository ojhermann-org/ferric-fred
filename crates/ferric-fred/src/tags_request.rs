use crate::{Client, Result, SortOrder, TagsResults};

/// A builder for a `tags` request, returned by [`Client::tags`]. Browses or
/// searches FRED's tag vocabulary, with optional search text, sort, and paging.
/// Finish with [`send`](TagsRequest::send).
///
/// ```no_run
/// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
/// let results = client.tags().search_text("gdp").limit(10).send().await?;
/// for tag in &results.tags {
///     println!("{} ({} series)", tag.name, tag.series_count);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
#[must_use = "a TagsRequest does nothing until you call `.send()`"]
pub struct TagsRequest<'a> {
    client: &'a Client,
    search_text: Option<String>,
    sort_order: Option<SortOrder>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl<'a> TagsRequest<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            search_text: None,
            sort_order: None,
            limit: None,
            offset: None,
        }
    }

    /// Restrict to tags matching these words (`search_text`).
    pub fn search_text(mut self, text: impl Into<String>) -> Self {
        self.search_text = Some(text.into());
        self
    }

    /// Sort order of the results (`sort_order`); tags are ordered by series
    /// count by default.
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

    /// Run the request and return a page of tags with pagination metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn send(self) -> Result<TagsResults> {
        self.client.execute_tags(&self).await
    }

    /// Serialize the set parameters to FRED query key/value pairs. `api_key` and
    /// `file_type` are added by the client, not here.
    pub(crate) fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params: Vec<(&'static str, String)> = Vec::new();
        if let Some(text) = &self.search_text {
            params.push(("search_text", text.clone()));
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
