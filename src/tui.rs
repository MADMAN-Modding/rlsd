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

use crate::stats_handling::{
        conversions::{format_bytes, get_unit, Unit}, database::{self, get_device_stats_after}, device_info::Device
    };

#[derive(Clone, Copy)]
enum TimeRange {
    Last30Min,
    Last1Hour,
    Last1Day,
    Last1Week,
    Last1Month,
    Last1Year,
}

impl TimeRange {
    fn all() -> Vec<TimeRange> {
        use TimeRange::*;
        vec![
            Last30Min, Last1Hour, Last1Day, Last1Week, Last1Month, Last1Year,
        ]
    }

    fn as_str(&self) -> &'static str {
        match self {
            TimeRange::Last30Min => "30 min",
            TimeRange::Last1Hour => "1 hour",
            TimeRange::Last1Day => "1 day",
            TimeRange::Last1Week => "1 week",
            TimeRange::Last1Month => "1 month",
            TimeRange::Last1Year => "1 year",
        }
    }

    fn duration_secs(&self) -> i64 {
        match self {
            TimeRange::Last30Min => 30 * 60,
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
        limit: f64,
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

        chart = self.detail_chart(chart, unit, time, limit);

        chart
    }

    fn detail_chart<'a>(
        &self,
        chart: Chart<'a>,
        unit: Unit,
        time: i64,
        limit: f64,
    ) -> Chart<'a> {
        chart
            .x_axis(
                Axis::default()
                    .title("Time")
                    .bounds([0.0, time as f64])
                    .labels(vec![
                        Span::raw(format!("{}s", time)),
                        Span::raw(format!("{}s", time / 2)),
                        Span::raw(format!("{}s", 0)),
                    ])
                    .style(Style::default().fg(Color::Gray)),
            )
            .y_axis(
                Axis::default()
                    .bounds([0.0, limit])
                    .labels(vec![
                        Span::raw(format!("0{unit}")),
                        Span::raw(format!("{:.2}{unit}", limit / 2.0)),
                        Span::raw(format!("{:.2}{unit}", limit)),
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
                    let filtered: Vec<_> = data.iter().filter(|d| d.time >= time_min).collect();

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
                        .map(|d| ((d.time - time_min) as f64, format_bytes(d.network_in as f64, Unit::BYTE)))
                        .collect();

                    // Network Out
                    let network_out_data: Vec<(f64, f64)> = filtered
                        .iter()
                        .map(|d| ((d.time - time_min) as f64, format_bytes(d.network_out as f64, Unit::BYTE)))
                        .collect();

                    let graph_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage(33),
                            Constraint::Percentage(33),
                            Constraint::Percentage(34),
                        ])
                        .split(chunks[2]);

                    let duration = app.selected_time_range().duration_secs();

                    // CPU Chart
                    let cpu_chart =
                        app.make_chart(&cpu_data, Unit::Percentage, duration, 100.0, "CPU", Color::Green);

                    f.render_widget(cpu_chart, graph_chunks[0]);

                    // RAM Chart

                    // Gets the largest value from the vector
                    let ram_total = filtered
                        .iter()
                        .map(|d| d.ram_total as f64)
                        .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
                        .unwrap_or(0.0);

                    // Makes RAM the chart
                    let ram_unit = get_unit(ram_total as usize, Unit::BYTE);

                    let ram_chart =
                        app.make_chart(&ram_data, ram_unit.clone(), duration, ram_total / ram_unit.to_f64(), "RAM", Color::Red);

                    f.render_widget(ram_chart, graph_chunks[1]);

                    // Network Chart
                    let network_in_max = filtered
                        .iter()
                        .map(|d| d.network_in as f64)
                        .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)).unwrap_or(0.0);

                    let network_in_unit = get_unit(network_in_max as usize, Unit::BYTE);

                    let network_out_max = filtered
                        .iter()
                        .map(|d| d.network_out as f64)
                        .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)).unwrap_or(0.0);

                    let network_out_unit = get_unit(network_out_max as usize, Unit::BYTE);

                    let (network_max, network_unit): (f64, Unit) = if network_in_max > network_out_max {
                        (network_in_max, network_in_unit)
                    } else {
                        (network_out_max, network_out_unit)
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

                    network_chart = app.detail_chart(network_chart, network_unit.clone(), duration, network_max / network_unit.to_f64());

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
