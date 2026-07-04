//! `fred-mcp` — an MCP server exposing FRED, built on the `ferric-fred` client.
//!
//! Speaks MCP over stdio ([ADR-0010]) using the `rmcp` SDK. Tools map onto the
//! library's endpoints and return the domain types as JSON. The API key comes
//! from `FRED_API_KEY` via `Client::from_env` (ADR-0009).
//!
//! Tools: `search_series`, `get_series`, `get_observations`, the category tools
//! (`get_category`, `get_category_children`, `get_category_series`), and the
//! release tools (`get_releases`, `get_release`, `get_release_series`) — one per
//! library endpoint, with typed inputs (see [`params`]).
//!
//! Note: over stdio, **stdout is the protocol channel** — nothing may be printed
//! to it. Any diagnostics must go to stderr.

mod params;

use anyhow::Context;
use chrono::NaiveDate;
use ferric_fred::{CategoryId, Client, ReleaseId, SeriesId};
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerInfo};
use rmcp::transport::stdio;
use rmcp::{tool, tool_handler, tool_router, ErrorData, ServerHandler, ServiceExt};
use schemars::JsonSchema;
use serde::Deserialize;

use params::{AggregationArg, FrequencyArg, OrderByArg, SortOrderArg, UnitsArg};

/// Input parameters for the `get_series` tool.
#[derive(Debug, Deserialize, JsonSchema)]
struct GetSeriesParams {
    /// The FRED series id, e.g. `GNPCA` or `UNRATE`.
    series_id: String,
}

/// Input parameters for the `search_series` tool.
#[derive(Debug, Deserialize, JsonSchema)]
struct SearchSeriesParams {
    /// Words to search for, e.g. "unemployment rate".
    text: String,
    /// Maximum number of results to return.
    limit: Option<u32>,
    /// Field to order results by (default: search relevance).
    order_by: Option<OrderByArg>,
    /// Sort direction.
    sort: Option<SortOrderArg>,
}

/// Input parameters for the `get_observations` tool.
#[derive(Debug, Deserialize, JsonSchema)]
struct GetObservationsParams {
    /// The FRED series id, e.g. `GNPCA` or `UNRATE`.
    series_id: String,
    /// Earliest observation date, `YYYY-MM-DD`.
    start: Option<String>,
    /// Latest observation date, `YYYY-MM-DD`.
    end: Option<String>,
    /// Maximum number of observations to return.
    limit: Option<u32>,
    /// Units transformation to apply.
    units: Option<UnitsArg>,
    /// Frequency to aggregate observations down to.
    frequency: Option<FrequencyArg>,
    /// Aggregation method, used together with `frequency`.
    aggregation: Option<AggregationArg>,
    /// Sort order by date.
    sort: Option<SortOrderArg>,
}

/// Input parameters for the `get_category` and `get_category_children` tools.
#[derive(Debug, Deserialize, JsonSchema)]
struct CategoryParams {
    /// The FRED category id (0 is the root of the category tree).
    category_id: u32,
}

/// Input parameters for the `get_category_series` tool.
#[derive(Debug, Deserialize, JsonSchema)]
struct CategorySeriesParams {
    /// The FRED category id (0 is the root of the category tree).
    category_id: u32,
    /// Maximum number of series to return.
    limit: Option<u32>,
    /// Field to order results by.
    order_by: Option<OrderByArg>,
    /// Sort direction.
    sort: Option<SortOrderArg>,
}

/// Input parameters for the `get_releases` tool.
#[derive(Debug, Deserialize, JsonSchema)]
struct GetReleasesParams {
    /// Maximum number of releases to return.
    limit: Option<u32>,
    /// Sort direction by release id.
    sort: Option<SortOrderArg>,
}

/// Input parameters for the `get_release` tool.
#[derive(Debug, Deserialize, JsonSchema)]
struct GetReleaseParams {
    /// The FRED release id, e.g. 53 (Gross Domestic Product).
    release_id: u32,
}

/// Input parameters for the `get_release_series` tool.
#[derive(Debug, Deserialize, JsonSchema)]
struct ReleaseSeriesParams {
    /// The FRED release id.
    release_id: u32,
    /// Maximum number of series to return.
    limit: Option<u32>,
    /// Field to order results by.
    order_by: Option<OrderByArg>,
    /// Sort direction.
    sort: Option<SortOrderArg>,
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

