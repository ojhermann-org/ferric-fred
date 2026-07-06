//! `fred` — a command-line interface to FRED (Federal Reserve Economic Data),
//! built on the `ferric-fred` client.
//!
//! Reads the API key from the `FRED_API_KEY` environment variable. Uses
//! `anyhow` for top-level error context (ADR-0004) over the library's typed
//! errors, and drives the async client with `#[tokio::main]` (ADR-0003).

mod args;
mod chart;

use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveDateTime};
use clap::{Args, Parser, Subcommand};
use ferric_fred::{
    CategoryId, Client, ObservationsRequest, Paginate, Release, ReleaseDate, ReleaseDatesResults,
    ReleaseElementId, ReleaseId, ReleaseTableElement, ReleasesResults, Series, SeriesId,
    SeriesSearchResults, Source, SourceId, SourcesResults, Tag, TagsRequest, TagsResults,
    VintageDates,
};

use args::{AggregationArg, FrequencyArg, OrderByArg, SortOrderArg, UnitsArg, UpdatesFilterArg};

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
    /// Fetch every page of a paginated list, not just the first (list views
    /// only; other commands ignore it). `--limit` still caps the total; without
    /// it, all results are returned. Mind FRED's rate limits on large lists.
    #[arg(long, global = true)]
    all: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Search for series matching text, or the tags of the matching series.
    ///
    /// By default lists the matching series. With `--tags`, lists the tags used
    /// by those series; with `--related-tags`, the tags co-occurring with a seed
    /// set among them.
    Search {
        /// Words to search for.
        text: String,
        /// Show the tags of the matching series instead of the series.
        #[arg(long, group = "view")]
        tags: bool,
        /// Show the tags co-occurring with these seed tags (comma-separated)
        /// among the matching series, e.g. --related-tags monthly,nsa.
        #[arg(long, value_delimiter = ',', value_name = "TAGS", group = "view")]
        related_tags: Vec<String>,
        /// Maximum number of results to show (default 10; with `--all`, a cap on
        /// the total).
        #[arg(long)]
        limit: Option<u32>,
        /// With the default series view: field to order results by.
        #[arg(long)]
        order_by: Option<OrderByArg>,
        /// Sort order.
        #[arg(long)]
        sort: Option<SortOrderArg>,
    },
    /// Show a series: its metadata by default, or a related view with a flag.
    ///
    /// `--tags`, `--categories`, `--release`, and `--vintages` are mutually
    /// exclusive.
    Series {
        /// FRED series id, e.g. GNPCA.
        id: String,
        #[command(flatten)]
        view: SeriesViewArgs,
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
    /// Browse the FRED category tree, or list a category's series or tags.
    ///
    /// With no flags, prints the category and its child categories (the root,
    /// id 0, by default). With `--related`, lists related categories elsewhere
    /// in the tree; with `--series`, the series in the category; with `--tags` /
    /// `--related-tags`, the tags used by those series.
    Category {
        /// Category id (default: 0, the tree root).
        #[arg(default_value_t = 0)]
        id: u32,
        /// List categories related to this one instead of its child categories.
        #[arg(long, group = "view")]
        related: bool,
        /// List the series in the category instead of its child categories.
        #[arg(long, group = "view")]
        series: bool,
        /// List the tags used by the category's series.
        #[arg(long, group = "view")]
        tags: bool,
        /// List the tags co-occurring with these seed tags (comma-separated)
        /// within the category, e.g. --related-tags gdp,quarterly.
        #[arg(long, value_delimiter = ',', value_name = "TAGS", group = "view")]
        related_tags: Vec<String>,
        /// With `--series`/`--tags`/`--related-tags`: maximum number of results.
        #[arg(long)]
        limit: Option<u32>,
        /// With `--series`: field to order results by.
        #[arg(long)]
        order_by: Option<OrderByArg>,
        /// With `--series`/`--tags`/`--related-tags`: sort order.
        #[arg(long)]
        sort: Option<SortOrderArg>,
    },
    /// List FRED data releases, show one, or list a release's series, sources,
    /// dates, tags, or table tree.
    ///
    /// With no id, lists all releases — or, with `--dates`, the publication
    /// calendar across every release. With an id, shows that release; add
    /// `--series` to list the series it publishes, `--sources` to list the
    /// sources it draws from, `--dates` for that release's own dates,
    /// `--tags` / `--related-tags` for the tags of its series, or `--tables`
    /// for its table tree.
    Release {
        /// Release id. Omit to list all releases.
        id: Option<u32>,
        /// With an id: list the release's series instead of its metadata.
        #[arg(long, requires = "id", conflicts_with = "sources")]
        series: bool,
        /// With an id: list the release's sources instead of its metadata.
        #[arg(long, requires = "id")]
        sources: bool,
        /// List release dates: the calendar across all releases (no id) or one
        /// release's dates (with an id).
        #[arg(long, conflicts_with_all = ["series", "sources"])]
        dates: bool,
        /// With `--dates`: include dates that have no data yet (e.g. scheduled
        /// future releases).
        #[arg(long, requires = "dates")]
        include_no_data: bool,
        /// With an id: list the tags used by the release's series.
        #[arg(long, requires = "id", conflicts_with_all = ["series", "sources", "dates", "related_tags"])]
        tags: bool,
        /// With an id: list the tags co-occurring with these seed tags
        /// (comma-separated) within the release, e.g. --related-tags gdp.
        #[arg(long, requires = "id", value_delimiter = ',', value_name = "TAGS",
              conflicts_with_all = ["series", "sources", "dates", "tags"])]
        related_tags: Vec<String>,
        /// With an id: print the release's table tree (sections, tables, and
        /// the series rows nested beneath them).
        #[arg(long, requires = "id",
              conflicts_with_all = ["series", "sources", "dates", "tags", "related_tags"])]
        tables: bool,
        /// With `--tables`: print only the subtree rooted at this element id.
        #[arg(long, requires = "tables", value_name = "ELEMENT_ID")]
        element: Option<u32>,
        /// With `--tables`: fold each series row's latest observation value into
        /// the tree (structure-only otherwise).
        #[arg(long, requires = "tables")]
        observation_values: bool,
        /// With `--tables`: observation value as of this date (`YYYY-MM-DD`);
        /// implies `--observation-values`.
        #[arg(long, requires = "tables", value_name = "YYYY-MM-DD")]
        observation_date: Option<NaiveDate>,
        /// Maximum number of results (applies to the list, `--series`, `--dates`, and tag views).
        #[arg(long)]
        limit: Option<u32>,
        /// With `--series`: field to order series by.
        #[arg(long)]
        order_by: Option<OrderByArg>,
        /// Sort order.
        #[arg(long)]
        sort: Option<SortOrderArg>,
    },
    /// List the series updated most recently (a "what changed" feed).
    Updates {
        /// Narrow to a class of series (default: all).
        #[arg(long)]
        filter: Option<UpdatesFilterArg>,
        /// Only series updated at/after this time (needs `--end-time`), e.g.
        /// `2024-03-01T14:30` or `2024-03-01 14:30` — FRED's timezone.
        #[arg(long, value_parser = parse_datetime, requires = "end_time")]
        start_time: Option<NaiveDateTime>,
        /// Only series updated at/before this time (needs `--start-time`).
        #[arg(long, value_parser = parse_datetime, requires = "start_time")]
        end_time: Option<NaiveDateTime>,
        /// Maximum number of results to show (default 20; with `--all`, a cap on
        /// the total).
        #[arg(long)]
        limit: Option<u32>,
    },
    /// List FRED data sources, show one, or list a source's releases.
    ///
    /// With no id, lists all sources. With an id, shows that source; add
    /// `--releases` to list the releases it produces.
    Source {
        /// Source id. Omit to list all sources.
        id: Option<u32>,
        /// With an id: list the source's releases instead of its metadata.
        #[arg(long, requires = "id")]
        releases: bool,
        /// Maximum number of results (applies to the list and to `--releases`).
        #[arg(long)]
        limit: Option<u32>,
        /// Sort order.
        #[arg(long)]
        sort: Option<SortOrderArg>,
    },
    /// Browse/search FRED tags, find series by tags, or find related tags.
    ///
    /// With no tag names, browses the tag vocabulary (use `--search-text` to
    /// filter). With one or more tag names, lists the series carrying all of
    /// them — or, with `--related`, the tags that co-occur with them.
    Tags {
        /// Tag names. Give one or more to list series carrying all of them (or,
        /// with --related, related tags); omit to browse the tag vocabulary.
        names: Vec<String>,
        #[command(flatten)]
        options: TagsOptions,
    },
}

