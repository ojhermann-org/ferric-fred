use chrono::NaiveDate;

use crate::{
    AggregationMethod, Client, Frequency, Observation, Result, SeriesId, SortOrder, Units,
};

/// A builder for an observations request, returned by [`Client::observations`].
///
/// Only parameters you set are sent; anything left unset uses FRED's default
/// (full history, `Levels` units, ascending by date, the series' native
/// frequency). Finish with [`send`](ObservationsRequest::send).
///
/// ```no_run
/// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
/// use ferric_fred::{SeriesId, SortOrder, Units};
/// let latest = client
///     .observations(&SeriesId::new("UNRATE"))
///     .units(Units::PercentChange)
///     .sort_order(SortOrder::Descending)
///     .limit(12)
///     .send()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
#[must_use = "an ObservationsRequest does nothing until you call `.send()`"]
pub struct ObservationsRequest<'a> {
    client: &'a Client,
    series_id: SeriesId,
    observation_start: Option<NaiveDate>,
    observation_end: Option<NaiveDate>,
    units: Option<Units>,
    frequency: Option<Frequency>,
    aggregation_method: Option<AggregationMethod>,
    sort_order: Option<SortOrder>,
    limit: Option<u32>,
    offset: Option<u32>,
    /// FRED's `realtime_start`/`realtime_end` — a real-time period (ALFRED,
    /// ADR-0024), stored as a pair since it reads as one window.
    realtime: Option<(NaiveDate, NaiveDate)>,
    vintage_dates: Vec<NaiveDate>,
}

impl<'a> ObservationsRequest<'a> {
    pub(crate) fn new(client: &'a Client, series_id: SeriesId) -> Self {
        Self {
            client,
            series_id,
            observation_start: None,
            observation_end: None,
            units: None,
            frequency: None,
            aggregation_method: None,
            sort_order: None,
            limit: None,
            offset: None,
            realtime: None,
            vintage_dates: Vec::new(),
        }
    }

    /// Earliest observation date to return (`observation_start`).
    pub fn observation_start(mut self, date: NaiveDate) -> Self {
        self.observation_start = Some(date);
        self
    }

    /// Latest observation date to return (`observation_end`).
    pub fn observation_end(mut self, date: NaiveDate) -> Self {
        self.observation_end = Some(date);
        self
    }

    /// Convenience for setting both ends of the date range at once.
    pub fn date_range(self, start: NaiveDate, end: NaiveDate) -> Self {
        self.observation_start(start).observation_end(end)
    }

    /// Units transformation to apply (`units`).
    pub fn units(mut self, units: Units) -> Self {
        self.units = Some(units);
        self
    }

    /// Aggregate observations down to a lower `frequency`. Pair with
    /// [`aggregation_method`](Self::aggregation_method) to control how.
    pub fn frequency(mut self, frequency: Frequency) -> Self {
        self.frequency = Some(frequency);
        self
    }

    /// How to aggregate when a lower [`frequency`](Self::frequency) is set
    /// (`aggregation_method`).
    pub fn aggregation_method(mut self, method: AggregationMethod) -> Self {
        self.aggregation_method = Some(method);
        self
    }

    /// Sort order of the returned observations by date (`sort_order`).
    pub fn sort_order(mut self, order: SortOrder) -> Self {
        self.sort_order = Some(order);
        self
    }

    /// Maximum number of observations to return, `1..=100000` (`limit`).
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Number of observations to skip from the start (`offset`), for paging.
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Return the data **as it was known** over this real-time period
    /// (`realtime_start`/`realtime_end`) — the ALFRED / point-in-time dimension
    /// (ADR-0024). Pass the **same date twice** for a point-in-time snapshot (the
    /// series as it looked on that day); pass a window to see each value's
    /// real-time period. Each returned [`Observation`] carries its own
    /// [`realtime_start`](Observation::realtime_start)/[`realtime_end`](Observation::realtime_end).
    pub fn realtime(mut self, start: NaiveDate, end: NaiveDate) -> Self {
        self.realtime = Some((start, end));
        self
    }