    #[tool(
        name = "search_series",
        description = "Search FRED for series matching text. Returns the matching series along \
                       with pagination metadata (total match count, offset, limit)."
    )]
    async fn search_series(
        &self,
        Parameters(params): Parameters<SearchSeriesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut request = self.client.search(params.text);
        if let Some(limit) = params.limit {
            request = request.limit(limit);
        }
        if let Some(order_by) = params.order_by {
            request = request.order_by(order_by.into());
        }
        if let Some(sort) = params.sort {
            request = request.sort_order(sort.into());
        }

        match request.send().await {
            Ok(results) => {
                let value = serde_json::to_value(&results)
                    .map_err(|error| ErrorData::internal_error(error.to_string(), None))?;
                Ok(CallToolResult::structured(value))
            }
            Err(error) => Ok(CallToolResult::error(vec![ContentBlock::text(
                error.to_string(),
            )])),
        }
    }

    #[tool(
        name = "get_observations",
        description = "Fetch a FRED series' observations (date/value pairs). Supports an optional \
                       date range, a units transform, aggregation to a lower frequency, sort \
                       order, and a result limit."
    )]
    async fn get_observations(
        &self,
        Parameters(params): Parameters<GetObservationsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut request = self
            .client
            .observations(&SeriesId::new(params.series_id.as_str()));
        if let Some(start) = &params.start {
            request = request.observation_start(parse_date(start, "start")?);
        }
        if let Some(end) = &params.end {
            request = request.observation_end(parse_date(end, "end")?);
        }
        if let Some(limit) = params.limit {
            request = request.limit(limit);
        }
        if let Some(units) = params.units {
            request = request.units(units.into());
        }
        if let Some(frequency) = params.frequency {
            request = request.frequency(frequency.into());
        }
        if let Some(aggregation) = params.aggregation {
            request = request.aggregation_method(aggregation.into());
        }
        if let Some(sort) = params.sort {
            request = request.sort_order(sort.into());
        }

        match request.send().await {
            Ok(observations) => {
                let value = serde_json::json!({
                    "series_id": params.series_id,
                    "count": observations.len(),
                    "observations": observations,
                });
                Ok(CallToolResult::structured(value))
            }
            Err(error) => Ok(CallToolResult::error(vec![ContentBlock::text(
                error.to_string(),
            )])),
        }
    }

    #[tool(
        name = "get_category",
        description = "Fetch a FRED category by its id (0 is the root of the category tree): its \
                       name and parent category id."
    )]
    async fn get_category(
        &self,
        Parameters(params): Parameters<CategoryParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self
            .client
            .category(CategoryId::new(params.category_id))
            .await
        {
            Ok(category) => {
                let value = serde_json::to_value(&category)
                    .map_err(|error| ErrorData::internal_error(error.to_string(), None))?;
                Ok(CallToolResult::structured(value))
            }
            Err(error) => Ok(CallToolResult::error(vec![ContentBlock::text(
                error.to_string(),
            )])),
        }
    }

    #[tool(
        name = "get_category_children",
        description = "List the child categories of a FRED category (use id 0 for the top-level \
                       categories). The primary way to walk the category tree downward."
    )]
    async fn get_category_children(
        &self,
        Parameters(params): Parameters<CategoryParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self
            .client
            .category_children(CategoryId::new(params.category_id))
            .await
        {
            Ok(children) => {
                let value = serde_json::json!({
                    "category_id": params.category_id,
                    "count": children.len(),
                    "children": children,
                });
                Ok(CallToolResult::structured(value))
            }
            Err(error) => Ok(CallToolResult::error(vec![ContentBlock::text(
                error.to_string(),
            )])),
        }
    }

    #[tool(
        name = "get_category_series",
        description = "List the FRED series that belong to a category, with pagination metadata \
                       (total count, offset, limit). Supports ordering, sort direction, and a \
                       result limit."
    )]
    async fn get_category_series(
        &self,
        Parameters(params): Parameters<CategorySeriesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut request = self
            .client
            .category_series(CategoryId::new(params.category_id));
        if let Some(limit) = params.limit {
            request = request.limit(limit);
        }
        if let Some(order_by) = params.order_by {
            request = request.order_by(order_by.into());
        }
        if let Some(sort) = params.sort {
            request = request.sort_order(sort.into());
        }

        match request.send().await {
            Ok(results) => {
                let value = serde_json::to_value(&results)
                    .map_err(|error| ErrorData::internal_error(error.to_string(), None))?;
                Ok(CallToolResult::structured(value))
            }
            Err(error) => Ok(CallToolResult::error(vec![ContentBlock::text(
                error.to_string(),
            )])),
        }
    }

    #[tool(
        name = "get_releases",
        description = "List FRED data releases (publications such as \"Gross Domestic Product\"), \
                       with pagination metadata. A browse axis parallel to categories. Supports \
                       sort direction and a result limit."
    )]
    async fn get_releases(
        &self,
        Parameters(params): Parameters<GetReleasesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut request = self.client.releases();
        if let Some(limit) = params.limit {
            request = request.limit(limit);
        }
        if let Some(sort) = params.sort {
            request = request.sort_order(sort.into());
        }

        match request.send().await {
            Ok(results) => {
                let value = serde_json::to_value(&results)
                    .map_err(|error| ErrorData::internal_error(error.to_string(), None))?;
                Ok(CallToolResult::structured(value))
            }
            Err(error) => Ok(CallToolResult::error(vec![ContentBlock::text(
                error.to_string(),
            )])),
        }
    }

    #[tool(
        name = "get_release",
        description = "Fetch a FRED data release by its id (e.g. 53 = Gross Domestic Product): its \
                       name, press-release flag, and link."
    )]
    async fn get_release(
        &self,
        Parameters(params): Parameters<GetReleaseParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client.release(ReleaseId::new(params.release_id)).await {
            Ok(release) => {
                let value = serde_json::to_value(&release)
                    .map_err(|error| ErrorData::internal_error(error.to_string(), None))?;
                Ok(CallToolResult::structured(value))
            }
            Err(error) => Ok(CallToolResult::error(vec![ContentBlock::text(
                error.to_string(),
            )])),
        }
    }

    #[tool(
        name = "get_release_series",
        description = "List the FRED series published in a release, with pagination metadata \
                       (total count, offset, limit). Supports ordering, sort direction, and a \
                       result limit."
    )]
    async fn get_release_series(
        &self,
        Parameters(params): Parameters<ReleaseSeriesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut request = self
            .client
            .release_series(ReleaseId::new(params.release_id));
        if let Some(limit) = params.limit {
            request = request.limit(limit);
        }
        if let Some(order_by) = params.order_by {
            request = request.order_by(order_by.into());
        }
        if let Some(sort) = params.sort {
            request = request.sort_order(sort.into());
        }

        match request.send().await {
            Ok(results) => {
                let value = serde_json::to_value(&results)
                    .map_err(|error| ErrorData::internal_error(error.to_string(), None))?;
                Ok(CallToolResult::structured(value))
            }
            Err(error) => Ok(CallToolResult::error(vec![ContentBlock::text(
                error.to_string(),
            )])),
        }
    }
}

