//! `fred` — a command-line interface to FRED (Federal Reserve Economic Data),
//! built on the `ferric-fred` client.
//!
//! Reads the API key from the `FRED_API_KEY` environment variable. Uses
//! `anyhow` for top-level error context (ADR-0004) over the library's typed
//! errors, and drives the async client with `#[tokio::main]` (ADR-0003).

mod args;
mod chart;

use anyhow::{Context, Result};
use chrono::NaiveDate;
use clap::{Args, Parser, Subcommand};
use ferric_fred::{CategoryId, Client, ObservationsRequest, ReleaseId, SeriesId};

use args::{AggregationArg, FrequencyArg, OrderByArg, SortOrderArg, UnitsArg};

/// Typed command-line access to FRED (Federal Reserve Economic Data).
#[derive(Parser)]
#[command(name = "fred", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    /// Output results as JSON instead of text (data commands only; `chart`
    /// ignores it).
    #[arg(long, global = true)]
    json: bool,
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
    /// Show metadata for a single series (or, with --tags, its tags).
    Series {
        /// FRED series id, e.g. GNPCA.
        id: String,
        /// List the series' tags instead of its metadata.
        #[arg(long)]
        tags: bool,
    },
    /// Print a series' observations (date and value).
    Observations {
        /// FRED series id, e.g. GNPCA.
        id: String,
        #[command(flatten)]
        options: ObservationOptions,
    },
    /// Draw an interactive terminal chart of a series' observations.
    Chart {
        /// FRED series id, e.g. GNPCA.
        id: String,
        #[command(flatten)]
        options: ObservationOptions,
    },
    /// Browse the FRED category tree, or list a category's series.
    ///
    /// With no flags, prints the category and its child categories (the root,
    /// id 0, by default). With `--series`, lists the series in the category.
    Category {
        /// Category id (default: 0, the tree root).
        #[arg(default_value_t = 0)]
        id: u32,
        /// List the series in the category instead of its child categories.
        #[arg(long)]
        series: bool,
        /// With `--series`: maximum number of series to return.
        #[arg(long)]
        limit: Option<u32>,
        /// With `--series`: field to order results by.
        #[arg(long)]
        order_by: Option<OrderByArg>,
        /// With `--series`: sort order.
        #[arg(long)]
        sort: Option<SortOrderArg>,
    },
    /// List FRED data releases, show one, or list a release's series.
    ///
    /// With no id, lists all releases. With an id, shows that release; add
    /// `--series` to list the series it publishes.
    Release {
        /// Release id. Omit to list all releases.
        id: Option<u32>,
        /// With an id: list the release's series instead of its metadata.
        #[arg(long, requires = "id")]
        series: bool,
        /// Maximum number of results (applies to the list and to `--series`).
        #[arg(long)]
        limit: Option<u32>,
        /// With `--series`: field to order series by.
        #[arg(long)]
        order_by: Option<OrderByArg>,
        /// Sort order.
        #[arg(long)]
        sort: Option<SortOrderArg>,
    },
    /// Browse/search FRED tags, or find series carrying a set of tags.
    ///
    /// With no tag names, browses the tag vocabulary (use `--search-text` to
    /// filter). With one or more tag names, lists the series carrying all of
    /// them.
    Tags {
        /// Tag names. Give one or more to list series carrying all of them;
        /// omit to browse/search the tag vocabulary.
        names: Vec<String>,
        /// With no names: restrict the tag list to those matching this text.
        #[arg(long)]
        search_text: Option<String>,
        /// Maximum number of results.
        #[arg(long)]
        limit: Option<u32>,
        /// With tag names: field to order the matching series by.
        #[arg(long)]
        order_by: Option<OrderByArg>,
        /// Sort order.
        #[arg(long)]
        sort: Option<SortOrderArg>,
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

    let json = cli.json;
    match cli.command {
        Command::Search {
            text,
            limit,
            order_by,
            sort,
        } => search(&client, &text, limit, order_by, sort, json).await,
        Command::Series { id, tags } => series(&client, &id, tags, json).await,
        Command::Observations { id, options } => observations(&client, &id, &options, json).await,
        Command::Chart { id, options } => chart_command(&client, &id, &options).await,
        Command::Category {
            id,
            series,
            limit,
            order_by,
            sort,
        } => category(&client, id, series, limit, order_by, sort, json).await,
        Command::Release {
            id,
            series,
            limit,
            order_by,
            sort,
        } => release(&client, id, series, limit, order_by, sort, json).await,
        Command::Tags {
            names,
            search_text,
            limit,
            order_by,
            sort,
        } => tags(&client, names, search_text, limit, order_by, sort, json).await,
    }
}

