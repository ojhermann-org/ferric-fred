//! `fred` — a command-line interface to FRED (Federal Reserve Economic Data),
//! built on the `ferric-fred` client.
//!
//! Reads the API key from the `FRED_API_KEY` environment variable. Uses
//! `anyhow` for top-level error context (ADR-0004) over the library's typed
//! errors, and drives the async client with `#[tokio::main]` (ADR-0003).

mod args;

use anyhow::{Context, Result};
use chrono::NaiveDate;
use clap::{Args, Parser, Subcommand};
use ferric_fred::{Client, SeriesId};

use args::{AggregationArg, FrequencyArg, OrderByArg, SortOrderArg, UnitsArg};

/// Typed command-line access to FRED (Federal Reserve Economic Data).
#[derive(Parser)]
#[command(name = "fred", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Search for series matching text.
    Search {
        /// Words to search for.
        text: String,
        /// Maximum number of results to show.
        #[arg(long, default_value_t = 10)]
        limit: u32,
        /// Field to order results by.
        #[arg(long)]
        order_by: Option<OrderByArg>,
        /// Sort order.
        #[arg(long)]
        sort: Option<SortOrderArg>,
    },
    /// Show metadata for a single series.
    Series {
        /// FRED series id, e.g. GNPCA.
        id: String,
    },
    /// Print a series' observations (date and value).
    Observations {
        /// FRED series id, e.g. GNPCA.
        id: String,
        #[command(flatten)]
        options: ObservationOptions,
    },
}

/// Options controlling an observations query.
#[derive(Args)]
struct ObservationOptions {
    /// Earliest observation date, YYYY-MM-DD.
    #[arg(long)]
    start: Option<NaiveDate>,
    /// Latest observation date, YYYY-MM-DD.
    #[arg(long)]
    end: Option<NaiveDate>,
    /// Maximum number of observations to return.
    #[arg(long)]
    limit: Option<u32>,
    /// Units transformation to apply.
    #[arg(long)]
    units: Option<UnitsArg>,
    /// Aggregate observations down to this frequency.
    #[arg(long)]
    frequency: Option<FrequencyArg>,
    /// Aggregation method, used together with --frequency.
    #[arg(long)]
    aggregation: Option<AggregationArg>,
    /// Sort order by date.
    #[arg(long)]
    sort: Option<SortOrderArg>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let client = Client::from_env()
        .context("could not initialize the FRED client (is FRED_API_KEY set?)")?;

    match cli.command {
        Command::Search {
            text,
            limit,
            order_by,
            sort,
        } => search(&client, &text, limit, order_by, sort).await,
        Command::Series { id } => series(&client, &id).await,
        Command::Observations { id, options } => observations(&client, &id, options).await,
    }
}

async fn search(
    client: &Client,
    text: &str,
    limit: u32,
    order_by: Option<OrderByArg>,
    sort: Option<SortOrderArg>,
) -> Result<()> {
    let mut request = client.search(text).limit(limit);
    if let Some(order_by) = order_by {
        request = request.order_by(order_by.into());
    }
    if let Some(sort) = sort {
        request = request.sort_order(sort.into());
    }

    let results = request
        .send()
        .await
        .with_context(|| format!("search for {text:?} failed"))?;

    println!(
        "{} match(es); showing {}:",
        results.count,
        results.series.len()
    );
    for series in &results.series {
        println!("{}\t{}", series.id, series.title);
    }
    Ok(())
}

async fn series(client: &Client, id: &str) -> Result<()> {
    let series = client
        .series(&SeriesId::new(id))
        .await
        .with_context(|| format!("fetching series `{id}` failed"))?;

    println!("{}: {}", series.id, series.title);
    println!("  frequency:  {}", series.frequency);
    println!("  seasonal:   {}", series.seasonal_adjustment);
    println!("  units:      {}", series.units);
    println!(
        "  range:      {} .. {}",
        series.observation_start, series.observation_end
    );
    println!("  updated:    {}", series.last_updated);
    Ok(())
}

async fn observations(client: &Client, id: &str, options: ObservationOptions) -> Result<()> {
    let mut request = client.observations(&SeriesId::new(id));
    if let Some(start) = options.start {
        request = request.observation_start(start);
    }
    if let Some(end) = options.end {
        request = request.observation_end(end);
    }
    if let Some(limit) = options.limit {
        request = request.limit(limit);
    }
    if let Some(units) = options.units {
        request = request.units(units.into());
    }
    if let Some(frequency) = options.frequency {
        request = request.frequency(frequency.into());
    }
    if let Some(aggregation) = options.aggregation {
        request = request.aggregation_method(aggregation.into());
    }
    if let Some(sort) = options.sort {
        request = request.sort_order(sort.into());
    }

    let observations = request
        .send()
        .await
        .with_context(|| format!("fetching observations for `{id}` failed"))?;

    println!("{} observation(s):", observations.len());
    for observation in &observations {
        match observation.value {
            Some(value) => println!("{}\t{}", observation.date, value),
            None => println!("{}\t.", observation.date),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use ferric_fred::{SortOrder, Units};

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn arg_enums_convert_to_library_types() {
        assert_eq!(Units::from(UnitsArg::Pch), Units::PercentChange);
        assert_eq!(SortOrder::from(SortOrderArg::Desc), SortOrder::Descending);
    }
}
