use crate::{Client, OrderBy, Result, SearchType, SeriesSearchResults, SortOrder};

/// A builder for a `series/search` request, returned by [`Client::search`].
///
/// Only parameters you set are sent; anything left unset uses FRED's default.
/// Finish with [`send`](SeriesSearchRequest::send).
///
/// ```no_run
/// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
/// use ferric_fred::{OrderBy, SortOrder};
/// let results = client
///     .search("unemployment rate")
///     .order_by(OrderBy::Popularity)
///     .sort_order(SortOrder::Descending)
///     .limit(10)
///     .send()
///     .await?;
/// println!("{} total matches", results.count);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct SeriesSearchRequest<'a> {
    client: &'a Client,
    search_text: String,
    search_type: Option<SearchType>,
    order_by: Option<OrderBy>,
    sort_order: Option<SortOrder>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl<'a> SeriesSearchRequest<'a> {
    pub(crate) fn new(client: &'a Client, search_text: String) -> Self {
        Self {
            client,
            search_text,
            search_type: None,
            order_by: None,
            sort_order: None,
            limit: None,
            offset: None,
        }
    }

    /// How the search text is interpreted (`search_type`).
    pub fn search_type(mut self, search_type: SearchType) -> Self {
        self.search_type = Some(search_type);
        self
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

    /// Run the search and return the matching series with pagination metadata.
    pub async fn send(self) -> Result<SeriesSearchResults> {
        self.client.execute_search(&self).await
    }

    /// Serialize the set parameters to FRED query key/value pairs. `api_key` and
    /// `file_type` are added by the client, not here.
    pub(crate) fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params: Vec<(&'static str, String)> = Vec::new();
        params.push(("search_text", self.search_text.clone()));
        if let Some(search_type) = self.search_type {
            params.push(("search_type", search_type.query_code().to_owned()));
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_client() -> Client {
        Client::new("test-key").expect("client builds")
    }

    #[test]
    fn defaults_send_only_the_search_text() {
        let client = test_client();
        let request = client.search("real gnp");
        assert_eq!(
            request.query_params(),
            vec![("search_text", "real gnp".to_owned())]
        );
    }

    #[test]
    fn parameters_serialize_to_fred_codes() {
        let client = test_client();
        let request = client
            .search("unemployment")
            .search_type(SearchType::FullText)
            .order_by(OrderBy::Popularity)
            .sort_order(SortOrder::Descending)
            .limit(25)
            .offset(50);

        let params = request.query_params();
        for expected in [
            ("search_text", "unemployment"),
            ("search_type", "full_text"),
            ("order_by", "popularity"),
            ("sort_order", "desc"),
            ("limit", "25"),
            ("offset", "50"),
        ] {
            assert!(
                params.contains(&(expected.0, expected.1.to_owned())),
                "missing {expected:?} in {params:?}"
            );
        }
    }
}
