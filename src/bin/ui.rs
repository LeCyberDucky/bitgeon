// TODO: UI should also be state machine. Or should it? Pseudo state machine maybe.
use crossterm::{self, event, ExecutableCommand};

use std::io;
use std::time;

use tui::{
    self,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    style,
    text::{Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget, Wrap},
};

use crate::util;

// Inter-process messages between ui and backend
pub enum UIMessage {
    Event(UIEvent),
}

pub struct ScrollList {
    pub heading: String,
    pub options: Vec<String>,
    pub state: ListState,
}

impl ScrollList {
    pub fn new(heading: String, options: Vec<String>) -> ScrollList {
        ScrollList {
            heading,
            options,
            state: ListState::default(),
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.options.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i as isize - 1).rem_euclid(self.options.len() as isize) as usize,
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub enum UIEvent {
    Selection(usize),
    StateChange(AppState),
}

#[derive(PartialEq)]
pub enum AppState {
    AddFiles,
    End,
    Home,
    Initialization,
}

pub enum Scene {
    AddFiles(SceneAddFiles),
    End,
    Home(SceneHome),
    Initialization,
}

pub struct SceneAddFiles {
    input: Vec<char>,
    file_paths: ScrollList,
}

impl SceneAddFiles {
    pub fn new() -> SceneAddFiles {
        SceneAddFiles {
            input: vec![],
            file_paths: ScrollList::new(String::from(""), vec![]),
        }
    }
}

pub struct SceneHome {
    menu: ScrollList,
}
impl SceneHome {
    pub fn new() -> SceneHome {
        SceneHome {
            menu: ScrollList::new(
                String::from("Choose an option:"),
                vec![
                    String::from("Add files"),
                    String::from("Receive"),
                    String::from("End"),
                ],
            ),
        }
    }
}

pub struct UI {
    pub application: util::Channel<UIMessage>,
    pub application_state: AppState,
    pub scene: Scene,
    pub ui_refresh_rate: u16,
    pub clock: time::Instant,
    pub frame_count: u64,
    pub last_frame: time::Instant,
    pub frame_changed: bool,
}

impl UI {
    pub fn run(application: util::Channel<UIMessage>) {
        // Setup
        let mut ui = UI {
            application,
            application_state: AppState::Initialization,
            scene: Scene::Home(SceneHome::new()),
            ui_refresh_rate: 60,
            clock: time::Instant::now(),
            frame_count: 0,
            last_frame: time::Instant::now(),
            frame_changed: false,
        };

        crossterm::terminal::enable_raw_mode();
        io::stdout().execute(crossterm::terminal::EnterAlternateScreen);
        io::stdout().execute(crossterm::cursor::Hide);
        let mut terminal = tui::Terminal::new(CrosstermBackend::new(io::stdout())).unwrap();

        while std::mem::discriminant(&ui.scene) != std::mem::discriminant(&Scene::End) {
            ui.update();
            if ui.frame_changed {
                // Visual change necessitates redraw
                ui.frame_changed = false;
                ui.draw(&mut terminal);
            }
            ui.interact(); // User interaction

            util::sleep_remaining_frame(&ui.clock, &mut ui.frame_count, &ui.ui_refresh_rate);
        }

        // Reset terminal to initial state
        crossterm::terminal::disable_raw_mode();
        io::stdout().execute(crossterm::terminal::LeaveAlternateScreen);
        io::stdout().execute(crossterm::cursor::Show);
    }

    pub fn draw(&mut self, terminal: &mut tui::Terminal<CrosstermBackend<io::Stdout>>) {
        match self.scene {
            Scene::AddFiles(_) => {
                terminal
                    .draw(|mut f| {
                        // Drag and drop files or directories to this window or type their path
                        let split_horizontal = Layout::default()
                            .direction(Direction::Vertical)
                            .margin(1)
                            .constraints(
                                [Constraint::Percentage(25), Constraint::Percentage(75)].as_ref(),
                            )
                            .split(f.size());

                        let style = style::Style::default();

                        if let Scene::AddFiles(data) = &mut self.scene {
                            let input: String = data.input.iter().collect();
                            let input = Span::raw(input);

                            let input = Paragraph::new(input)
                                .block(Block::default().title("").borders(Borders::NONE))
                                .alignment(Alignment::Left)
                                .wrap(Wrap{trim: true});
                            f.render_widget(input, split_horizontal[0]);

                            let file_paths: Vec<ListItem> = data.file_paths.options.iter().map(|i| ListItem::new(i.as_ref())).collect();
                            let file_paths = List::new(file_paths)
                                .block(Block::default().borders(Borders::ALL).title("Files"))
                                .style(style)
                                .highlight_style(
                                    style
                                        .fg(style::Color::Rgb(253, 3, 166))
                                        .add_modifier(style::Modifier::BOLD),
                                )
                                .highlight_symbol("> ");
                            // f.render_widget(menu_frame, split_vertical[0]);
                            f.render_stateful_widget(file_paths, split_horizontal[1], &mut data.file_paths.state);
                        }

                        let top = Block::default().title("").borders(Borders::ALL);
                        f.render_widget(top, split_horizontal[0]);

                        let bottom = Block::default().title("").borders(Borders::ALL);
                        f.render_widget(bottom, split_horizontal[1]);
                    })
                    .unwrap();
            }

            Scene::Home(_) => {
                terminal
                    .draw(|mut f| {
                        // UI sections
                        let split_horizontal = Layout::default()
                            .direction(Direction::Vertical)
                            .margin(1)
                            .constraints(
                                [Constraint::Percentage(90), Constraint::Percentage(10)].as_ref(),
                            )
                            .split(f.size());

                        let split_vertical = Layout::default()
                            .direction(Direction::Horizontal)
                            .margin(0)
                            .constraints(
                                [Constraint::Percentage(40), Constraint::Percentage(60)].as_ref(),
                            )
                            .split(split_horizontal[0]);

                        let split_horizontal_1 = Layout::default()
                            .direction(Direction::Vertical)
                            .margin(0)
                            .constraints(
                                [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                            )
                            .split(split_vertical[1]);

                        // Widgets
                        // let menu_frame = Block::default().title("Menu").borders(Borders::ALL);

                        let style = style::Style::default();

                        if let Scene::Home(data) = &mut self.scene {
                            let menu: Vec<ListItem> = data.menu.options.iter().map(|i| ListItem::new(i.as_ref())).collect();
                            let menu = List::new(menu)
                                .block(Block::default().borders(Borders::ALL).title("Menu"))
                                .style(style)
                                .highlight_style(
                                    style
                                        .fg(style::Color::Rgb(253, 3, 166))
                                        .add_modifier(style::Modifier::BOLD),
                                )
                                .highlight_symbol("> ");
                            // f.render_widget(menu_frame, split_vertical[0]);
                            f.render_stateful_widget(menu, split_vertical[0], &mut data.menu.state);
                        }

                        let info = Block::default().title("Info").borders(Borders::ALL);
                        f.render_widget(info, split_horizontal[1]);

                        let sending = Block::default().title("Sending").borders(Borders::ALL);
                        f.render_widget(sending, split_horizontal_1[0]);

                        let receiving = Block::default().title("Receiving").borders(Borders::ALL);
                        f.render_widget(receiving, split_horizontal_1[1]);
                    })
                    .unwrap();
            }
            _ => todo!(),
        }
    }

    pub fn interact(&mut self) {
        let mut frame_changed = true;

        if event::poll(time::Duration::from_secs(0)).unwrap() {
            match &mut self.scene {
                Scene::AddFiles(data) => {
                    match event::read().unwrap() {
                        event::Event::Key(event) => match event.code {
                            event::KeyCode::Char(character) => data.input.push(character),
                            _ => todo!(),
                        }
                        _ => frame_changed = self.frame_changed,
                    }
                },

                Scene::Home(data) => {
                    match event::read().unwrap() {
                        event::Event::Key(event) => match event.code {
                            event::KeyCode::Up => data.menu.previous(),
                            event::KeyCode::Down => data.menu.next(),
                            event::KeyCode::Enter => match data.menu.state.selected() {
                                Some(option) => {
                                    self.application
                                        .send(UIMessage::Event(UIEvent::Selection(option)));
                                }
                                None => (),
                            },
                            _ => todo!(),
                        },
                        _ => frame_changed = self.frame_changed, // This is weird some kind of events trigger without user interaction, I guess...
                    }
                }
                _ => todo!(),
            }
        }
        self.frame_changed = frame_changed;
    }

    pub fn update(&mut self) {
        // Get updates from logic of the program. Progress for progress bars for example
        let app_updates = self.application.receive();

        if app_updates.len() > 0 {
            for message in app_updates {
                match message {
                    UIMessage::Event(event) => match event {
                        UIEvent::StateChange(state) => {
                            self.application_state = state;
                            self.scene = match &self.application_state {
                                AppState::AddFiles => Scene::AddFiles(SceneAddFiles::new()),
                                AppState::End => Scene::End,
                                AppState::Home => Scene::Home(SceneHome::new()),
                                AppState::Initialization => todo!(),
                            }
                        }
                        UIEvent::Selection(option) => unimplemented!(),
                    },
                }
            }
            self.frame_changed = true;
        }
    }

    pub fn period_elapsed(&self, count: &u64, rate: &u16) -> bool {
        self.clock.elapsed().as_micros() >= *count as u128 * 1000 / *rate as u128
        // Should we be using floating point values here?
    }

    // pub fn frame_elapsed(&self) -> bool {
    //     // self.period_elapsed(&self.frame_count, &self.ui_refresh_rate)
    //     util::period_elapsed(&self.clock, &self.frame_count, &self.ui_refresh_rate)
    // }
}