/// Print a value as pretty-printed JSON to stdout.
fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(value).context("serializing result to JSON failed")?;
    println!("{json}");
    Ok(())
}

async fn search(
    client: &Client,
    text: &str,
    limit: u32,
    order_by: Option<OrderByArg>,
    sort: Option<SortOrderArg>,
    json: bool,
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

    if json {
        return print_json(&results);
    }

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

async fn series(client: &Client, id: &str, tags: bool, json: bool) -> Result<()> {
    let series_id = SeriesId::new(id);

    if tags {
        let results = client
            .series_tags(&series_id)
            .await
            .with_context(|| format!("fetching tags for series `{id}` failed"))?;

        if json {
            return print_json(&results);
        }

        println!("{} tags for {id}:", results.count);
        for tag in &results.tags {
            println!(
                "{}\t{}\t{} series",
                tag.name, tag.group_id, tag.series_count
            );
        }
        return Ok(());
    }

    let series = client
        .series(&series_id)
        .await
        .with_context(|| format!("fetching series `{id}` failed"))?;

    if json {
        return print_json(&series);
    }

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

async fn category(
    client: &Client,
    id: u32,
    series: bool,
    limit: Option<u32>,
    order_by: Option<OrderByArg>,
    sort: Option<SortOrderArg>,
    json: bool,
) -> Result<()> {
    let category_id = CategoryId::new(id);

    if series {
        let mut request = client.category_series(category_id);
        if let Some(limit) = limit {
            request = request.limit(limit);
        }
        if let Some(order_by) = order_by {
            request = request.order_by(order_by.into());
        }
        if let Some(sort) = sort {
            request = request.sort_order(sort.into());
        }

        let results = request
            .send()
            .await
            .with_context(|| format!("fetching series for category {id} failed"))?;

        if json {
            return print_json(&results);
        }

        println!("{} series in category {id}:", results.count);
        for series in &results.series {
            println!("{}\t{}", series.id, series.title);
        }
        return Ok(());
    }

    let category = client
        .category(category_id)
        .await
        .with_context(|| format!("fetching category {id} failed"))?;
    let children = client
        .category_children(category_id)
        .await
        .with_context(|| format!("fetching children of category {id} failed"))?;

    if json {
        return print_json(&serde_json::json!({
            "category": category,
            "children": children,
        }));
    }

    println!(
        "{}: {}  (parent {})",
        category.id, category.name, category.parent_id
    );
    if children.is_empty() {
        println!("  (no subcategories — use --series to list this category's series)");
    } else {
        for child in &children {
            println!("  {}\t{}", child.id, child.name);
        }
    }
    Ok(())
}

async fn release(
    client: &Client,
    id: Option<u32>,
    series: bool,
    limit: Option<u32>,
    order_by: Option<OrderByArg>,
    sort: Option<SortOrderArg>,
    json: bool,
) -> Result<()> {
    // clap guarantees `--series` is only set alongside an id, so here (no id)
    // `series` is always false — just list all releases.
    let Some(id) = id else {
        let mut request = client.releases();
        if let Some(limit) = limit {
            request = request.limit(limit);
        }
        if let Some(sort) = sort {
            request = request.sort_order(sort.into());
        }

        let results = request.send().await.context("listing releases failed")?;

        if json {
            return print_json(&results);
        }

        println!("{} releases:", results.count);
        for release in &results.releases {
            println!("{}\t{}", release.id, release.name);
        }
        return Ok(());
    };

    let release_id = ReleaseId::new(id);

    if series {
        let mut request = client.release_series(release_id);
        if let Some(limit) = limit {
            request = request.limit(limit);
        }
        if let Some(order_by) = order_by {
            request = request.order_by(order_by.into());
        }
        if let Some(sort) = sort {
            request = request.sort_order(sort.into());
        }

        let results = request
            .send()
            .await
            .with_context(|| format!("fetching series for release {id} failed"))?;

        if json {
            return print_json(&results);
        }

        println!("{} series in release {id}:", results.count);
        for series in &results.series {
            println!("{}\t{}", series.id, series.title);
        }
        return Ok(());
    }

    let release = client
        .release(release_id)
        .await
        .with_context(|| format!("fetching release {id} failed"))?;

    if json {
        return print_json(&release);
    }

    println!("{}: {}", release.id, release.name);
    println!("  press release: {}", release.press_release);
    if let Some(link) = &release.link {
        println!("  link:          {link}");
    }
    Ok(())
}

async fn tags(
    client: &Client,
    names: Vec<String>,
    search_text: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderByArg>,
    sort: Option<SortOrderArg>,
    json: bool,
) -> Result<()> {
    if names.is_empty() {
        // Browse / search the tag vocabulary.
        let mut request = client.tags();
        if let Some(text) = search_text {
            request = request.search_text(text);
        }
        if let Some(limit) = limit {
            request = request.limit(limit);
        }
        if let Some(sort) = sort {
            request = request.sort_order(sort.into());
        }

        let results = request.send().await.context("listing tags failed")?;

        if json {
            return print_json(&results);
        }

        println!("{} tags:", results.count);
        for tag in &results.tags {
            println!(
                "{}\t{}\t{} series",
                tag.name, tag.group_id, tag.series_count
            );
        }
        return Ok(());
    }

    // Series carrying all of the given tags.
    let mut request = client.tags_series(&names);
    if let Some(limit) = limit {
        request = request.limit(limit);
    }
    if let Some(order_by) = order_by {
        request = request.order_by(order_by.into());
    }
    if let Some(sort) = sort {
        request = request.sort_order(sort.into());
    }

    let results = request
        .send()
        .await
        .with_context(|| format!("fetching series for tags {names:?} failed"))?;

    if json {
        return print_json(&results);
    }

    println!("{} series tagged {}:", results.count, names.join(", "));
    for series in &results.series {
        println!("{}\t{}", series.id, series.title);
    }
    Ok(())
}

/// Build an observations request from the shared CLI options. Used by both the
/// `observations` and `chart` commands.
fn build_request<'a>(
    client: &'a Client,
    id: &str,
    options: &ObservationOptions,
) -> ObservationsRequest<'a> {
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
    request
}

async fn observations(
    client: &Client,
    id: &str,
    options: &ObservationOptions,
    json: bool,
) -> Result<()> {
    let observations = build_request(client, id, options)
        .send()
        .await
        .with_context(|| format!("fetching observations for `{id}` failed"))?;

    if json {
        return print_json(&observations);
    }

    println!("{} observation(s):", observations.len());
    for observation in &observations {
        match observation.value {
            Some(value) => println!("{}\t{}", observation.date, value),
            None => println!("{}\t.", observation.date),
        }
    }
    Ok(())
}

async fn chart_command(client: &Client, id: &str, options: &ObservationOptions) -> Result<()> {
    let observations = build_request(client, id, options)
        .send()
        .await
        .with_context(|| format!("fetching observations for `{id}` failed"))?;

    chart::run(&observations, id)
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
