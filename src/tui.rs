use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, LegendPosition, Paragraph, Tabs},
    Terminal,
};
use sqlx::{Pool, Sqlite};
use std::{
    cmp::Ordering,
    collections::HashMap,
    io,
    time::{Duration, Instant},
};

use crate::{
    constants::{
        DOWN_SAMPLE_POINTS, DO_INTERPOLATION, INTERPOLATION_STEPS
    },
    stats_handling::{
        conversions::{format_bytes, format_time, get_byte_unit, get_time_unit, Unit},
        database::{self, get_device_stats_after},
        device_info::Device,
    },
};

#[derive(Clone, Copy)]
enum TimeRange {
    Last1Hour,
    Last1Day,
    Last1Week,
    Last1Month,
    Last1Year,
}

impl TimeRange {
    fn all() -> Vec<TimeRange> {
        use TimeRange::*;
        vec![Last1Hour, Last1Day, Last1Week, Last1Month, Last1Year]
    }

    fn as_str(&self) -> &'static str {
        match self {
            TimeRange::Last1Hour => "1 hour",
            TimeRange::Last1Day => "1 day",
            TimeRange::Last1Week => "1 week",
            TimeRange::Last1Month => "1 month",
            TimeRange::Last1Year => "1 year",
        }
    }

    fn duration_secs(&self) -> i64 {
        match self {
            TimeRange::Last1Hour => 60 * 60,
            TimeRange::Last1Day => 24 * 60 * 60,
            TimeRange::Last1Week => 7 * 24 * 60 * 60,
            TimeRange::Last1Month => 30 * 24 * 60 * 60,
            TimeRange::Last1Year => 365 * 24 * 60 * 60,
        }
    }
}

struct App {
    device_names: Vec<String>,
    device_ids: Vec<String>,
    selected_device: usize,
    time_range_index: usize,
    metrics_cache: HashMap<String, Vec<Device>>,
    last_updated: Instant,
}

impl App {
    // Return device_id for the currently selected device tab
    fn selected_device_id(&self) -> Option<&str> {
        self.device_ids
            .get(self.selected_device)
            .map(|s| s.as_str())
    }

    fn selected_time_range(&self) -> TimeRange {
        TimeRange::all()[self.time_range_index]
    }

    async fn refresh_data(&mut self, database: &Pool<Sqlite>) {
        if let Some(device_id) = self.selected_device_id() {
            let since = chrono::Utc::now().timestamp() - self.selected_time_range().duration_secs();
            let data = get_device_stats_after(&database, device_id, since).await;

            self.metrics_cache.insert(device_id.to_string(), data);
        }
        self.last_updated = Instant::now();
    }

    fn make_chart<'a>(
        &self,
        data: &'a [(f64, f64)],
        unit: Unit,
        time: i64,
        min: f64,
        max: f64,
        title: &'a str,
        color: Color,
    ) -> Chart<'a> {
        // Chart maker
        let mut chart = Chart::new(vec![Dataset::default()
            .name(format!("{title} Used"))
            .marker(symbols::Marker::Dot)
            .style(Style::default().fg(color))
            .data(&data)])
        .legend_position(Some(LegendPosition::TopLeft))
        .block(
            Block::default()
                .title(format!("{title} Usage"))
                .borders(Borders::ALL),
        );

        chart = self.detail_chart(chart, unit, time, min, max);

        chart
    }

    fn detail_chart<'a>(&self, chart: Chart<'a>, unit: Unit, time: i64, min: f64, max: f64) -> Chart<'a> {
        let time = time as u128;

        chart
            .x_axis(
                Axis::default()
                    .title("Time")
                    .bounds([0.0, time as f64])
                    .labels(vec![
                        Span::raw(format!(
                            "{:.1} {}",
                            format_time(time, Unit::SECOND),
                            get_time_unit(time, Unit::SECOND)
                        )),
                        Span::raw(format!(
                            "{:.1} {}",
                            format_time(time / 2, Unit::SECOND),
                            get_time_unit(time / 2, Unit::SECOND)
                        )),
                        Span::raw(format!(
                            "{} {}",
                            format_time(0, Unit::SECOND),
                            get_time_unit(0, Unit::SECOND)
                        )),
                    ])
                    .style(Style::default().fg(Color::Gray)),
            )
            .y_axis(
                Axis::default()
                    .bounds([min, max])
                    .labels(vec![
                        Span::raw(format!("{:.2}{unit}", min)),
                        Span::raw(format!("{:.2}{unit}", (max-min) * 0.25 + min)),
                        Span::raw(format!("{:.2}{unit}", (max-min) * 0.5 + min)),
                        Span::raw(format!("{:.2}{unit}", (max-min) * 0.75 + min)),
                        Span::raw(format!("{:.2}{unit}", max)),
                    ])
                    .style(Style::default().fg(Color::Gray)),
            )
    }
}