/// The mutually-exclusive "view" flags for the `series` command. With none set,
/// the command prints the series' metadata.
//
// Four bool fields trips clippy::struct_excessive_bools, but they *are* the CLI
// surface (--tags / --categories / --release / --vintages); a clap flag group is
// their idiomatic representation. `selected()` collapses them to a `SeriesView`
// so the rest of the code never juggles the bools.
#[allow(clippy::struct_excessive_bools)]
#[derive(Args)]
struct SeriesViewArgs {
    /// Show the series' tags instead of its metadata.
    #[arg(long, group = "view")]
    tags: bool,
    /// Show the categories the series belongs to.
    #[arg(long, group = "view")]
    categories: bool,
    /// Show the release the series belongs to.
    #[arg(long, group = "view")]
    release: bool,
    /// Show the dates the series was revised (its vintage dates).
    #[arg(long, group = "view")]
    vintages: bool,
}

impl SeriesViewArgs {
    /// Resolve the flags to the single selected view (clap's `group` guarantees
    /// at most one is set).
    fn selected(&self) -> SeriesView {
        if self.tags {
            SeriesView::Tags
        } else if self.categories {
            SeriesView::Categories
        } else if self.release {
            SeriesView::Release
        } else if self.vintages {
            SeriesView::Vintages
        } else {
            SeriesView::Metadata
        }
    }
}

