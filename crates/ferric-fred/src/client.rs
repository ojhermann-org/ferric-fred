use serde::Deserialize;

use crate::{Error, Observation, Result, SeriesId};

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

    /// Fetch every observation for a series.
    pub async fn observations(&self, series_id: &SeriesId) -> Result<Vec<Observation>> {
        let response = self
            .http
            .get(format!("{}/series/observations", self.base_url))
            .query(&[
                ("series_id", series_id.as_str()),
                ("api_key", self.api_key.as_str()),
                ("file_type", "json"),
            ])
            .send()
            .await?;

        let status = response.status();
        let body = response.bytes().await?;

        if !status.is_success() {
            return Err(api_error(status, &body));
        }

        let parsed: ObservationsResponse = serde_json::from_slice(&body)?;
        Ok(parsed.observations)
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

/// FRED's error response body.
#[derive(Deserialize)]
struct FredErrorBody {
    error_code: Option<u32>,
    error_message: Option<String>,
}
