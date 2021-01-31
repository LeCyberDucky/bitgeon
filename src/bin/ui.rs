use anyhow::{self, Result};

use crossterm::{
    self,
    event::{self, KeyCode},
    ExecutableCommand,
};

use std::io;
use std::time;

use tui::{
    self,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style,
    widgets::{Block, Borders, List, ListItem},
};

use crate::util;
use crate::widget;
use scene::Scene;
use widget::{ScrollList, StyledPathList};

// Inter-process messages between ui and backend
// TODO: This should just be called Message
pub enum Message {
    Data(Data),
    Event(Event),
}

pub enum Data {
    FilePathList(StyledPathList),
}

pub enum Event {
    Selection(usize),
    StateChange(AppState),
}

pub enum AppState {
    EditFiles(StyledPathList),
    End,
    Home(String),
    Initialization,
}

mod scene {
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
    }
}

pub struct UI {
    pub application: util::ThreadChannel<Message>,
    pub application_state: AppState,
    pub scene: Scene,
    pub ui_refresh_rate: u16,
    pub clock: time::Instant,
    pub frame_count: u64,
    pub last_frame: time::Instant,
    pub frame_changed: bool,
}

impl UI {
    pub fn run(application: util::ThreadChannel<Message>) -> Result<()> {
        // Setup
        let mut ui = UI {
            application,
            application_state: AppState::Initialization,
            scene: Scene::Home(scene::Home::new(String::from(""))),
            ui_refresh_rate: 60,
            clock: time::Instant::now(),
            frame_count: 0,
            last_frame: time::Instant::now(),
            frame_changed: false,
        };

        crossterm::terminal::enable_raw_mode()?;
        io::stdout().execute(crossterm::terminal::EnterAlternateScreen)?;
        io::stdout().execute(crossterm::cursor::Hide)?;
        let mut terminal = tui::Terminal::new(CrosstermBackend::new(io::stdout()))?;

        // TODO: Figure out what is happening here and document that in a comment
        while std::mem::discriminant(&ui.scene) != std::mem::discriminant(&Scene::End) {
            ui.update();
            if ui.frame_changed {
                // Visual change necessitates redraw
                ui.frame_changed = false;
                ui.draw(&mut terminal)?;
            }
            ui.interact()?; // User interaction

            util::sleep_remaining_frame(&ui.clock, &mut ui.frame_count, &ui.ui_refresh_rate);
        }

        // Reset terminal to initial state
        crossterm::terminal::disable_raw_mode()?;
        io::stdout().execute(crossterm::terminal::LeaveAlternateScreen)?;
        io::stdout().execute(crossterm::cursor::Show)?;
        Ok(())
    }

    pub fn draw(&mut self, terminal: &mut tui::Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        match self.scene {
            Scene::EditFiles(_) => {
                terminal
                    .draw(|f| {
                        let style = style::Style::default();

                        if let Scene::EditFiles(data) = &mut self.scene {
                            let styled_paths = data.file_paths.get_styled_paths();
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
                            f.render_stateful_widget(
                                file_paths,
                                f.size(),
                                &mut data.file_paths.state,
                            );
                        }

                        let block = Block::default().borders(Borders::ALL);
                        f.render_widget(block, f.size());
                    })?;
            }

            Scene::Home(_) => {
                terminal
                    .draw(|f| {
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

                        let style = style::Style::default();

                        if let Scene::Home(data) = &mut self.scene {
                            let menu: Vec<ListItem> = data
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
                            f.render_stateful_widget(menu, split_vertical[0], &mut data.menu.state);
                        }

                        let info = Block::default().title("Info").borders(Borders::ALL);
                        f.render_widget(info, split_horizontal[1]);

                        let sending = Block::default().title("Sending").borders(Borders::ALL);
                        f.render_widget(sending, split_horizontal_1[0]);

                        let receiving = Block::default().title("Receiving").borders(Borders::ALL);
                        f.render_widget(receiving, split_horizontal_1[1]);
                    })
                    ?;
            }
            _ => todo!(),
        }
        Ok(())
    }

    pub fn interact(&mut self) -> Result<()> {
        let mut frame_changed = true;

        if event::poll(time::Duration::from_secs(0))? {
            match &mut self.scene {
                Scene::EditFiles(data) => match event::read()? {
                    event::Event::Key(event) => match event.code {
                        KeyCode::Backspace
                        | KeyCode::Char(_)
                        | KeyCode::Delete
                        | KeyCode::Down
                        | KeyCode::Enter
                        | KeyCode::Up => data.file_paths.edit_selected(&event.code)?,
                        KeyCode::Esc => {
                            data.file_paths.edit_selected(&event.code)?;
                            self.application
                                .send(Message::Data(Data::FilePathList(data.file_paths.clone())))?;
                        }

                        _ => (),
                    },
                    // _ => todo!(),
                    _ => frame_changed = self.frame_changed,
                },

                Scene::Home(data) => {
                    match event::read()? {
                        event::Event::Key(event) => match event.code {
                            KeyCode::Up => data.menu.previous(),
                            KeyCode::Down => data.menu.next(),
                            KeyCode::Enter => match data.menu.state.selected() {
                                Some(option) => {
                                    self.application
                                        .send(Message::Event(Event::Selection(option)))?;
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
        Ok(())
    }

    pub fn update(&mut self) {
        // Get updates from logic of the program. Progress for progress bars for example
        let app_updates = self.application.receive();

        if app_updates.len() > 0 {
            for message in app_updates {
                match message {
                    Message::Event(event) => match event {
                        Event::StateChange(state) => {
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
                        Event::Selection(_option) => todo!(),
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

    // pub fn frame_elapsed(&self) -> bool {
    //     // self.period_elapsed(&self.frame_count, &self.ui_refresh_rate)
    //     util::period_elapsed(&self.clock, &self.frame_count, &self.ui_refresh_rate)
    // }
}
