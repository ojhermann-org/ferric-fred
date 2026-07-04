use crate::{Client, Result, SortOrder, TagsResults};

/// A builder for the tag-listing endpoints, returned by [`Client::tags`]
/// (`fred/tags`, browse/search the whole vocabulary) and
/// [`Client::related_tags`] (`fred/related_tags`, tags co-occurring with a seed
/// set). Both share optional search text, sort, and paging and return
/// [`TagsResults`]; `related_tags` additionally carries the seed `tag_names`.
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
    /// The endpoint path, `/tags` or `/related_tags`.
    path: &'static str,
    /// FRED's `tag_names` (seed tags joined with `;`); required by
    /// `/related_tags`, absent for `/tags`.
    tag_names: Option<String>,
    search_text: Option<String>,
    sort_order: Option<SortOrder>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl<'a> TagsRequest<'a> {
    pub(crate) fn new(client: &'a Client, path: &'static str) -> Self {
        Self {
            client,
            path,
            tag_names: None,
            search_text: None,
            sort_order: None,
            limit: None,
            offset: None,
        }
    }

    /// Construct a request for `/related_tags` with the seed `tag_names`.
    pub(crate) fn with_tag_names(
        client: &'a Client,
        path: &'static str,
        tag_names: String,
    ) -> Self {
        Self {
            tag_names: Some(tag_names),
            ..Self::new(client, path)
        }
    }

    /// The endpoint path this request targets (used by the client to dispatch).
    pub(crate) fn path(&self) -> &'static str {
        self.path
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
        if let Some(tag_names) = &self.tag_names {
            params.push(("tag_names", tag_names.clone()));
        }
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
