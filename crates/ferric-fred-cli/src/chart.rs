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

/// Observations reduced to a plottable series plus axis bounds and labels.
struct ChartData {
    points: Vec<(f64, f64)>,
    x_bounds: [f64; 2],
    y_bounds: [f64; 2],
    x_labels: [String; 2],
    y_labels: [String; 2],
}

/// Build plottable data from observations: drop missing values, sort by date,
/// and compute axis bounds/labels. Returns `None` when fewer than two points
/// have values (nothing meaningful to plot).
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

    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for &(_, y) in &points {
        y_min = y_min.min(y);
        y_max = y_max.max(y);
    }
    if (y_max - y_min).abs() < f64::EPSILON {
        // Flat series: give the axis some height so the line stays visible.
        y_min -= 1.0;
        y_max += 1.0;
    }

    Some(ChartData {
        points,
        x_bounds,
        y_bounds: [y_min, y_max],
        x_labels: [dated[0].0.to_string(), dated[dated.len() - 1].0.to_string()],
        y_labels: [format!("{y_min:.2}"), format!("{y_max:.2}")],
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
        .block(Block::bordered().title(format!("{title}  —  press q to quit")))
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
        assert_eq!(data.points.len(), 2); // the None is dropped
        assert_eq!(data.y_labels, ["1.00".to_string(), "3.00".to_string()]);
        assert_eq!(data.x_labels[0], "2020-01-01");
        assert_eq!(data.x_labels[1], "2022-01-01");
    }

    #[test]
    fn too_few_points_is_none() {
        assert!(build_chart_data(&[obs(2020, Some(1.0))]).is_none());
        assert!(build_chart_data(&[obs(2020, None), obs(2021, None)]).is_none());
    }

    #[test]
    fn renders_a_frame_without_error() {
        let data = build_chart_data(&[obs(2020, Some(1.0)), obs(2022, Some(3.0))]).unwrap();
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal
            .draw(|frame| render_chart(frame, &data, "GNPCA"))
            .expect("frame renders without error");
    }
}
