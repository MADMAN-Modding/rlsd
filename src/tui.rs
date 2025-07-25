use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Tabs},
};
use sqlx::{Pool, Sqlite};
use std::{
    collections::HashMap,
    io,
    time::{Duration, Instant},
};

use crate::{constants::conversions, stats_handling::{
    database::{self, get_device_stats_after},
    device_info::Device,
}};

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
        }

        terminal.draw(|f| {
            let size = f.area();

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

            let time_range = TimeRange::all()[app.time_range_index].as_str();
            let dropdown = Paragraph::new(Span::styled(
                format!("[ {} ▲ ▼ ]", time_range),
                Style::default().fg(Color::Green),
            ));
            f.render_widget(dropdown, chunks[1]);

            if let Some(device_id) = app.selected_device_id() {
                if let Some(data) = app.metrics_cache.get(device_id) {
                    let now = chrono::Utc::now().timestamp();
                    let time_min = now - app.selected_time_range().duration_secs();
                    let filtered: Vec<_> = data.iter().filter(|d| d.time >= time_min).collect();

                    let cpu_data: Vec<(f64, f64)> = filtered
                        .iter()
                        .map(|d| ((d.time - time_min) as f64, d.cpu_usage as f64))
                        .collect();
                    let ram_data: Vec<(f64, f64)> = filtered
                        .iter()
                        .map(|d| ((d.time - time_min) as f64, d.ram_used as f64 / 1e9))
                        .collect(); // GB
                    // let network_in_data: Vec<(f64, f64)> = filtered
                    //     .iter()
                    //     .map(|d| ((d.time - time_min) as f64, d.network_in as f64))
                    //     .collect();
                    // let network_out_data: Vec<(f64, f64)> = filtered
                    //     .iter()
                    //     .map(|d| ((d.time - time_min) as f64, d.network_out as f64))
                    //     .collect();

                    let ram_total = filtered.last().map_or(1.0, |d| d.ram_total as f64 / conversions::byte::GIBIBYTE);

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
                    let cpu_chart = Chart::new(vec![
                        Dataset::default()
                            .name("CPU Usage")
                            .marker(symbols::Marker::Dot)
                            .style(Style::default().fg(Color::Green))
                            .data(&cpu_data),
                    ])
                    .block(
                        Block::default()
                            .title("CPU Usage (%)")
                            .borders(Borders::ALL),
                    )
                    .x_axis(
                        Axis::default()
                            .title("Time (seconds ago)")
                            .bounds([0.0, duration as f64])
                            .labels(vec![
                                Span::raw("0s"),
                                Span::raw(format!("{}s", duration / 2)),
                                Span::raw(format!("{}s", duration)),
                            ])
                            .style(Style::default().fg(Color::Gray)),
                    )
                    .y_axis(
                        Axis::default()
                            .title("CPU %")
                            .bounds([0.0, 1.0])
                            .labels(vec![Span::raw("0%"), Span::raw("50%"), Span::raw("100%")])
                            .style(Style::default().fg(Color::Gray)),
                    );

                    f.render_widget(cpu_chart, graph_chunks[0]);

                    // RAM Chart
                    let ram_line = [(time_min as f64, ram_total), (now as f64, ram_total)];
                    let ram_chart = Chart::new(vec![
                        Dataset::default()
                            .name("RAM Used")
                            .marker(symbols::Marker::Dot)
                            .style(Style::default().fg(Color::Cyan))
                            .data(&ram_data),
                        Dataset::default()
                            .name("RAM Total")
                            .graph_type(GraphType::Line)
                            .style(Style::default().fg(Color::DarkGray))
                            .data(&ram_line),
                    ])
                    .block(Block::default().title("RAM (GB)").borders(Borders::ALL))
                    .x_axis(
                        Axis::default()
                            .title("Time")
                            .bounds([0.0, duration as f64])
                            .labels(vec![
                                Span::raw("0s"),
                                Span::raw(format!("{}s", duration / 2)),
                                Span::raw(format!("{}s", duration)),
                            ])
                            .style(Style::default().fg(Color::Gray)),
                    )
                    .y_axis(
                        Axis::default()
                            .bounds([0.0, ram_total.max(0.1)])
                            .labels(vec![
                                Span::raw("0 GiB"), Span::raw(format!("{:.2} GiB", ram_total / 2.0)), Span::raw(format!("{:.2} GiB", ram_total))
                            ])
                            .style(Style::default().fg(Color::Gray)),
                    );

                    f.render_widget(ram_chart, graph_chunks[1]);

                    // Network Chart (need to fix the way data is obtained before this is used)
                    // let max_net = network_in_data
                    //     .iter()
                    //     .chain(&network_out_data)
                    //     .map(|(_, v)| *v)
                    //     .fold(0.0_f64, f64::max)
                    //     .max(1.0);
                    // let net_chart = Chart::new(vec![
                    //     Dataset::default()
                    //         .name("Net In")
                    //         .marker(symbols::Marker::Braille)
                    //         .style(Style::default().fg(Color::Blue))
                    //         .data(&network_in_data),
                    //     Dataset::default()
                    //         .name("Net Out")
                    //         .marker(symbols::Marker::Braille)
                    //         .style(Style::default().fg(Color::Red))
                    //         .data(&network_out_data),
                    // ])
                    // .block(
                    //     Block::default()
                    //         .title("Network I/O (bytes)")
                    //         .borders(Borders::ALL),
                    // )
                    // .x_axis(
                    //     Axis::default()
                    //         .title("Time")
                    //         .bounds([0.0, duration as f64])
                    //         .labels(vec![
                    //             Span::raw("0s"),
                    //             Span::raw(format!("{}s", duration / 2)),
                    //             Span::raw(format!("{}s", duration)),
                    //         ])
                    //         .style(Style::default().fg(Color::Gray)),
                    // )
                    // .y_axis(
                    //     Axis::default()
                    //         .bounds([0.0, max_net])
                    //         .style(Style::default().fg(Color::Gray)),
                    // );

                    // f.render_widget(net_chart, graph_chunks[2]);
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