/// Which view of a series to print.
#[derive(Debug, Clone, Copy)]
enum SeriesView {
    Metadata,
    Tags,
    Categories,
    Release,
    Vintages,
}

/// Options for the `tags` command.
#[derive(Args)]
struct TagsOptions {
    /// With tag names: list the tags that co-occur with them, instead of the
    /// matching series.
    #[arg(long)]
    related: bool,
    /// When browsing or with --related: restrict tags to those matching this text.
    #[arg(long)]
    search_text: Option<String>,
    /// Maximum number of results.
    #[arg(long)]
    limit: Option<u32>,
    /// With tag names (series mode): field to order the matching series by.
    #[arg(long)]
    order_by: Option<OrderByArg>,
    /// Sort order.
    #[arg(long)]
    sort: Option<SortOrderArg>,
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
    /// ALFRED: start of the real-time period (YYYY-MM-DD) — the data as it was
    /// known then. Use the same value for --realtime-end to snapshot the series
    /// as of one day (point-in-time). Needs --realtime-end.
    #[arg(long, requires = "realtime_end", value_name = "YYYY-MM-DD")]
    realtime_start: Option<NaiveDate>,
    /// ALFRED: end of the real-time period (YYYY-MM-DD). Needs --realtime-start.
    #[arg(long, requires = "realtime_start", value_name = "YYYY-MM-DD")]
    realtime_end: Option<NaiveDate>,
    /// ALFRED: specific revision dates to fetch (comma-separated YYYY-MM-DD),
    /// e.g. --vintage-dates 2020-03-26,2021-03-25.
    #[arg(long, value_delimiter = ',', value_name = "DATES")]
    vintage_dates: Vec<NaiveDate>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let client = Client::from_env()
        .context("could not initialize the FRED client (is FRED_API_KEY set?)")?;

    let json = cli.json;
    let all = cli.all;
    match cli.command {
        Command::Search {
            text,
            tags,
            related_tags,
            limit,
            order_by,
            sort,
        } => {
            search(
                &client,
                SearchArgs {
                    text,
                    tags,
                    related_tags,
                    limit,
                    order_by,
                    sort,
                },
                json,
                all,
            )
            .await
        }
        Command::Series { id, view } => series(&client, &id, view.selected(), json, all).await,
        Command::Observations { id, options } => observations(&client, &id, &options, json).await,
        Command::Chart { id, options } => chart_command(&client, &id, &options).await,
        Command::Category {
            id,
            related,
            series,
            tags,
            related_tags,
            limit,
            order_by,
            sort,
        } => {
            category(
                &client,
                CategoryArgs {
                    id,
                    related,
                    series,
                    tags,
                    related_tags,
                    limit,
                    order_by,
                    sort,
                },
                json,
                all,
            )
            .await
        }
        Command::Release {
            id,
            series,
            sources,
            dates,
            include_no_data,
            tags,
            related_tags,
            tables,
            element,
            observation_values,
            observation_date,
            limit,
            order_by,
            sort,
        } => {
            release(
                &client,
                ReleaseArgs {
                    id,
                    series,
                    sources,
                    dates,
                    include_no_data,
                    tags,
                    related_tags,
                    tables,
                    element,
                    observation_values,
                    observation_date,
                    limit,
                    order_by,
                    sort,
                },
                json,
                all,
            )
            .await
        }
        Command::Source {
            id,
            releases,
            limit,
            sort,
        } => source(&client, id, releases, limit, sort, json, all).await,
        Command::Tags { names, options } => tags(&client, names, &options, json, all).await,
        Command::Updates {
            filter,
            start_time,
            end_time,
            limit,
        } => updates(&client, filter, start_time, end_time, limit, json, all).await,
    }
}

