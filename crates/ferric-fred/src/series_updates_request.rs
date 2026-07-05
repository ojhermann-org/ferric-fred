use crate::{Client, Result, SeriesSearchResults, UpdatesFilter};
use chrono::NaiveDateTime;

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
    /// FRED's `start_time`/`end_time` window — a required pair (ADR-0019), so we
    /// hold them together to make "one set, the other missing" unrepresentable.
    time_window: Option<(NaiveDateTime, NaiveDateTime)>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl<'a> SeriesUpdatesRequest<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            filter: None,
            time_window: None,
            limit: None,
            offset: None,
        }
    }

    /// Narrow the results to a class of series (`filter_value`); defaults to all.
    pub fn filter(mut self, filter: UpdatesFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Limit results to series updated within a time window (`start_time` /
    /// `end_time`), down to the minute. FRED requires these as a pair, so this
    /// method takes both bounds at once (ADR-0019).
    ///
    /// The times are naive wall-clock in FRED's own timezone — they are sent as
    /// given (formatted `%Y%m%d%H%M`), with no timezone conversion and
    /// minute granularity.
    pub fn time_window(mut self, start: NaiveDateTime, end: NaiveDateTime) -> Self {
        self.time_window = Some((start, end));
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
        if let Some((start, end)) = self.time_window {
            params.push(("start_time", start.format("%Y%m%d%H%M").to_string()));
            params.push(("end_time", end.format("%Y%m%d%H%M").to_string()));
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
