//! The `geofred` subcommand group — GeoFRED / Maps API: regional data and the
//! geographic shape files to map it (ADR-0025). Kept in its own module so the
//! four related endpoints don't swell `main.rs`.

use anyhow::{Context, Result};
use chrono::NaiveDate;
use clap::Subcommand;
use ferric_fred::{Client, RegionalData, SeriesGroup, SeriesGroupId, SeriesId, ShapeFile};

use crate::args::{FrequencyArg, RegionTypeArg, SeasonArg, ShapeTypeArg};
use crate::print_json;

/// GeoFRED / Maps endpoints.
#[derive(Subcommand)]
pub(crate) enum GeofredCommand {
    /// A region cross-section for a series group: the value in every region on a
    /// date. FRED requires all of these options.
    Regional {
        /// GeoFRED series-group id, e.g. 882.
        #[arg(value_name = "SERIES_GROUP_ID")]
        series_group: String,
        /// Region granularity to break the data down to.
        #[arg(long)]
        region_type: RegionTypeArg,
        /// The date to report, YYYY-MM-DD.
        #[arg(long, value_name = "YYYY-MM-DD")]
        date: NaiveDate,
        /// Unit-of-measurement label (free text; FRED echoes it into the title),
        /// e.g. "Dollars".
        #[arg(long)]
        units: String,
        /// Reporting frequency.
        #[arg(long)]
        frequency: FrequencyArg,
        /// Seasonal adjustment.
        #[arg(long)]
        season: SeasonArg,
    },
    /// One regional series' values across regions, optionally over time.
    SeriesData {
        /// A regional FRED series id, e.g. SMU56000000500000001.
        #[arg(value_name = "SERIES_ID")]
        series_id: String,
        /// Report a single date, YYYY-MM-DD (default: the most recent).
        #[arg(long, value_name = "YYYY-MM-DD", conflicts_with = "start_date")]
        date: Option<NaiveDate>,
        /// Report every date from this one onward, YYYY-MM-DD.
        #[arg(long, value_name = "YYYY-MM-DD")]
        start_date: Option<NaiveDate>,
    },
    /// The series-group metadata for a regional series (title, region type, date
    /// span).
    Group {
        /// A regional FRED series id, e.g. SMU56000000500000001.
        #[arg(value_name = "SERIES_ID")]
        series_id: String,
    },
    /// Region boundary polygons as GeoJSON. Text prints a summary; use `--json`
    /// for the full geometry.
    Shapes {
        /// Which boundary set to fetch.
        #[arg(long)]
        shape: ShapeTypeArg,
    },
}

pub(crate) async fn run(client: &Client, command: GeofredCommand, json: bool) -> Result<()> {
    match command {
        GeofredCommand::Regional {
            series_group,
            region_type,
            date,
            units,
            frequency,
            season,
        } => {
            let data = client
                .regional_data(
                    &SeriesGroupId::new(series_group.clone()),
                    region_type.into(),
                    date,
                    units,
                    frequency.into(),
                    season.into(),
                )
                .await
                .with_context(|| {
                    format!("fetching regional data for group {series_group} failed")
                })?;
            print_regional_data(&data, json)
        }
        GeofredCommand::SeriesData {
            series_id,
            date,
            start_date,
        } => {
            let mut request = client.series_data(&SeriesId::new(series_id.clone()));
            if let Some(date) = date {
                request = request.date(date);
            }
            if let Some(start_date) = start_date {
                request = request.start_date(start_date);
            }
            let data = request
                .send()
                .await
                .with_context(|| format!("fetching series data for `{series_id}` failed"))?;
            print_regional_data(&data, json)
        }
        GeofredCommand::Group { series_id } => {
            let group = client
                .series_group(&SeriesId::new(series_id.clone()))
                .await
                .with_context(|| format!("fetching series group for `{series_id}` failed"))?;
            print_series_group(&group, json)
        }
        GeofredCommand::Shapes { shape } => {
            let shapes = client
                .shape_file(shape.into())
                .await
                .context("fetching shape file failed")?;
            print_shape_file(&shapes, json)
        }
    }
}

/// Print a [`RegionalData`] result: as JSON under `--json`, else the title and,
/// per date, a `region <TAB> code <TAB> value` line (`.` for a missing value).
fn print_regional_data(data: &RegionalData, json: bool) -> Result<()> {
    if json {
        return print_json(data);
    }

    println!("{}", data.meta.title);
    for (date, points) in &data.meta.data {
        println!("{date}:");
        for point in points {
            let value = match point.value {
                Some(value) => value.to_string(),
                None => ".".to_string(),
            };
            println!("  {}\t{}\t{}", point.region, point.code, value);
        }
    }
    Ok(())
}

/// Print a [`SeriesGroup`]: as JSON under `--json`, else a labelled summary.
fn print_series_group(group: &SeriesGroup, json: bool) -> Result<()> {
    if json {
        return print_json(group);
    }

    println!("{}: {}", group.id, group.title);
    println!("  region type: {}", group.region_type);
    println!("  season:      {}", group.season);
    println!("  units:       {}", group.units);
    println!("  frequency:   {}", group.frequency);
    println!("  dates:       {} .. {}", group.min_date, group.max_date);
    Ok(())
}

/// Print a [`ShapeFile`]: the full GeoJSON under `--json`, else a summary (the
/// geometry is large and in a display projection, so text stays a headline).
fn print_shape_file(shapes: &ShapeFile, json: bool) -> Result<()> {
    if json {
        return print_json(shapes);
    }

    println!("{} ({})", shapes.name, shapes.kind);
    println!("{} feature(s)", shapes.features.len());
    println!("(use --json for the full GeoJSON geometry)");
    Ok(())
}
