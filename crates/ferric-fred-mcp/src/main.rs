//! `fred-mcp` — an MCP server exposing FRED, built on the `ferric-fred` client.
//!
//! Speaks MCP over stdio ([ADR-0010]) using the `rmcp` SDK. Tools map onto the
//! library's endpoints and return the domain types as JSON. The API key comes
//! from `FRED_API_KEY` via `Client::from_env` (ADR-0009).
//!
//! This is the first slice: a single `get_series` tool, proving the SDK and the
//! stdio handshake before the remaining tools are added.
//!
//! Note: over stdio, **stdout is the protocol channel** — nothing may be printed
//! to it. Any diagnostics must go to stderr.

use anyhow::Context;
use ferric_fred::{Client, SeriesId};
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerInfo};
use rmcp::transport::stdio;
use rmcp::{tool, tool_handler, tool_router, ErrorData, ServerHandler, ServiceExt};
use schemars::JsonSchema;
use serde::Deserialize;

/// Input parameters for the `get_series` tool.
#[derive(Debug, Deserialize, JsonSchema)]
struct GetSeriesParams {
    /// The FRED series id, e.g. `GNPCA` or `UNRATE`.
    series_id: String,
}

/// The MCP server state: the FRED client plus the macro-generated tool router.
#[derive(Clone)]
struct FredServer {
    client: Client,
    tool_router: ToolRouter<FredServer>,
}

#[tool_router]
impl FredServer {
    fn new(client: Client) -> Self {
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        name = "get_series",
        description = "Fetch metadata for a FRED series by its id (e.g. GNPCA, UNRATE): title, \
                       frequency, seasonal adjustment, units, observation date range, and popularity."
    )]
    async fn get_series(
        &self,
        Parameters(params): Parameters<GetSeriesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client.series(&SeriesId::new(params.series_id)).await {
            Ok(series) => {
                let value = serde_json::to_value(&series)
                    .map_err(|error| ErrorData::internal_error(error.to_string(), None))?;
                Ok(CallToolResult::structured(value))
            }
            // A FRED-side failure (unknown id, transport error, …) is a
            // tool-level error: surface the message to the caller rather than an
            // opaque protocol error (see rmcp's CallToolResult::error docs).
            Err(error) => Ok(CallToolResult::error(vec![ContentBlock::text(
                error.to_string(),
            )])),
        }
    }
}

// Route tool calls through the cached router built once in `new()`, rather than
// the macro default of reconstructing `Self::tool_router()` on every call.
#[tool_handler(router = self.tool_router)]
impl ServerHandler for FredServer {
    fn get_info(&self) -> ServerInfo {
        // ServerInfo is #[non_exhaustive]; start from Default and set fields.
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        // env! resolves in *this* crate (rmcp's from_build_env() would report
        // rmcp's own name/version instead).
        info.server_info = Implementation::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        info.instructions = Some(
            "Query FRED (Federal Reserve Economic Data). Use get_series to look up a \
             series' metadata by its id (e.g. GNPCA, UNRATE)."
                .to_string(),
        );
        info
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::from_env()
        .context("could not initialize the FRED client (is FRED_API_KEY set?)")?;

    let service = FredServer::new(client)
        .serve(stdio())
        .await
        .context("failed to start the MCP server over stdio")?;

    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_info_advertises_tools_and_identity() {
        let server = FredServer::new(Client::new("test-key").expect("client builds"));
        let info = server.get_info();
        assert!(
            info.capabilities.tools.is_some(),
            "the server must advertise the tools capability"
        );
        assert_eq!(info.server_info.name, "ferric-fred-mcp");
    }

    #[test]
    fn get_series_params_deserialize_from_arguments() {
        let params: GetSeriesParams =
            serde_json::from_value(serde_json::json!({"series_id": "GNPCA"})).unwrap();
        assert_eq!(params.series_id, "GNPCA");
    }
}
