use std::time::{Duration, Instant};

use clap::{Parser, Subcommand};
use misp_interp::Misp;
use ratatui::{
    Frame, Terminal,
    crossterm::{
        event::{self, Event, KeyCode},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Repl,
    Eval { expression: String },
}

#[derive(Default)]
struct App {
    input: String,
    cursor: usize,
    history: Vec<String>,
    misp: Misp,
}

impl App {
    fn insert_char(&mut self, c: char) {
        match c {
            '(' => {
                self.input.insert(self.cursor, '(');
                self.input.insert(self.cursor + 1, ')');
                self.cursor += 1;
            }
            ')' => {
                if self.input.chars().nth(self.cursor) == Some(')') {
                    self.cursor += 1;
                } else {
                    self.input.insert(self.cursor, c);
                    self.cursor += 1;
                }
            }
            _ => {
                self.input.insert(self.cursor, c);
                self.cursor += 1;
            }
        }
    }

    fn delete_char(&mut self) {
        if self.cursor > 0 {
            if self.cursor > 0 && self.cursor < self.input.len() {
                let prev_char = self.input.chars().nth(self.cursor - 1);
                let next_char = self.input.chars().nth(self.cursor);

                if prev_char == Some('(') && next_char == Some(')') {
                    self.input.remove(self.cursor);
                    self.input.remove(self.cursor - 1);
                    self.cursor -= 1;
                    return;
                }
            }

            self.input.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    fn execute_input(&mut self) {
        let line = self.input.trim().to_string();
        if line.is_empty() {
            return;
        }

        self.history.push(format!("misp >> {}", line));

        match self.misp.eval(&line) {
            Ok(value) => self.history.push(Misp::print(&value)),
            Err(err) => self.history.push(format!("{}", err)),
        }

        self.input.clear();
        self.cursor = 0;
    }

    fn render(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .direction(Direction::Vertical)
            .split(f.area());

        const MAX_HISTORY: usize = 1000;
        if self.history.len() > MAX_HISTORY {
            self.history.drain(0..self.history.len() - MAX_HISTORY);
        }

        let content_height = self.history.len() as u16;
        let visible_height = chunks[0].height.saturating_sub(2);
        let scroll_offset = content_height.saturating_sub(visible_height);

        let history = Paragraph::new(self.history.join("\n"))
            .block(Block::bordered().title("misp repl"))
            .scroll((scroll_offset, 0));

        f.render_widget(&history, chunks[0]);

        let input = Paragraph::new(format!("misp >> {}", self.input))
            .block(Block::default().borders(Borders::ALL).title("input"));
        f.render_widget(input, chunks[1]);

        f.set_cursor_position((chunks[1].x + (self.cursor as u16) + 9, chunks[1].y + 1));
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App, tick_rate: Duration) {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|frame| app.render(frame)).unwrap();

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if !event::poll(timeout).unwrap() {
            last_tick = Instant::now();
            continue;
        }

        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                KeyCode::Enter => app.execute_input(),
                KeyCode::Backspace => app.delete_char(),
                KeyCode::Char(c) => app.insert_char(c),
                KeyCode::Left => {
                    if app.cursor > 0 {
                        app.cursor -= 1;
                    }
                }
                KeyCode::Right => {
                    if app.cursor < app.input.len() {
                        app.cursor += 1;
                    }
                }
                KeyCode::Esc => break,
                _ => {}
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Repl => {
            enable_raw_mode().unwrap();
            let mut stdout = std::io::stdout();
            execute!(stdout, EnterAlternateScreen).unwrap();
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend).unwrap();

            let app = App::default();
            run_app(&mut terminal, app, Duration::from_millis(100));

            disable_raw_mode().unwrap();
            execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
            terminal.show_cursor().unwrap();
        }
        Commands::Eval { expression } => {
            let mut misp = Misp::default();

            match misp.eval_script(&expression) {
                Ok(value) => println!(
                    "{}",
                    value
                        .iter()
                        .map(Misp::print)
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
                Err(err) => eprintln!("Error: {}", err),
            }
        }
    }
}
