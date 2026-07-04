use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::{Error, Observation, ObservationsRequest, Result, Series, SeriesId};

/// Base URL for the FRED REST API.
const FRED_BASE_URL: &str = "https://api.stlouisfed.org/fred";

/// An async client for the FRED API.
///
/// Cheap to clone — the underlying `reqwest::Client` holds a connection pool
/// behind an `Arc`, so clones share it.
#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl Client {
    /// Build a client with the given FRED API key.
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        let http = reqwest::Client::builder().build()?;
        Ok(Self {
            http,
            api_key: api_key.into(),
            base_url: FRED_BASE_URL.to_owned(),
        })
    }

    /// Build a client, reading the API key from the `FRED_API_KEY` environment
    /// variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("FRED_API_KEY").map_err(|_| {
            Error::InvalidInput("FRED_API_KEY environment variable is not set".to_owned())
        })?;
        Self::new(api_key)
    }

    /// Begin an observations request for a series.
    ///
    /// Returns a builder; set optional parameters (date range, units transform,
    /// frequency aggregation, sort order, paging) and call
    /// [`ObservationsRequest::send`] to run it. With nothing set, FRED's
    /// defaults apply (full history, levels, ascending by date).
    ///
    /// ```no_run
    /// # async fn run(client: &ferric_fred::Client) -> ferric_fred::Result<()> {
    /// use ferric_fred::{SeriesId, Units};
    /// let obs = client
    ///     .observations(&SeriesId::new("GNPCA"))
    ///     .units(Units::PercentChange)
    ///     .limit(10)
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn observations(&self, series_id: &SeriesId) -> ObservationsRequest<'_> {
        ObservationsRequest::new(self, series_id.clone())
    }

    /// Run an observations request (invoked by [`ObservationsRequest::send`]).
    pub(crate) async fn execute_observations(
        &self,
        request: &ObservationsRequest<'_>,
    ) -> Result<Vec<Observation>> {
        let response: ObservationsResponse = self
            .get("/series/observations", &request.query_params())
            .await?;
        Ok(response.observations)
    }

    /// Fetch metadata for a series (the `fred/series` endpoint).
    pub async fn series(&self, series_id: &SeriesId) -> Result<Series> {
        let response: SeriesResponse = self
            .get("/series", &[("series_id", series_id.as_str().to_owned())])
            .await?;
        response
            .seriess
            .into_iter()
            .next()
            .ok_or_else(|| Error::Api {
                status: 200,
                code: None,
                message: format!("FRED returned no series for id `{series_id}`"),
            })
    }

    /// GET `path` with `params` plus `api_key`/`file_type`, then deserialize the
    /// JSON body as `T`. A non-success status becomes [`Error::Api`] (or
    /// [`Error::RateLimited`]); a body that doesn't match `T` becomes
    /// [`Error::Deserialize`].
    async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&'static str, String)],
    ) -> Result<T> {
        let mut query: Vec<(&str, String)> = Vec::with_capacity(params.len() + 2);
        query.push(("api_key", self.api_key.clone()));
        query.push(("file_type", "json".to_owned()));
        query.extend(params.iter().cloned());

        let response = self
            .http
            .get(format!("{}{}", self.base_url, path))
            .query(&query)
            .send()
            .await?;

        let status = response.status();
        let body = response.bytes().await?;

        if !status.is_success() {
            return Err(api_error(status, &body));
        }

        serde_json::from_slice(&body).map_err(Error::from)
    }
}

/// Build an [`Error`] from a non-success FRED response, decoding FRED's error
/// body (`{"error_code": N, "error_message": "..."}`) when present.
fn api_error(status: reqwest::StatusCode, body: &[u8]) -> Error {
    let fred: Option<FredErrorBody> = serde_json::from_slice(body).ok();
    let code = fred.as_ref().and_then(|e| e.error_code);
    let message = fred.and_then(|e| e.error_message).unwrap_or_else(|| {
        status
            .canonical_reason()
            .unwrap_or("unknown error")
            .to_owned()
    });

    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Error::RateLimited { retry_after: None };
    }

    Error::Api {
        status: status.as_u16(),
        code,
        message,
    }
}

/// The `series/observations` response envelope. Metadata fields (realtime range,
/// units, paging) are ignored for this slice; serde drops unknown fields.
#[derive(Deserialize)]
struct ObservationsResponse {
    observations: Vec<Observation>,
}

/// The `series` response envelope. FRED pluralizes the array key as `seriess`
/// (sic); other metadata fields are ignored for this slice.
#[derive(Deserialize)]
struct SeriesResponse {
    seriess: Vec<Series>,
}

/// FRED's error response body.
#[derive(Deserialize)]
struct FredErrorBody {
    error_code: Option<u32>,
    error_message: Option<String>,
}
