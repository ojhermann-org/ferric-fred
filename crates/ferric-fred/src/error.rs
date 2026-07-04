use std::time::Duration;

/// Errors returned by the `ferric-fred` client.
///
/// The library never panics on a network or parse failure — those surface here
/// as `Err`. The enum is `#[non_exhaustive]` so new variants can be added
/// without a breaking change (see ADR-0004).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A connection, timeout, or TLS-level failure from the HTTP transport.
    #[error("HTTP transport error")]
    Transport(#[from] reqwest::Error),

    /// FRED returned an error payload. FRED encodes errors in the response body
    /// with a code and message alongside the HTTP status.
    #[error("FRED API error (HTTP {status}): {message}")]
    Api {
        /// HTTP status code of the response.
        status: u16,
        /// FRED's own error code, when present in the body.
        code: Option<u32>,
        /// Human-readable error message.
        message: String,
    },

    /// A response body did not match the expected shape.
    #[error("failed to deserialize FRED response")]
    Deserialize(#[from] serde_json::Error),

    /// Caller-side validation failed before any request was made.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// FRED rate-limited the request. Surfaced distinctly so callers can back
    /// off.
    #[error("rate limited by FRED")]
    RateLimited {
        /// Suggested delay before retrying, if FRED provided one.
        retry_after: Option<Duration>,
    },
}
