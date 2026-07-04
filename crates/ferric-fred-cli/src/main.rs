//! `fred` — a command-line interface to FRED (Federal Reserve Economic Data),
//! built on the `ferric-fred` client.
//!
//! Reads the API key from the `FRED_API_KEY` environment variable. Uses
//! `anyhow` for top-level error context (ADR-0004) over the library's typed
//! errors, and drives the async client with `#[tokio::main]` (ADR-0003).

use anyhow::{Context, Result};
use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use ferric_fred::{Client, SeriesId};

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
        /// Earliest observation date, YYYY-MM-DD.
        #[arg(long)]
        start: Option<NaiveDate>,
        /// Latest observation date, YYYY-MM-DD.
        #[arg(long)]
        end: Option<NaiveDate>,
        /// Maximum number of observations to return.
        #[arg(long)]
        limit: Option<u32>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let client = Client::from_env()
        .context("could not initialize the FRED client (is FRED_API_KEY set?)")?;

    match cli.command {
        Command::Search { text, limit } => search(&client, &text, limit).await,
        Command::Series { id } => series(&client, &id).await,
        Command::Observations {
            id,
            start,
            end,
            limit,
        } => observations(&client, &id, start, end, limit).await,
    }
}

async fn search(client: &Client, text: &str, limit: u32) -> Result<()> {
    let results = client
        .search(text)
        .limit(limit)
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

async fn observations(
    client: &Client,
    id: &str,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    limit: Option<u32>,
) -> Result<()> {
    let mut request = client.observations(&SeriesId::new(id));
    if let Some(start) = start {
        request = request.observation_start(start);
    }
    if let Some(end) = end {
        request = request.observation_end(end);
    }
    if let Some(limit) = limit {
        request = request.limit(limit);
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

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }
}
