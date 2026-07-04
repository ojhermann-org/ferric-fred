//! Interactive terminal chart of a series' observations (the `chart` command),
//! rendered with ratatui.

use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use ferric_fred::Observation;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::style::{Color, Style};
use ratatui::symbols;
use ratatui::text::Line;
use ratatui::widgets::{Axis, Block, Chart, Dataset, GraphType};
use ratatui::{DefaultTerminal, Frame};

/// Observations reduced to a plottable series plus axis bounds, tick labels, and
/// a one-line stats summary.
struct ChartData {
    points: Vec<(f64, f64)>,
    x_bounds: [f64; 2],
    y_bounds: [f64; 2],
    /// X tick labels (dates) low→high: first, midpoint, last.
    x_labels: Vec<String>,
    /// Y tick labels low→high: min, midpoint, max.
    y_labels: Vec<String>,
    /// `n=… min … max … last …`, shown along the chart's bottom border.
    stats: String,
}

/// Build plottable data from observations: drop missing values, sort by date,
/// and compute axis bounds, tick labels, and a stats summary. Returns `None`
/// when fewer than two points have values (nothing meaningful to plot).
fn build_chart_data(observations: &[Observation]) -> Option<ChartData> {
    let mut dated: Vec<(NaiveDate, f64)> = observations
        .iter()
        .filter_map(|observation| observation.value.map(|value| (observation.date, value)))
        .collect();
    dated.sort_by_key(|(date, _)| *date);
    if dated.len() < 2 {
        return None;
    }

    let points: Vec<(f64, f64)> = dated
        .iter()
        .map(|(date, value)| (f64::from(date.num_days_from_ce()), *value))
        .collect();

    let x_bounds = [points[0].0, points[points.len() - 1].0];

    let mut data_min = f64::INFINITY;
    let mut data_max = f64::NEG_INFINITY;
    for &(_, y) in &points {
        data_min = data_min.min(y);
        data_max = data_max.max(y);
    }
    // Plot to the data range, but pad a flat series so its line stays visible.
    // Tick labels track these (padded) bounds so they line up on the axis.
    let (y_lo, y_hi) = if (data_max - data_min).abs() < f64::EPSILON {
        (data_min - 1.0, data_max + 1.0)
    } else {
        (data_min, data_max)
    };

    let first_date = dated[0].0;
    let last_date = dated[dated.len() - 1].0;
    let x_mid = f64::midpoint(x_bounds[0], x_bounds[1]);
    let mid_date = NaiveDate::from_num_days_from_ce_opt(x_mid as i32).unwrap_or(first_date);

    let latest = points[points.len() - 1].1;
    let stats = format!(
        "n={}  min {data_min:.2}  max {data_max:.2}  last {latest:.2}",
        points.len()
    );

    Some(ChartData {
        points,
        x_bounds,
        y_bounds: [y_lo, y_hi],
        x_labels: vec![
            first_date.to_string(),
            mid_date.to_string(),
            last_date.to_string(),
        ],
        y_labels: vec![
            format!("{y_lo:.2}"),
            format!("{:.2}", f64::midpoint(y_lo, y_hi)),
            format!("{y_hi:.2}"),
        ],
        stats,
    })
}

/// Render the chart into a frame.
fn render_chart(frame: &mut Frame, data: &ChartData, title: &str) {
    let dataset = Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&data.points);

    let axis_style = Style::default().fg(Color::Gray);
    let chart = Chart::new(vec![dataset])
        .block(
            Block::bordered()
                .title(format!("{title}  —  press q to quit"))
                .title_bottom(data.stats.as_str()),
        )
        .x_axis(
            Axis::default()
                .style(axis_style)
                .bounds(data.x_bounds)
                .labels(data.x_labels.iter().map(|l| Line::from(l.as_str()))),
        )
        .y_axis(
            Axis::default()
                .style(axis_style)
                .bounds(data.y_bounds)
                .labels(data.y_labels.iter().map(|l| Line::from(l.as_str()))),
        );

    frame.render_widget(chart, frame.area());
}

/// Display an interactive chart of the observations, blocking until the user
/// quits (`q`, `Esc`, or `Ctrl-C`).
///
/// # Errors
///
/// Returns an error if there are too few plottable points, or if the terminal
/// backend fails while drawing or reading events.
pub fn run(observations: &[Observation], title: &str) -> Result<()> {
    let Some(data) = build_chart_data(observations) else {
        anyhow::bail!(
            "not enough data to chart: need at least two observations with values \
             (try a wider --start/--end range)"
        );
    };

    let mut terminal = ratatui::init();
    let result = event_loop(&mut terminal, &data, title);
    ratatui::restore();
    result
}

fn event_loop(terminal: &mut DefaultTerminal, data: &ChartData, title: &str) -> Result<()> {
    loop {
        terminal.draw(|frame| render_chart(frame, data, title))?;

        if let Event::Key(key) = event::read()? {
            let quit = matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
                || (key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.code == KeyCode::Char('c'));
            if quit {
                break;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn obs(year: i32, value: Option<f64>) -> Observation {
        Observation {
            date: NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
            value,
        }
    }

    #[test]
    fn drops_missing_values_and_computes_labels() {
        let data = build_chart_data(&[obs(2020, Some(1.0)), obs(2021, None), obs(2022, Some(3.0))])
            .expect("two plottable points");
        // The `None` is dropped, leaving two plotted points.
        assert_eq!(data.points.len(), 2);
        // Y axis: min / midpoint / max.
        assert_eq!(data.y_labels, ["1.00", "2.00", "3.00"].map(String::from));
        // X axis: first / midpoint / last.
        assert_eq!(data.x_labels.len(), 3);
        assert_eq!(data.x_labels[0], "2020-01-01");
        assert_eq!(data.x_labels[2], "2022-01-01");
    }

    #[test]
    fn stats_summarize_the_series() {
        // Latest date (2022) has value 3.0; min/max span all plotted points.
        let data = build_chart_data(&[
            obs(2020, Some(1.0)),
            obs(2021, Some(5.0)),
            obs(2022, Some(3.0)),
        ])
        .expect("three plottable points");
        assert_eq!(data.stats, "n=3  min 1.00  max 5.00  last 3.00");
    }

    #[test]
    fn too_few_points_is_none() {
        assert!(build_chart_data(&[obs(2020, Some(1.0))]).is_none());
        assert!(build_chart_data(&[obs(2020, None), obs(2021, None)]).is_none());
    }

    #[test]
    fn renders_a_frame_without_error() {
        let series: Vec<Observation> = (2000..=2015)
            .map(|y| obs(y, Some((f64::from(y - 2000) - 7.0).powi(2))))
            .collect();
        let data = build_chart_data(&series).unwrap();
        let mut terminal = Terminal::new(TestBackend::new(72, 16)).unwrap();
        terminal
            .draw(|frame| render_chart(frame, &data, "DEMO"))
            .expect("frame renders without error");
    }
}
