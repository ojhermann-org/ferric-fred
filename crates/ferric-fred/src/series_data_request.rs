use chrono::NaiveDate;

use crate::{Client, RegionalData, Result, SeriesId};

/// A builder for a GeoFRED / Maps `series/data` request, returned by
/// [`Client::series_data`] — one regional series' values across regions, over
/// time.
///
/// Only `series_id` is required; with no date set FRED returns the most recent
/// date. Set an optional single [`date`](SeriesDataRequest::date) or a
/// [`start_date`](SeriesDataRequest::start_date) (which returns every date from
/// then on), and finish with [`send`](SeriesDataRequest::send).
///
/// ```no_run
/// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
/// use ferric_fred::SeriesId;
/// let data = client
///     .series_data(&SeriesId::new("SMU56000000500000001"))
///     .send()
///     .await?;
/// println!("{} dates", data.meta.data.len());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
#[must_use = "a SeriesDataRequest does nothing until you call `.send()`"]
pub struct SeriesDataRequest<'a> {
    client: &'a Client,
    series_id: SeriesId,
    date: Option<NaiveDate>,
    start_date: Option<NaiveDate>,
}

impl<'a> SeriesDataRequest<'a> {
    pub(crate) fn new(client: &'a Client, series_id: SeriesId) -> Self {
        Self {
            client,
            series_id,
            date: None,
            start_date: None,
        }
    }

    /// Return data for a single date (`date`). With neither this nor
    /// [`start_date`](Self::start_date) set, FRED returns the most recent date.
    pub fn date(mut self, date: NaiveDate) -> Self {
        self.date = Some(date);
        self
    }

    /// Return data for every date from `start_date` onward.
    pub fn start_date(mut self, start_date: NaiveDate) -> Self {
        self.start_date = Some(start_date);
        self
    }

    /// Run the request and return the regional data.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized. An unknown or
    /// non-regional series id surfaces as a clear [`Error::Api`](crate::Error::Api)
    /// naming the id — FRED answers that case with a bare HTTP 500, which the
    /// client rewrites into an actionable message.
    pub async fn send(self) -> Result<RegionalData> {
        self.client.execute_series_data(&self).await
    }

    /// Serialize the set parameters to FRED query key/value pairs. `api_key` and
    /// `file_type` are added by the client, not here.
    pub(crate) fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params: Vec<(&'static str, String)> = Vec::new();
        params.push(("series_id", self.series_id.as_str().to_owned()));
        if let Some(date) = self.date {
            params.push(("date", date.to_string()));
        }
        if let Some(start_date) = self.start_date {
            params.push(("start_date", start_date.to_string()));
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
    fn defaults_send_only_the_series_id() {
        let client = test_client();
        let request = client.series_data(&SeriesId::new("SMU56000000500000001"));
        assert_eq!(
            request.query_params(),
            vec![("series_id", "SMU56000000500000001".to_owned())]
        );
    }

    #[test]
    fn date_and_start_date_serialize() {
        let client = test_client();
        let request = client
            .series_data(&SeriesId::new("SMU56000000500000001"))
            .date(NaiveDate::from_ymd_opt(2013, 1, 1).unwrap())
            .start_date(NaiveDate::from_ymd_opt(2010, 1, 1).unwrap());
        let params = request.query_params();
        for expected in [
            ("series_id", "SMU56000000500000001"),
            ("date", "2013-01-01"),
            ("start_date", "2010-01-01"),
        ] {
            assert!(
                params.contains(&(expected.0, expected.1.to_owned())),
                "missing {expected:?} in {params:?}"
            );
        }
    }
}
