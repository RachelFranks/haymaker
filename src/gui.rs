//
// Haymaker
//

use std::{
    error::Error,
    fmt::Result,
    io,
    io::{Stdout, Write},
    process::Stdio,
    sync::{
        atomic::AtomicBool,
        mpsc::{Receiver, Sender},
        Arc,
    },
    thread,
    time::Duration,
};
//use futures::channel::mpsc::Receiver;
use termion::{
    event::Key,
    input::{MouseTerminal, TermRead},
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
};
use tui::{
    backend::TermionBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders},
    Terminal,
};

pub enum Event<T> {
    Input(T),
    State,
    Tick,
}

pub struct Gui {
    terminal: Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>>,
    events: Receiver<Event<Key>>,
    tabs: Vec<Tab>,
}

pub struct Tab {
    title: String,
    width: u16,
    printer: Vec<String>,
}

impl Tab {
    fn new(title: &str) -> Self {
        Tab {
            title: title.to_owned(),
            width: 2 + title.chars().count() as u16,
            printer: vec![],
        }
    }
}

impl Gui {
    pub fn new() -> (Self, Sender<Event<Key>>) {
        let (emit, events) = std::sync::mpsc::channel();
        let done = Arc::new(AtomicBool::new(false));

        let keys_emit = emit.clone();
        let time_emit = emit.clone();
        let keys_done = done.clone();
        let time_done = done;

        thread::spawn(move || {
            let stdin = io::stdin();
            for event in stdin.keys() {
                if let Ok(key) = event {
                    if let Err(err) = keys_emit.send(Event::Input(key)) {
                        panic!("{}", err);
                    }

                    if key == Key::Char('q') {
                        return keys_done.store(true, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            }
        });

        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(250));
            if let Err(err) = time_emit.send(Event::Tick) {
                panic!("{}", err);
            }
            if time_done.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }
        });

        // Terminal initialization
        let stdout = io::stdout().into_raw_mode().unwrap();
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();

        let tabs = vec![Tab::new("$"), Tab::new("!"), Tab::new("≡")];

        let gui = Self {
            terminal,
            events,
            tabs,
        };
        (gui, emit)
    }

    pub fn present(mut self) -> eyre::Result<()> {
        let mut open_tab = 0_usize;

        loop {
            self.terminal.draw(|frame| {
                //

                let screen = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Length(1),       // tabs
                            Constraint::Length(1),       // search
                            Constraint::Percentage(100), // view
                        ]
                        .as_ref(),
                    )
                    .split(frame.size());

                let mut widths: Vec<_> =
                    self.tabs.iter().map(|tab| Constraint::Length(tab.width)).collect();
                widths.push(Constraint::Min(0));
                let tab_bar = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(widths.as_ref())
                    .split(screen[0]);

                for (index, tab) in self.tabs.iter().enumerate() {
                    let color = match index == open_tab {
                        true => Color::Green,
                        false => Color::Blue,
                    };

                    let block = Block::default()
                        .title(vec![Span::from(tab.title.clone())])
                        .title_alignment(Alignment::Center)
                        .style(Style::default().bg(color));
                    frame.render_widget(block, tab_bar[index]);
                }

                let search_hint = "F3 search";

                let search_bar = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Length(1 + search_hint.len() as u16),
                            Constraint::Min(0),
                        ]
                        .as_ref(),
                    )
                    .split(screen[1]);

                let search_hint = Block::default()
                    .title(Span::styled("F3 search", Style::default().fg(Color::DarkGray)))
                    .title_alignment(Alignment::Left);
                frame.render_widget(search_hint, search_bar[0]);

                let block = Block::default()
                    .title("── Console ")
                    .border_style(Style::default().fg(Color::Cyan))
                    .borders(Borders::TOP);
                //.border_type(BorderType::Thick);
                frame.render_widget(block, screen[2]);
            })?;

            match self.events.recv()? {
                Event::Input(key) => {
                    let clip = |mut value: usize| -> usize {
                        if value > self.tabs.len() - 1 {
                            value = self.tabs.len() - 1;
                        }
                        value
                    };

                    match key {
                        Key::Left => open_tab = open_tab.saturating_sub(1),
                        Key::Right => open_tab = clip(open_tab + 1),
                        Key::Char(x @ '1'..='9') => {
                            open_tab = clip(x.to_digit(10).unwrap() as usize - 1)
                        }
                        Key::Char('0') => open_tab = clip(9),
                        Key::Char('q') => return Ok(()),
                        _ => {}
                    }
                }
                Event::State => {
                    println!("state");
                }
                Event::Tick => {
                    println!("tick");
                }
            }
        }
    }
}