/// Print a value as pretty-printed JSON to stdout.
fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(value).context("serializing result to JSON failed")?;
    println!("{json}");
    Ok(())
}

// Wrap a fully-collected `--all` (`send_all`) result set back into the paginated
// response shape the printers expect: `count` is the number collected and
// `offset` is 0, so `--all` reuses the same text and JSON output as a single
// page. Since every result was fetched, `count` is the true total.

fn all_series(series: Vec<Series>) -> SeriesSearchResults {
    let count = series.len() as u32;
    SeriesSearchResults {
        series,
        count,
        offset: 0,
        limit: count,
    }
}

fn all_tags(tags: Vec<Tag>) -> TagsResults {
    let count = tags.len() as u32;
    TagsResults {
        count,
        offset: 0,
        limit: count,
        tags,
    }
}

fn all_releases(releases: Vec<Release>) -> ReleasesResults {
    let count = releases.len() as u32;
    ReleasesResults {
        count,
        offset: 0,
        limit: count,
        releases,
    }
}

fn all_release_dates(release_dates: Vec<ReleaseDate>) -> ReleaseDatesResults {
    let count = release_dates.len() as u32;
    ReleaseDatesResults {
        count,
        offset: 0,
        limit: count,
        release_dates,
    }
}

fn all_sources(sources: Vec<Source>) -> SourcesResults {
    let count = sources.len() as u32;
    SourcesResults {
        count,
        offset: 0,
        limit: count,
        sources,
    }
}

fn all_vintage_dates(vintage_dates: Vec<NaiveDate>) -> VintageDates {
    let count = vintage_dates.len() as u32;
    VintageDates {
        count,
        offset: 0,
        limit: count,
        vintage_dates,
    }
}

/// Run a scoped tags request (applying limit/sort), then print the resulting
/// tags as text — or as JSON under `--json`. Shared by the tag-facet views on
/// the `category`, `release`, and `search` commands.
async fn print_tags_result(
    request: TagsRequest<'_>,
    limit: Option<u32>,
    sort: Option<SortOrderArg>,
    heading: &str,
    json: bool,
    all: bool,
) -> Result<()> {
    let mut request = request;
    if let Some(limit) = limit {
        request = request.limit(limit);
    }
    if let Some(sort) = sort {
        request = request.sort_order(sort.into());
    }

    let results = if all {
        all_tags(
            request
                .send_all()
                .await
                .with_context(|| format!("fetching {heading} failed"))?,
        )
    } else {
        request
            .send()
            .await
            .with_context(|| format!("fetching {heading} failed"))?
    };

    if json {
        return print_json(&results);
    }

    println!("{} {heading}:", results.count);
    print_tag_lines(&results.tags);
    Ok(())
}

/// Parsed arguments for the `search` command: a series search, or (with `--tags`
/// / `--related-tags`) the tag facets of the matching series.
struct SearchArgs {
    text: String,
    tags: bool,
    related_tags: Vec<String>,
    limit: Option<u32>,
    order_by: Option<OrderByArg>,
    sort: Option<SortOrderArg>,
}

