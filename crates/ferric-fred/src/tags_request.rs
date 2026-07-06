use crate::{Client, Result, SortOrder, TagsResults};

/// A builder for the tag-listing endpoints — the whole-vocabulary pair
/// [`Client::tags`] (`fred/tags`) / [`Client::related_tags`]
/// (`fred/related_tags`), and the *scoped* variants that list the tags of one
/// category, release, or series search: [`Client::category_tags`],
/// [`Client::category_related_tags`], [`Client::release_tags`],
/// [`Client::release_related_tags`], [`Client::series_search_tags`], and
/// [`Client::series_search_related_tags`].
///
/// All share optional tag-filter text, sort, and paging and return
/// [`TagsResults`]. Each `*_related_tags` variant additionally carries a seed
/// `tag_names` set (the tags to find co-occurring tags for); each scoped
/// variant carries its scope facet (`category_id` / `release_id` /
/// `series_search_text`). Finish with [`send`](TagsRequest::send).
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
    /// The endpoint path, e.g. `/tags`, `/related_tags`, `/category/tags`.
    path: &'static str,
    /// The scope facet for a scoped endpoint: `("category_id", "125")`,
    /// `("release_id", "53")`, or `("series_search_text", "...")`. Absent for
    /// the whole-vocabulary `/tags` and `/related_tags`.
    scope: Option<(&'static str, String)>,
    /// FRED's `tag_names` (seed tags joined with `;`); required by the
    /// `*related_tags` endpoints, absent for the plain `*tags` endpoints.
    tag_names: Option<String>,
    search_text: Option<String>,
    /// The query key the tag-filter text is sent under: `search_text`
    /// everywhere except the `series/search/*` endpoints, which use
    /// `tag_search_text` (their `series_search_text` is the scope facet).
    search_text_key: &'static str,
    sort_order: Option<SortOrder>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl<'a> TagsRequest<'a> {
    pub(crate) fn new(client: &'a Client, path: &'static str) -> Self {
        Self {
            client,
            path,
            scope: None,
            tag_names: None,
            search_text: None,
            search_text_key: "search_text",
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

    /// Construct a scoped tags request: `<scope>/tags` or
    /// `<scope>/related_tags`. `scope` is the facet key/value (e.g.
    /// `("category_id", "125")`); `tag_names` is the seed set (present only for
    /// the `*related_tags` variants); `search_text_key` is the query key the
    /// tag-filter text is sent under.
    pub(crate) fn scoped(
        client: &'a Client,
        path: &'static str,
        scope: (&'static str, String),
        tag_names: Option<String>,
        search_text_key: &'static str,
    ) -> Self {
        Self {
            scope: Some(scope),
            tag_names,
            search_text_key,
            ..Self::new(client, path)
        }
    }

    /// The endpoint path this request targets (used by the client to dispatch).
    pub(crate) fn path(&self) -> &'static str {
        self.path
    }

    /// Restrict to tags matching these words (`search_text`, or
    /// `tag_search_text` for the `series/search/*` endpoints).
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
        if let Some((key, value)) = &self.scope {
            params.push((key, value.clone()));
        }
        if let Some(tag_names) = &self.tag_names {
            params.push(("tag_names", tag_names.clone()));
        }
        if let Some(text) = &self.search_text {
            params.push((self.search_text_key, text.clone()));
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

impl crate::paginate::sealed::Sealed for TagsRequest<'_> {}
impl crate::paginate::Paginate for TagsRequest<'_> {
    type Page = TagsResults;
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
