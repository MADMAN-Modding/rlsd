use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{buffer::Buffer, layout::Rect, style::Stylize, symbols::border, text::{Line, Text}, widgets::{Block, Paragraph, Widget}, DefaultTerminal, Frame};

#[derive(Debug, Default)]
pub struct App {
    exit: bool
}

impl App {
    pub fn setup() -> io::Result<()> {
        let mut terminal = ratatui::init();

        let app_result = App::default().run(&mut terminal);
        app_result 
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while  !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }, 
            _ => {}
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    } 
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("R.L.S.D.").bold();

        let instructions = Line::from(vec!["Bottom Info......".into()]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        Paragraph::new(Text::from("test")).centered().block(block).render(area, buf);
    }
}