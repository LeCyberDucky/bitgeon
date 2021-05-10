use std::io;
use std::net::IpAddr;
use std::time;

use anyhow::{self, Result};
use crossterm::{self, event::KeyCode, ExecutableCommand};
use tui::{self, backend::CrosstermBackend};

use crate::backend;
use crate::server;
use crate::util;
use crate::widget;
use scene::Scene;
use widget::StyledPathList;

pub trait Data {}
pub trait Event {}

#[derive(Clone)]
pub enum Message<D, E>
where
    // Bounds probably not necessary, but I want to using them. Also, they might make things look a bit nicer
    D: Data,
    E: Event,
{
    Data(D),
    Event(E),
}

pub mod data {
    use super::*;

    #[derive(Clone)]
    pub enum Backend {
        FilePathList(StyledPathList),
    }

    #[derive(Clone)]
    pub enum Server {
        ConnectionInfo {
            public_ip: Option<IpAddr>,
            external_port: Option<u16>,
            // status: ServerStatus,
            secret_key: String,
        },
    }

    impl Data for Backend {}
    impl Data for Server {}
}

pub mod event {
    use super::*;

    #[derive(Clone)]
    pub enum Backend {
        StateChange(AppState),
    }

    #[derive(Clone)]
    pub enum Server {}

    impl Event for Backend {}
    impl Event for Server {}
}

// // Inter-process messages between ui and backend
// #[derive(Clone)]
// pub enum Message {
//     Data(Data),
//     Event(Event),
// }

// // TODO: Data should only be data that is of use for the UI
// #[derive(Clone)]
// pub enum Data {
//     FilePathList(StyledPathList),
//     ConnectionInfo {
//         public_ip: Option<IpAddr>,
//         external_port: Option<u16>,
//         // status: ServerStatus,
//         secret_key: String,
//     },
// }

// // TODO: Events should only be events that the UI can react to
// #[derive(Clone)]
// pub enum Event {
//     Selection(usize),
//     StateChange(AppState),
// }

#[derive(Clone)]
pub enum AppState {
    EditFiles(StyledPathList),
    End,
    Home(String),
    Initialization,
}

pub struct Ui {
    pub application: util::ThreadChannel<
        backend::Message<backend::data::Ui, backend::event::Ui>,
        Message<data::Backend, event::Backend>,
    >,
    pub server: util::ThreadChannel<
        server::Message<server::data::Ui, server::event::Ui>,
        Message<data::Server, event::Server>,
    >,
    pub application_state: AppState,
    pub scene: Scene,
    pub ui_refresh_rate: u128,
    pub clock: time::Instant,
    pub frame_count: u128,
    pub last_frame: time::Instant,
    pub frame_changed: bool,
}