    /// Return the data as of these specific revision dates (`vintage_dates`) —
    /// each date selects that vintage of the series (ALFRED, ADR-0024).
    /// Complements [`realtime`](Self::realtime); an empty list is ignored.
    pub fn vintage_dates(mut self, dates: impl IntoIterator<Item = NaiveDate>) -> Self {
        self.vintage_dates = dates.into_iter().collect();
        self
    }

    /// Run the request and return the matching observations.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send, FRED returns a non-success
    /// status, or the response body cannot be deserialized.
    pub async fn send(self) -> Result<Vec<Observation>> {
        self.client.execute_observations(&self).await
    }

    /// Serialize the set parameters to FRED query key/value pairs. `api_key` and
    /// `file_type` are added by the client, not here.
    pub(crate) fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params: Vec<(&'static str, String)> = Vec::new();
        params.push(("series_id", self.series_id.as_str().to_owned()));
        if let Some(date) = self.observation_start {
            params.push(("observation_start", date.to_string()));
        }
        if let Some(date) = self.observation_end {
            params.push(("observation_end", date.to_string()));
        }
        if let Some(units) = self.units {
            params.push(("units", units.query_code().to_owned()));
        }
        if let Some(frequency) = &self.frequency {
            params.push(("frequency", frequency.query_code().to_owned()));
        }
        if let Some(method) = self.aggregation_method {
            params.push(("aggregation_method", method.query_code().to_owned()));
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
        if let Some((start, end)) = self.realtime {
            params.push(("realtime_start", start.to_string()));
            params.push(("realtime_end", end.to_string()));
        }
        if !self.vintage_dates.is_empty() {
            let joined = self
                .vintage_dates
                .iter()
                .map(NaiveDate::to_string)
                .collect::<Vec<_>>()
                .join(",");
            params.push(("vintage_dates", joined));
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
        let request = client.observations(&SeriesId::new("GNPCA"));
        assert_eq!(
            request.query_params(),
            vec![("series_id", "GNPCA".to_owned())]
        );
    }

    #[test]
    fn parameters_serialize_to_fred_codes() {
        let client = test_client();
        let request = client
            .observations(&SeriesId::new("GNPCA"))
            .date_range(
                NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2010, 12, 31).unwrap(),
            )
            .units(Units::PercentChange)
            .frequency(Frequency::Quarterly)
            .aggregation_method(AggregationMethod::Sum)
            .sort_order(SortOrder::Descending)
            .limit(50)
            .offset(5);

        let params = request.query_params();
        for expected in [
            ("series_id", "GNPCA"),
            ("observation_start", "2000-01-01"),
            ("observation_end", "2010-12-31"),
            ("units", "pch"),
            ("frequency", "q"),
            ("aggregation_method", "sum"),
            ("sort_order", "desc"),
            ("limit", "50"),
            ("offset", "5"),
        ] {
            assert!(
                params.contains(&(expected.0, expected.1.to_owned())),
                "missing {expected:?} in {params:?}"
            );
        }
    }

    #[test]
    fn realtime_and_vintage_dates_serialize() {
        let client = test_client();
        let request = client
            .observations(&SeriesId::new("GNPCA"))
            .realtime(
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            )
            .vintage_dates([
                NaiveDate::from_ymd_opt(2020, 3, 26).unwrap(),
                NaiveDate::from_ymd_opt(2021, 3, 25).unwrap(),
            ]);

        let params = request.query_params();
        for expected in [
            ("realtime_start", "2020-01-01"),
            ("realtime_end", "2020-01-01"),
            ("vintage_dates", "2020-03-26,2021-03-25"),
        ] {
            assert!(
                params.contains(&(expected.0, expected.1.to_owned())),
                "missing {expected:?} in {params:?}"
            );
        }
    }
}