pub async fn start_tui(database: &Pool<Sqlite>) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut device_names: Vec<String> = Vec::new();
    let mut device_ids: Vec<String> = Vec::new();

    // Load device IDs and names from DB
    for device_id in database::get_all_device_uids(&database).await.iter() {
        let device_name = database::get_device_name_from_uid(&database, device_id).await;
        device_names.push(device_name);
        device_ids.push(device_id.clone());
    }

    device_bubble_sort(&mut device_names, &mut device_ids);

    let mut app = App {
        device_names,
        device_ids,
        selected_device: 0,
        time_range_index: 0,
        metrics_cache: HashMap::new(),
        last_updated: Instant::now() - Duration::from_secs(999),
    };

    let tick_rate = Duration::from_millis(200);

    loop {
        if app.last_updated.elapsed().as_secs() > 10 {
            app.refresh_data(&database).await;

            let mut device_names: Vec<String> = Vec::new();
            let mut device_ids: Vec<String> = Vec::new();

            for device_id in database::get_all_device_uids(&database).await.iter() {
                let device_name = database::get_device_name_from_uid(&database, device_id).await;
                device_names.push(device_name);
                device_ids.push(device_id.clone());
            }

            device_bubble_sort(&mut device_names, &mut device_ids);

            app.device_names = device_names;
            app.device_ids = device_ids;
        }

        terminal.draw(|f| {
            let size = f.area();

            // Different areas to make
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(size);

            let titles: Vec<Span> = app
                .device_names
                .iter()
                .map(|d| Span::from(Span::raw(d)))
                .collect();
            let tabs = Tabs::new(titles)
                .select(app.selected_device)
                .block(Block::default().title("Devices").borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Yellow));
            f.render_widget(tabs, chunks[0]);

            // Gets the time range as a str
            let time_range = TimeRange::all()[app.time_range_index].as_str();

            // Allows the user to cycle through how much data should be shown
            let time_range_selector = Paragraph::new(Span::styled(
                format!("[ {} ▲ ▼ ]", time_range),
                Style::default().fg(Color::Green),
            ));
            f.render_widget(time_range_selector, chunks[1]);

            if let Some(device_id) = app.selected_device_id() {
                if let Some(data) = app.metrics_cache.get(device_id) {
                    let now = chrono::Utc::now().timestamp();
                    let time_min = now - app.selected_time_range().duration_secs();
                    let unfiltered_data: Vec<_> = data.iter().filter(|d| d.time >= time_min).collect();

                    let graph_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage(33),
                            Constraint::Percentage(33),
                            Constraint::Percentage(34),
                        ])
                        .split(chunks[2]);

                    // Makes all the datasets
                    let data = make_dataset(time_min, data);

                    // Assigns the different vectors to their respective values
                    let cpu_data = data.get(0).unwrap();
                    let ram_data = data.get(1).unwrap();
                    let network_in_data = data.get(2).unwrap();
                    let network_out_data = data.get(3).unwrap();

                    let duration = app.selected_time_range().duration_secs();

                    let (cpu_min, cpu_max) = find_min_max(&cpu_data);

                    // let cpu_data = down_sample(&interpolate(&cpu_data, 4), 10);
                    let cpu_data = down_sample(&cpu_data, 40);

                    let cpu_chart = app.make_chart(
                        &cpu_data,
                        Unit::Percentage,
                        duration,
                        cpu_min,
                        cpu_max,
                        "CPU",
                        Color::Green,
                    );

                    f.render_widget(cpu_chart, graph_chunks[0]);

                    // RAM Chart

                    // Gets the largest value from the vector
                    let (ram_min, ram_max) = find_min_max(&ram_data); 
                    // Makes RAM the chart
                    let ram_unit = get_byte_unit(
                        unfiltered_data
                            .iter()
                            .map(|d| d.ram_used)
                            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
                            .unwrap_or(0) as usize,
                        Unit::BYTE,
                    );

                    let ram_chart = app.make_chart(
                        &ram_data,
                        ram_unit.clone(),
                        duration,
                        ram_min,
                        ram_max,
                        "RAM",
                        Color::Red,
                    );

                    f.render_widget(ram_chart, graph_chunks[1]);

                    // Network Chart
                    let (network_in_min, network_in_max) = find_min_max(&network_in_data);

                    // Get the network in max from the unfiltered data
                    let network_in_unit = get_byte_unit(
                        unfiltered_data
                            .iter()
                            .map(|d| d.network_in)
                            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
                            .unwrap_or(0) as usize,
                        Unit::BYTE);

                    let (network_out_min, network_out_max) = find_min_max(&network_out_data);

                    // Get the network out max from the unfiltered data
                    let network_out_unit = get_byte_unit(
                        unfiltered_data
                            .iter()
                            .map(|d| d.network_out)
                            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
                            .unwrap_or(0) as usize,
                        Unit::BYTE);

                    let (network_min, network_max, network_unit): (f64, f64, Unit) = {
                        let min = network_in_min.min(network_out_min);

                        if network_in_max > network_out_max {
                            (min, network_in_max, network_in_unit)
                        } else {
                            (min, network_out_max, network_out_unit)
                        }
                    };

                    let mut network_chart = Chart::new(vec![
                        Dataset::default()
                            .name("Network In")
                            .marker(symbols::Marker::Dot)
                            .style(Style::default().fg(Color::Magenta))
                            .data(&network_in_data),
                        Dataset::default()
                            .name("Network Out")
                            .marker(symbols::Marker::Dot)
                            .style(Style::default().fg(Color::Cyan))
                            .data(&network_out_data),
                    ])
                    .legend_position(Some(LegendPosition::TopLeft))
                    .hidden_legend_constraints((
                        Constraint::Percentage(50),
                        Constraint::Percentage(60),
                    ))
                    .block(
                        Block::default()
                            .title("Network Usage")
                            .borders(Borders::ALL),
                    );

                    network_chart = app.detail_chart(
                        network_chart,
                        network_unit.clone(),
                        duration,
                        network_min,
                        network_max,
                    );

                    f.render_widget(network_chart, graph_chunks[2]);
                }
            }
        })?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Tab => {
                        app.selected_device = (app.selected_device + 1) % app.device_names.len();
                        app.refresh_data(&database).await;
                    }
                    KeyCode::BackTab => {
                        if app.selected_device == 0 {
                            app.selected_device = app.device_names.len() - 1;
                        } else {
                            app.selected_device -= 1;
                        }
                        app.refresh_data(&database).await;
                    }
                    KeyCode::Left => {
                        if app.selected_device == 0 {
                            app.selected_device = app.device_names.len() - 1;
                        } else {
                            app.selected_device -= 1;
                        }
                        app.refresh_data(&database).await
                    }
                    KeyCode::Right => {
                        if app.selected_device == app.device_names.len() - 1 {
                            app.selected_device = 0;
                        } else {
                            app.selected_device += 1;
                        }
                        app.refresh_data(&database).await
                    }
                    KeyCode::Up => {
                        app.time_range_index = (app.time_range_index + 1) % TimeRange::all().len();
                        app.refresh_data(&database).await;
                    }
                    KeyCode::Down => {
                        if app.time_range_index == 0 {
                            app.time_range_index = TimeRange::all().len() - 1;
                        } else {
                            app.time_range_index -= 1;
                        }
                        app.refresh_data(&database).await;
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn device_bubble_sort(device_names: &mut Vec<String>, device_ids: &mut Vec<String>) {
    let n = device_names.len();
    for i in 0..n {
        for j in 0..n - 1 - i {
            if device_names[j] > device_names[j + 1] {
                device_names.swap(j, j + 1);
                device_ids.swap(j, j + 1);
            }
        }
    }
}

pub fn make_dataset(time_min: i64, data: &Vec<Device>) -> Vec<Vec<(f64, f64)>> {
    let filtered: Vec<_> = data.iter().filter(|d| d.time >= time_min).collect();

    let mut datasets: Vec<Vec<(f64, f64)>> = vec![Vec::new(), Vec::new(), Vec::new(), Vec::new()];

    // Take the data and make it usable on the charts
    // CPU usage
    let cpu_data: Vec<(f64, f64)> = filtered
        .iter()
        .map(|d| ((d.time - time_min) as f64, (d.cpu_usage * 100.0) as f64))
        .collect();

    // Ram Usage
    let ram_data: Vec<(f64, f64)> = filtered
        .iter()
        .map(|d| {
            (
                (d.time - time_min) as f64,
                format_bytes(d.ram_used as f64, Unit::BYTE),
            )
        })
        .collect();

    // Network In
    let network_in_data: Vec<(f64, f64)> = filtered
        .iter()
        .map(|d| {
            (
                (d.time - time_min) as f64,
                format_bytes(d.network_in as f64, Unit::BYTE),
            )
        })
        .collect();

    // Network Out
    let network_out_data: Vec<(f64, f64)> = filtered
        .iter()
        .map(|d| {
            (
                (d.time - time_min) as f64,
                format_bytes(d.network_out as f64, Unit::BYTE),
            )
        })
        .collect();

    datasets[0] = filter(&cpu_data, DO_INTERPOLATION);
    datasets[1] = filter(&ram_data, DO_INTERPOLATION);
    datasets[2] = filter(&network_in_data, DO_INTERPOLATION);
    datasets[3] = filter(&network_out_data, DO_INTERPOLATION);

    datasets
}

fn down_sample(data: &[(f64, f64)], target_points: u16) -> Vec<(f64, f64)> {
    if target_points == 0 || data.is_empty() {
        return vec![];
    }

    let chunk_size = (data.len() as f64 / target_points as f64).ceil() as usize;

    data.chunks(chunk_size)
        .map(|chunk| {
            let avg_x = chunk.iter().map(|(x, _)| x).sum::<f64>() / chunk.len() as f64;
            let avg_y = chunk.iter().map(|(_, y)| y).sum::<f64>() / chunk.len() as f64;
            (avg_x, avg_y)
        })
        .collect()
}

fn interpolate(data: &[(f64, f64)], steps_per_segment: u16) -> Vec<(f64, f64)> {
    let mut interpolated = Vec::new();

    for window in data.windows(2) {
        let (x0, y0) = window[0];
        let (x1, y1) = window[1];

        for step in 0..steps_per_segment {
            let t = step as f64 / steps_per_segment as f64;
            let x = x0 + t * (x1 - x0);
            let y = y0 + t * (y1 - y0);
            interpolated.push((x, y));
        }
    }

    // Optionally push the last point
    if let Some(&last) = data.last() {
        interpolated.push(last);
    }

    interpolated
}

fn _remove_outliers(data: &[(f64, f64)], threshold: f64) -> Vec<(f64, f64)> {
    let ys: Vec<f64> = data.iter().map(|(_, y)| *y).collect();

    let mean = ys.iter().copied().sum::<f64>() / ys.len() as f64;
    let std_dev = (ys.iter().map(|y| (y - mean).powi(2)).sum::<f64>() / ys.len() as f64).sqrt();

    data.iter()
        .copied()
        .filter(|(_, y)| {
            let z = (y - mean).abs() / std_dev;
            z <= threshold
        })
        .collect()
}

fn filter(data: &[(f64, f64)], do_interpolation: bool) -> Vec<(f64, f64)> {
    // let data = remove_outliers(&data, OUTLIER_THRESHOLD);

    let data = down_sample(&data, DOWN_SAMPLE_POINTS);

    if do_interpolation {
        interpolate(&data, INTERPOLATION_STEPS)
    } else {
        data
    }
}

fn find_min_max(data: &[(f64, f64)]) -> (f64, f64) {
    if data.is_empty() {
        return (0.0, 0.0);
    }


    let mut min: f64 = f64::MAX;
    // f64::MIN isn't used here as it leads to that value being shown when do data is present
    let mut max: f64 = 0.0;

    for d in data {
        if d.1 != 0.0 {
            min = min.min(d.1)
        }
    }

    for d in data {
        max = max.max(d.1)
    }


    (min, max)
}