impl Ui {
    pub fn new(
        application: util::ThreadChannel<
            backend::Message<backend::data::Ui, backend::event::Ui>,
            Message<data::Backend, event::Backend>,
        >,
        server: util::ThreadChannel<
            server::Message<server::data::Ui, server::event::Ui>,
            Message<data::Server, event::Server>,
        >,
    ) -> Self {
        Self {
            application,
            server,
            application_state: AppState::Initialization,
            scene: Scene::Home(scene::Home::new(String::from(""))),
            ui_refresh_rate: 60,
            clock: time::Instant::now(),
            frame_count: 0,
            last_frame: time::Instant::now(),
            frame_changed: true,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        io::stdout().execute(crossterm::terminal::EnableLineWrap)?;
        io::stdout().execute(crossterm::terminal::EnterAlternateScreen)?;
        io::stdout().execute(crossterm::cursor::Hide)?;
        let mut terminal = tui::Terminal::new(CrosstermBackend::new(io::stdout()))?;

        // TODO: Figure out what is happening here and document that in a comment
        while std::mem::discriminant(&self.scene) != std::mem::discriminant(&Scene::End) {
            self.update();
            if self.frame_changed {
                // Visual change necessitates redraw
                self.frame_changed = false;
                self.draw(&mut terminal)?;
            }
            self.interact()?; // User interaction

            util::sleep_remaining_frame(&self.clock, &mut self.frame_count, self.ui_refresh_rate);
        }

        // Reset terminal to initial state
        crossterm::terminal::disable_raw_mode()?;
        io::stdout().execute(crossterm::terminal::LeaveAlternateScreen)?;
        io::stdout().execute(crossterm::cursor::Show)?;
        Ok(())
    }

    pub fn draw(
        &mut self,
        terminal: &mut tui::Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<()> {
        match &mut self.scene {
            Scene::EditFiles(scene) => scene.draw(terminal),
            Scene::Home(scene) => scene.draw(terminal),
            _ => {
                todo!();
                Ok(())
            }
        }
    }

    pub fn interact(&mut self) -> Result<()> {
        if crossterm::event::poll(time::Duration::from_secs(0))? {
            let event = crossterm::event::read()?;
            match event {
                crossterm::event::Event::Key(key) => match key.code {
                    KeyCode::Backspace
                    | KeyCode::Char(_)
                    | KeyCode::Delete
                    | KeyCode::Down
                    | KeyCode::Enter
                    | KeyCode::Up
                    | KeyCode::Esc => {
                        let message;
                        match &mut self.scene {
                            Scene::EditFiles(scene) => message = scene.interact(event)?,
                            Scene::Home(scene) => message = scene.interact(event)?,
                            _ => {
                                todo!();
                            }
                        }

                        // let message = self.scene.interact(event)?;
                        if let Some(message) = message {
                            self.application.send(message)?;
                        }
                        self.frame_changed = true;
                    }
                    _ => (),
                },
                _ => (),
            }
        }
        Ok(())
    }

    pub fn update(&mut self) {
        // Get updates from logic of the program. Progress for progress bars for example
        let app_updates = self.application.receive();

        if !app_updates.is_empty() {
            for message in app_updates {
                match message {
                    Message::Event(event) => match event {
                        event::Backend::StateChange(state) => {
                            self.application_state = state;
                            self.scene = match &self.application_state {
                                AppState::EditFiles(file_list) => {
                                    Scene::EditFiles(scene::EditFiles::new(file_list.to_owned()))
                                }
                                AppState::End => Scene::End,
                                AppState::Home(connection_info) => {
                                    Scene::Home(scene::Home::new(connection_info.to_owned()))
                                }
                                AppState::Initialization => todo!(),
                            }
                        }
                    },
                    _ => todo!(),
                }
            }
            self.frame_changed = true;
        }
    }

    pub fn period_elapsed(&self, count: &u64, rate: &u16) -> bool {
        self.clock.elapsed().as_micros() >= *count as u128 * 1000 / *rate as u128
        // Should we be using floating point values here?
    }
}

mod scene {
    // TODO: Import super::*;
    use std::io;

    use anyhow::{self, Result};
    use crossterm::{self, event::KeyCode};
    use tui::{
        self,
        backend::CrosstermBackend,
        layout::{Constraint, Direction, Layout},
        style,
        widgets::{Block, Borders, List, ListItem},
    };

    use crate::backend;
    use crate::widget;

    use widget::{ScrollList, StyledPathList};

    pub enum Scene {
        EditFiles(EditFiles),
        End,
        Home(Home),
        Initialization,
    }

    pub struct Home {
        pub menu: ScrollList,
        pub connection_info: String,
    }

    impl Home {
        pub fn new(connection_info: String) -> Home {
            let mut scene = Home {
                menu: ScrollList::new(
                    String::from("Choose an option:"),
                    vec![
                        String::from("Add or remove files"),
                        String::from("Receive"),
                        String::from("End"),
                    ],
                ),
                connection_info,
            };
            scene.menu.next();
            scene
        }

        pub fn interact(
            &mut self,
            event: crossterm::event::Event,
        ) -> Result<Option<backend::Message<backend::data::Ui, backend::event::Ui>>> {
            if let crossterm::event::Event::Key(event) = event {
                match event.code {
                    KeyCode::Up => self.menu.previous(),
                    KeyCode::Down => self.menu.next(),
                    KeyCode::Enter => {
                        if let Some(option) = self.menu.state.selected() {
                            return Ok(Some(backend::Message::Event(
                                backend::event::Ui::Selection(option),
                            )));
                        }
                    }
                    _ => (),
                }
            }
            Ok(None)
        }

        pub fn draw(
            &mut self,
            terminal: &mut tui::Terminal<CrosstermBackend<io::Stdout>>,
        ) -> Result<()> {
            terminal.draw(|f| {
                // UI sections
                let split_horizontal = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
                    .split(f.size());

                let split_vertical = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(0)
                    .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
                    .split(split_horizontal[0]);

                let split_horizontal_1 = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(split_vertical[1]);

                let style = style::Style::default();

                let menu: Vec<ListItem> = self
                    .menu
                    .options
                    .iter()
                    .map(|i| ListItem::new(i.as_ref()))
                    .collect();
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
                f.render_stateful_widget(menu, split_vertical[0], &mut self.menu.state);

                let info = Block::default().title("Info").borders(Borders::ALL);
                f.render_widget(info, split_horizontal[1]);

                let sending = Block::default().title("Sending").borders(Borders::ALL);
                f.render_widget(sending, split_horizontal_1[0]);

                let receiving = Block::default().title("Receiving").borders(Borders::ALL);
                f.render_widget(receiving, split_horizontal_1[1]);
            })?;
            Ok(())
        }
    }

    pub struct EditFiles {
        pub input: Vec<char>,
        pub file_paths: StyledPathList,
    }

    impl EditFiles {
        pub fn new(file_paths: StyledPathList) -> EditFiles {
            let mut scene = EditFiles {
                input: vec![],
                file_paths,
            };
            scene.file_paths.select_first();
            scene
        }

        pub fn interact(
            &mut self,
            event: crossterm::event::Event,
        ) -> Result<Option<backend::Message<backend::data::Ui, backend::event::Ui>>> {
            if let crossterm::event::Event::Key(event) = event {
                match event.code {
                    KeyCode::Backspace
                    | KeyCode::Char(_)
                    | KeyCode::Delete
                    | KeyCode::Down
                    | KeyCode::Enter
                    | KeyCode::Up => {
                        self.file_paths.edit_selected(&event.code)?;
                    }
                    KeyCode::Esc => {
                        self.file_paths.edit_selected(&event.code)?;
                        return Ok(Some(backend::Message::Data(
                            backend::data::Ui::FilePathList(self.file_paths.clone()),
                        )));
                    }

                    _ => (),
                }
            }
            Ok(None)
        }

        pub fn draw(
            &mut self,
            terminal: &mut tui::Terminal<CrosstermBackend<io::Stdout>>,
        ) -> Result<()> {
            terminal.draw(|f| {
                let style = style::Style::default();

                let styled_paths = self.file_paths.get_styled_paths();
                let file_paths: Vec<ListItem> = styled_paths
                    .iter()
                    .map(|i| ListItem::new(i.as_ref()))
                    .collect();
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
                f.render_stateful_widget(file_paths, f.size(), &mut self.file_paths.state);

                let block = Block::default().borders(Borders::ALL);
                f.render_widget(block, f.size());
            })?;
            Ok(())
        }
    }
}