async fn search(client: &Client, args: SearchArgs, json: bool, all: bool) -> Result<()> {
    let SearchArgs {
        text,
        tags,
        related_tags,
        limit,
        order_by,
        sort,
    } = args;

    // Single-page search defaults to 10 results; `--all` drops that default so a
    // bare `--limit` becomes a ceiling and no `--limit` means "everything".
    let limit = if all {
        limit
    } else {
        Some(limit.unwrap_or(10))
    };

    if tags {
        return print_tags_result(
            client.series_search_tags(text.as_str()),
            limit,
            sort,
            &format!("tags for series matching {text:?}"),
            json,
            all,
        )
        .await;
    }
    if !related_tags.is_empty() {
        return print_tags_result(
            client.series_search_related_tags(text.as_str(), &related_tags),
            limit,
            sort,
            &format!(
                "tags related to {} among series matching {text:?}",
                related_tags.join(", ")
            ),
            json,
            all,
        )
        .await;
    }

    let mut request = client.search(text.as_str());
    if let Some(limit) = limit {
        request = request.limit(limit);
    }
    if let Some(order_by) = order_by {
        request = request.order_by(order_by.into());
    }
    if let Some(sort) = sort {
        request = request.sort_order(sort.into());
    }

    let results = if all {
        all_series(
            request
                .send_all()
                .await
                .with_context(|| format!("search for {text:?} failed"))?,
        )
    } else {
        request
            .send()
            .await
            .with_context(|| format!("search for {text:?} failed"))?
    };

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

async fn series(client: &Client, id: &str, view: SeriesView, json: bool, all: bool) -> Result<()> {
    let series_id = SeriesId::new(id);

    match view {
        SeriesView::Tags => {
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
        }

        SeriesView::Categories => {
            let categories = client
                .series_categories(&series_id)
                .await
                .with_context(|| format!("fetching categories for series `{id}` failed"))?;

            if json {
                return print_json(&categories);
            }

            println!("{} categories for {id}:", categories.len());
            for category in &categories {
                println!("{}\t{}", category.id, category.name);
            }
        }

        SeriesView::Release => {
            let release = client
                .series_release(&series_id)
                .await
                .with_context(|| format!("fetching release for series `{id}` failed"))?;

            if json {
                return print_json(&release);
            }

            println!("release for {id}: {} ({})", release.name, release.id);
            if let Some(link) = &release.link {
                println!("  link: {link}");
            }
        }

        SeriesView::Vintages => {
            let request = client.series_vintagedates(&series_id);
            let dates =
                if all {
                    all_vintage_dates(request.send_all().await.with_context(|| {
                        format!("fetching vintage dates for series `{id}` failed")
                    })?)
                } else {
                    request.send().await.with_context(|| {
                        format!("fetching vintage dates for series `{id}` failed")
                    })?
                };

            if json {
                return print_json(&dates);
            }

            println!("{} vintage dates for {id}:", dates.count);
            for date in &dates.vintage_dates {
                println!("{date}");
            }
        }

        SeriesView::Metadata => {
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
        }
    }

    Ok(())
}

/// Parsed arguments for the `category` command: browse the tree, or list a
/// category's related categories, series, or tag facets.
struct CategoryArgs {
    id: u32,
    related: bool,
    series: bool,
    tags: bool,
    related_tags: Vec<String>,
    limit: Option<u32>,
    order_by: Option<OrderByArg>,
    sort: Option<SortOrderArg>,
}

async fn category(client: &Client, args: CategoryArgs, json: bool, all: bool) -> Result<()> {
    let CategoryArgs {
        id,
        related,
        series,
        tags,
        related_tags,
        limit,
        order_by,
        sort,
    } = args;
    let category_id = CategoryId::new(id);

    if related {
        let categories = client
            .category_related(category_id)
            .await
            .with_context(|| format!("fetching categories related to {id} failed"))?;

        if json {
            return print_json(&categories);
        }

        println!("{} categories related to {id}:", categories.len());
        for category in &categories {
            println!("{}\t{}", category.id, category.name);
        }
        return Ok(());
    }

    if tags {
        return print_tags_result(
            client.category_tags(category_id),
            limit,
            sort,
            &format!("tags in category {id}"),
            json,
            all,
        )
        .await;
    }
    if !related_tags.is_empty() {
        return print_tags_result(
            client.category_related_tags(category_id, &related_tags),
            limit,
            sort,
            &format!(
                "tags related to {} in category {id}",
                related_tags.join(", ")
            ),
            json,
            all,
        )
        .await;
    }

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

        let results = if all {
            all_series(
                request
                    .send_all()
                    .await
                    .with_context(|| format!("fetching series for category {id} failed"))?,
            )
        } else {
            request
                .send()
                .await
                .with_context(|| format!("fetching series for category {id} failed"))?
        };

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

/// Print a release-table element and its descendants, indented by depth. A
/// `series`-type row shows its series id in brackets, and — when observation
/// values were requested — its value and FRED-formatted date (` = 292.26 [Jun
/// 2023]`).
fn print_table_element(element: &ReleaseTableElement, depth: usize) {
    let indent = "  ".repeat(depth);
    match &element.series_id {
        Some(series_id) => print!(
            "{indent}{}  ({}, {series_id})",
            element.name, element.element_type
        ),
        None => print!("{indent}{}  ({})", element.name, element.element_type),
    }
    if let Some(value) = element.observation_value {
        match &element.observation_date {
            Some(date) => print!(" = {value} [{date}]"),
            None => print!(" = {value}"),
        }
    }
    println!();
    for child in &element.children {
        print_table_element(child, depth + 1);
    }
}

/// Parsed arguments for the `release` command. A struct rather than positional
/// parameters because `release` now carries several mutually-exclusive view
/// flags (`--series`/`--sources`/`--dates`/`--tags`/`--related-tags`) plus their
/// modifiers.
struct ReleaseArgs {
    id: Option<u32>,
    series: bool,
    sources: bool,
    dates: bool,
    include_no_data: bool,
    tags: bool,
    related_tags: Vec<String>,
    tables: bool,
    element: Option<u32>,
    observation_values: bool,
    observation_date: Option<NaiveDate>,
    limit: Option<u32>,
    order_by: Option<OrderByArg>,
    sort: Option<SortOrderArg>,
}

async fn release(client: &Client, args: ReleaseArgs, json: bool, all: bool) -> Result<()> {
    let ReleaseArgs {
        id,
        series,
        sources,
        dates,
        include_no_data,
        tags,
        related_tags,
        tables,
        element,
        observation_values,
        observation_date,
        limit,
        order_by,
        sort,
    } = args;

    // clap guarantees `--series`/`--sources` are only set alongside an id, so
    // here (no id) both are false. `--dates` is allowed without an id and lists
    // the calendar across every release; otherwise just list all releases.
    let Some(id) = id else {
        if dates {
            let mut request = client.releases_dates();
            if let Some(limit) = limit {
                request = request.limit(limit);
            }
            if let Some(sort) = sort {
                request = request.sort_order(sort.into());
            }
            if include_no_data {
                request = request.include_dates_with_no_data(true);
            }

            let results = if all {
                all_release_dates(
                    request
                        .send_all()
                        .await
                        .context("listing release dates failed")?,
                )
            } else {
                request
                    .send()
                    .await
                    .context("listing release dates failed")?
            };

            if json {
                return print_json(&results);
            }

            // `releases/dates` spans every release, so each row names its own.
            println!("{} release dates:", results.count);
            for date in &results.release_dates {
                match &date.release_name {
                    Some(name) => println!("{}\t{}\t{}", date.date, date.release_id, name),
                    None => println!("{}\t{}", date.date, date.release_id),
                }
            }
            return Ok(());
        }

        let mut request = client.releases();
        if let Some(limit) = limit {
            request = request.limit(limit);
        }
        if let Some(sort) = sort {
            request = request.sort_order(sort.into());
        }

        let results = if all {
            all_releases(
                request
                    .send_all()
                    .await
                    .context("listing releases failed")?,
            )
        } else {
            request.send().await.context("listing releases failed")?
        };

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

        let results = if all {
            all_series(
                request
                    .send_all()
                    .await
                    .with_context(|| format!("fetching series for release {id} failed"))?,
            )
        } else {
            request
                .send()
                .await
                .with_context(|| format!("fetching series for release {id} failed"))?
        };

        if json {
            return print_json(&results);
        }

        println!("{} series in release {id}:", results.count);
        for series in &results.series {
            println!("{}\t{}", series.id, series.title);
        }
        return Ok(());
    }

    if sources {
        // `/release/sources` is unpaginated — no limit/sort to apply.
        let sources = client
            .release_sources(release_id)
            .await
            .with_context(|| format!("fetching sources for release {id} failed"))?;

        if json {
            return print_json(&sources);
        }

        println!("{} sources for release {id}:", sources.len());
        for source in &sources {
            println!("{}\t{}", source.id, source.name);
        }
        return Ok(());
    }

    if dates {
        let mut request = client.release_dates(release_id);
        if let Some(limit) = limit {
            request = request.limit(limit);
        }
        if let Some(sort) = sort {
            request = request.sort_order(sort.into());
        }
        if include_no_data {
            request = request.include_dates_with_no_data(true);
        }

        let results = if all {
            all_release_dates(
                request
                    .send_all()
                    .await
                    .with_context(|| format!("fetching dates for release {id} failed"))?,
            )
        } else {
            request
                .send()
                .await
                .with_context(|| format!("fetching dates for release {id} failed"))?
        };

        if json {
            return print_json(&results);
        }

        // The release is fixed, so entries omit the name — just print the dates.
        println!("{} release dates for release {id}:", results.count);
        for date in &results.release_dates {
            println!("{}", date.date);
        }
        return Ok(());
    }

    if tags {
        return print_tags_result(
            client.release_tags(release_id),
            limit,
            sort,
            &format!("tags in release {id}"),
            json,
            all,
        )
        .await;
    }
    if !related_tags.is_empty() {
        return print_tags_result(
            client.release_related_tags(release_id, &related_tags),
            limit,
            sort,
            &format!(
                "tags related to {} in release {id}",
                related_tags.join(", ")
            ),
            json,
            all,
        )
        .await;
    }

    if tables {
        let mut request = client.release_tables(release_id);
        if let Some(element_id) = element {
            request = request.element(ReleaseElementId::new(element_id));
        }
        if observation_values {
            request = request.include_observation_values(true);
        }
        if let Some(date) = observation_date {
            request = request.observation_date(date);
        }

        let table = request
            .send()
            .await
            .with_context(|| format!("fetching table tree for release {id} failed"))?;

        if json {
            return print_json(&table);
        }

        match &table.name {
            Some(name) => println!("release {id} table — {name}:"),
            None => println!("release {id} tables:"),
        }
        if table.roots.is_empty() {
            println!("  (no table elements)");
        } else {
            for root in &table.roots {
                print_table_element(root, 1);
            }
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

async fn source(
    client: &Client,
    id: Option<u32>,
    releases: bool,
    limit: Option<u32>,
    sort: Option<SortOrderArg>,
    json: bool,
    all: bool,
) -> Result<()> {
    // clap guarantees `--releases` is only set alongside an id, so here (no id)
    // just list all sources.
    let Some(id) = id else {
        let mut request = client.sources();
        if let Some(limit) = limit {
            request = request.limit(limit);
        }
        if let Some(sort) = sort {
            request = request.sort_order(sort.into());
        }

        let results = if all {
            all_sources(request.send_all().await.context("listing sources failed")?)
        } else {
            request.send().await.context("listing sources failed")?
        };

        if json {
            return print_json(&results);
        }

        println!("{} sources:", results.count);
        for source in &results.sources {
            println!("{}\t{}", source.id, source.name);
        }
        return Ok(());
    };

    let source_id = SourceId::new(id);

    if releases {
        let mut request = client.source_releases(source_id);
        if let Some(limit) = limit {
            request = request.limit(limit);
        }
        if let Some(sort) = sort {
            request = request.sort_order(sort.into());
        }

        let results = if all {
            all_releases(
                request
                    .send_all()
                    .await
                    .with_context(|| format!("fetching releases for source {id} failed"))?,
            )
        } else {
            request
                .send()
                .await
                .with_context(|| format!("fetching releases for source {id} failed"))?
        };

        if json {
            return print_json(&results);
        }

        println!("{} releases from source {id}:", results.count);
        for release in &results.releases {
            println!("{}\t{}", release.id, release.name);
        }
        return Ok(());
    }

    let source = client
        .source(source_id)
        .await
        .with_context(|| format!("fetching source {id} failed"))?;

    if json {
        return print_json(&source);
    }

    println!("{}: {}", source.id, source.name);
    if let Some(link) = &source.link {
        println!("  link: {link}");
    }
    Ok(())
}

/// Print a page of tags as `name<TAB>group<TAB>N series` lines.
fn print_tag_lines(tags: &[ferric_fred::Tag]) {
    for tag in tags {
        println!(
            "{}\t{}\t{} series",
            tag.name, tag.group_id, tag.series_count
        );
    }
}

async fn tags(
    client: &Client,
    names: Vec<String>,
    options: &TagsOptions,
    json: bool,
    all: bool,
) -> Result<()> {
    if names.is_empty() {
        anyhow::ensure!(
            !options.related,
            "--related needs one or more tag names, e.g. `fred tags gdp --related`"
        );

        // Browse / search the tag vocabulary.
        let mut request = client.tags();
        if let Some(text) = &options.search_text {
            request = request.search_text(text.clone());
        }
        if let Some(limit) = options.limit {
            request = request.limit(limit);
        }
        if let Some(sort) = options.sort {
            request = request.sort_order(sort.into());
        }

        let results = if all {
            all_tags(request.send_all().await.context("listing tags failed")?)
        } else {
            request.send().await.context("listing tags failed")?
        };

        if json {
            return print_json(&results);
        }

        println!("{} tags:", results.count);
        print_tag_lines(&results.tags);
        return Ok(());
    }

    if options.related {
        // Tags that co-occur with the given tags.
        let mut request = client.related_tags(&names);
        if let Some(text) = &options.search_text {
            request = request.search_text(text.clone());
        }
        if let Some(limit) = options.limit {
            request = request.limit(limit);
        }
        if let Some(sort) = options.sort {
            request = request.sort_order(sort.into());
        }

        let results = if all {
            all_tags(
                request
                    .send_all()
                    .await
                    .with_context(|| format!("fetching tags related to {names:?} failed"))?,
            )
        } else {
            request
                .send()
                .await
                .with_context(|| format!("fetching tags related to {names:?} failed"))?
        };

        if json {
            return print_json(&results);
        }

        println!("{} tags related to {}:", results.count, names.join(", "));
        print_tag_lines(&results.tags);
        return Ok(());
    }

    // Series carrying all of the given tags.
    let mut request = client.tags_series(&names);
    if let Some(limit) = options.limit {
        request = request.limit(limit);
    }
    if let Some(order_by) = options.order_by {
        request = request.order_by(order_by.into());
    }
    if let Some(sort) = options.sort {
        request = request.sort_order(sort.into());
    }

    let results = if all {
        all_series(
            request
                .send_all()
                .await
                .with_context(|| format!("fetching series for tags {names:?} failed"))?,
        )
    } else {
        request
            .send()
            .await
            .with_context(|| format!("fetching series for tags {names:?} failed"))?
    };

    if json {
        return print_json(&results);
    }

    println!("{} series tagged {}:", results.count, names.join(", "));
    for series in &results.series {
        println!("{}\t{}", series.id, series.title);
    }
    Ok(())
}

/// Parse a `--start-time` / `--end-time` value into a `NaiveDateTime`. Accepts
/// `YYYY-MM-DDTHH:MM[:SS]` or `YYYY-MM-DD HH:MM[:SS]` (seconds optional). FRED's
/// window is minute-granularity in its own timezone (ADR-0019).
fn parse_datetime(raw: &str) -> Result<NaiveDateTime, String> {
    const FORMATS: [&str; 4] = [
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
    ];
    FORMATS
        .iter()
        .find_map(|fmt| NaiveDateTime::parse_from_str(raw, fmt).ok())
        .ok_or_else(|| {
            format!(
                "expected a date-time like `2024-03-01T14:30` or `2024-03-01 14:30`, got `{raw}`"
            )
        })
}

async fn updates(
    client: &Client,
    filter: Option<UpdatesFilterArg>,
    start_time: Option<NaiveDateTime>,
    end_time: Option<NaiveDateTime>,
    limit: Option<u32>,
    json: bool,
    all: bool,
) -> Result<()> {
    // Single-page updates default to 20 results; `--all` drops that default so a
    // bare `--limit` becomes a ceiling and no `--limit` means "everything".
    let limit = if all {
        limit
    } else {
        Some(limit.unwrap_or(20))
    };

    let mut request = client.series_updates();
    if let Some(limit) = limit {
        request = request.limit(limit);
    }
    if let Some(filter) = filter {
        request = request.filter(filter.into());
    }
    // clap enforces that `--start-time` and `--end-time` are given together, so
    // by here they are both present or both absent.
    if let (Some(start), Some(end)) = (start_time, end_time) {
        request = request.time_window(start, end);
    }

    let results = if all {
        all_series(
            request
                .send_all()
                .await
                .context("fetching recently updated series failed")?,
        )
    } else {
        request
            .send()
            .await
            .context("fetching recently updated series failed")?
    };

    if json {
        return print_json(&results);
    }

    println!("{} series updated recently:", results.count);
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
    if let (Some(start), Some(end)) = (options.realtime_start, options.realtime_end) {
        request = request.realtime(start, end);
    }
    if !options.vintage_dates.is_empty() {
        request = request.vintage_dates(options.vintage_dates.iter().copied());
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

    // Show each row's real-time period only for an ALFRED (point-in-time /
    // vintage) query — the latest view stays a clean `date  value`.
    let show_realtime = options.realtime_start.is_some() || !options.vintage_dates.is_empty();

    println!("{} observation(s):", observations.len());
    for observation in &observations {
        let value = match observation.value {
            Some(value) => value.to_string(),
            None => ".".to_string(),
        };
        if show_realtime {
            println!(
                "{}\t{}\t[{}..{}]",
                observation.date, value, observation.realtime_start, observation.realtime_end
            );
        } else {
            println!("{}\t{}", observation.date, value);
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