/// Parse a `YYYY-MM-DD` date from a tool argument, mapping a bad format to an
/// `invalid_params` protocol error naming the offending field.
fn parse_date(raw: &str, field: &str) -> Result<NaiveDate, ErrorData> {
    NaiveDate::parse_from_str(raw, "%Y-%m-%d").map_err(|_| {
        ErrorData::invalid_params(
            format!("invalid `{field}` date `{raw}`, expected YYYY-MM-DD"),
            None,
        )
    })
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
            "Query FRED (Federal Reserve Economic Data). Tools: search_series (find series by \
             text), get_series (metadata for a series id), get_observations (a series' date/value \
             observations, with optional date range, units transform, and frequency aggregation), \
             the category tools — get_category, get_category_children (walk the category tree from \
             the root, id 0), and get_category_series (the series in a category) — and the release \
             tools — get_releases (list publications), get_release, and get_release_series (the \
             series in a release)."
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

    #[test]
    fn search_params_deserialize_enums_from_fred_codes() {
        let params: SearchSeriesParams = serde_json::from_value(serde_json::json!({
            "text": "gdp",
            "order_by": "popularity",
            "sort": "desc"
        }))
        .unwrap();
        assert_eq!(params.text, "gdp");
        assert!(matches!(params.order_by, Some(OrderByArg::Popularity)));
        assert!(matches!(params.sort, Some(SortOrderArg::Desc)));
        assert!(params.limit.is_none());
    }

    #[test]
    fn observation_params_optional_fields_default_to_none() {
        let params: GetObservationsParams =
            serde_json::from_value(serde_json::json!({"series_id": "GNPCA"})).unwrap();
        assert_eq!(params.series_id, "GNPCA");
        assert!(params.start.is_none() && params.units.is_none());
    }

    #[test]
    fn category_params_deserialize_from_arguments() {
        let params: CategoryParams =
            serde_json::from_value(serde_json::json!({"category_id": 125})).unwrap();
        assert_eq!(params.category_id, 125);
    }

    #[test]
    fn category_series_params_deserialize_enums_and_default_none() {
        let params: CategorySeriesParams = serde_json::from_value(serde_json::json!({
            "category_id": 13,
            "order_by": "popularity",
            "sort": "desc"
        }))
        .unwrap();
        assert_eq!(params.category_id, 13);
        assert!(matches!(params.order_by, Some(OrderByArg::Popularity)));
        assert!(matches!(params.sort, Some(SortOrderArg::Desc)));
        assert!(params.limit.is_none());
    }

    #[test]
    fn release_params_deserialize_from_arguments() {
        let params: GetReleaseParams =
            serde_json::from_value(serde_json::json!({"release_id": 53})).unwrap();
        assert_eq!(params.release_id, 53);

        let list: GetReleasesParams =
            serde_json::from_value(serde_json::json!({"sort": "desc"})).unwrap();
        assert!(matches!(list.sort, Some(SortOrderArg::Desc)));
        assert!(list.limit.is_none());
    }

    #[test]
    fn release_series_params_deserialize_enums() {
        let params: ReleaseSeriesParams = serde_json::from_value(serde_json::json!({
            "release_id": 53,
            "order_by": "popularity"
        }))
        .unwrap();
        assert_eq!(params.release_id, 53);
        assert!(matches!(params.order_by, Some(OrderByArg::Popularity)));
        assert!(params.sort.is_none());
    }

    #[test]
    fn parse_date_accepts_iso_and_rejects_garbage() {
        assert_eq!(
            parse_date("2020-01-01", "start").unwrap(),
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()
        );
        assert!(parse_date("01/2020", "start").is_err());
    }
}
