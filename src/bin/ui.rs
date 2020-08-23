use anyhow::{Result};

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
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::file_processing;
use crate::util;
use file_processing::PathState;

// Inter-process messages between ui and backend
pub enum UIMessage {
    Data(UIData),
    Event(UIEvent),
}

pub enum UIData {
    FilePathList(StyledPathList),
}

pub enum UIEvent {
    Selection(usize),
    StateChange(AppState),
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

#[derive(Clone)]
pub struct StyledFilePath {
    path: String,
    display_path: String,
    state: PathState,
}

impl StyledFilePath {
    // TODO: Fix item count in directories
    pub fn new(path: &str) -> StyledFilePath {
        StyledFilePath {
            path: path.to_string(),
            display_path: path.to_string(),
            state: PathState::Unchecked,
        }
    }

    pub fn deselect(&mut self) {
        // TODO: Deselection stuff
    }

    pub fn edit(&mut self, key: &KeyCode) {
        match key {
            KeyCode::Backspace => {
                self.path.pop();
            }
            KeyCode::Char(character) => {
                let sanitized_character = match *character {
                    'â' => '\\',
                    _ => *character,
                };
                self.path.push(sanitized_character);
            }
            _ => (),
        }
        if self.state != PathState::Unchecked {
            self.state = PathState::Unchecked;
        }
        self.select();
    }

    pub fn select(&mut self) {
        self.display_path = "   ".to_string(); // Padding spaces to account for emojis of other strings
        self.display_path.push_str(&self.path);
    }

    // TODO: Rename this to style. Validation is done elsewhere.
    pub fn style(&mut self) {
        // Update display_path
        self.display_path = self.path.clone();
        match self.state {
            PathState::Directory(count) => {
                self.display_path.insert_str(0, "✔  ");
                match count {
                    1 => self.display_path.push_str(" | 1 Accessible item"),
                    _ => self
                        .display_path
                        .push_str(&format!(" | {} Accessible items", count)),
                }
            }
            PathState::File => self.display_path.insert_str(0, "✔  "),
            PathState::Invalid => self.display_path.insert_str(0, "❌ "),
            _ => (),
        }
    }

    pub fn validate(&mut self) {
        self.state = match file_processing::check_path(&self.path) {
            Ok(state) => state,
            Err(_) => PathState::Invalid,
        }
    }
}

#[derive(Clone)]
pub struct StyledPathList {
    heading: String,
    paths: Vec<StyledFilePath>,
    state: ListState,
}

impl StyledPathList {
    pub fn new(heading: String, paths: Vec<StyledFilePath>) -> StyledPathList {
        StyledPathList {
            heading,
            paths,
            state: ListState::default(),
        }
    }

    pub fn edit_selected(&mut self, key: &KeyCode) {
        match key {
            KeyCode::Backspace | KeyCode::Char(_) => {
                if let Some(index) = self.state.selected() {
                    self.paths[index].edit(key);
                }
            }
            KeyCode::Delete => {
                // Delete current entry. Make sure not to select something invalid
                if let Some(mut index) = self.state.selected() {
                    self.paths.remove(index);
                    match self.paths.len() {
                        0 => self.paths.push(StyledFilePath::new("")),
                        _ => (),
                    }
                    if index >= self.paths.len() {
                        index -= 1;
                    }
                    self.state.select(Some(index));
                    self.paths[index].select();
                }
            }
            KeyCode::Down => {
                self.parse_selected();
                self.next();
            }
            KeyCode::Enter => {
                if let Some(index) = self.state.selected() {
                    self.parse_selected();
                    self.insert_empty_element(index);
                    // let mut new_element = StyledFilePath::new("");
                    // new_element.style();
                    // self.paths.insert(index, new_element);
                    self.paths[index].select();
                }
            }
            KeyCode::Up => {
                self.parse_selected();
                self.previous();
            }
            _ => (),
        }
    }

    pub fn export(&mut self) -> StyledPathList {
        // Making sure that everything is parsed and checked before exporting
        self.previous();
        self.next();
        self.clone()
    }

    pub fn get_styled_paths(&self) -> Vec<String> {
        self.paths
            .iter()
            .map(|path| path.display_path.clone())
            .collect()
    }

    pub fn insert_empty_element(&mut self, index: usize) {
        let mut new_element = StyledFilePath::new("");
        new_element.style();
        self.paths.insert(index, new_element);
    }

    pub fn next(&mut self) -> Result<()> {
        let mut offset = 1;
        let i = match self.state.selected() {
            Some(i) => {
                if self.paths[i].state == PathState::Unchecked {
                    offset += self.parse_selected()?;
                } else {
                    self.paths[i].deselect();
                }
                (i + offset) % self.paths.len()
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.paths[i].select();
        Ok(())
    }

    pub fn parse_selected(&mut self) -> Result<usize> {
        let index = self.state.selected().unwrap();
        let mut parsed_paths = file_processing::parse_paths(&self.paths[index].path)?;
        let mut new_path_count = parsed_paths.len();

        if new_path_count == 0 {
            parsed_paths.push(String::from(""));
            new_path_count = 1;
        }

        let new_paths: Vec<StyledFilePath> = parsed_paths
            .iter()
            .map(|path| {
                let mut styled_path = StyledFilePath::new(path);
                styled_path.validate();
                styled_path.style();
                styled_path
            })
            .collect();

        // Replace selected element with parsed elements
        self.paths.splice(index..index + 1, new_paths);

        Ok(new_path_count - 1)
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if self.paths[i].state == PathState::Unchecked {
                    self.parse_selected();
                } else {
                    self.paths[i].deselect();
                }
                (i as isize - 1).rem_euclid(self.paths.len() as isize) as usize
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.paths[i].select();
    }

    pub fn selected(&mut self) -> Option<&mut StyledFilePath> {
        if let Some(index) = self.state.selected() {
            return Some(&mut self.paths[index]);
        }
        None
    }

    pub fn select_first(&mut self) {
        if self.paths.len() == 0 {
            self.insert_empty_element(0);
        }

        self.paths[0].select();
        self.state.select(Some(0));
    }
}

pub enum AppState {
    AddFiles(StyledPathList),
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
    file_paths: StyledPathList,
}

impl SceneAddFiles {
    pub fn new(file_paths: StyledPathList) -> SceneAddFiles {
        let mut scene = SceneAddFiles {
            input: vec![],
            file_paths,
        };
        scene.file_paths.select_first();
        scene
    }
}

pub struct SceneHome {
    menu: ScrollList,
}
impl SceneHome {
    pub fn new() -> SceneHome {
        let mut scene = SceneHome {
            menu: ScrollList::new(
                String::from("Choose an option:"),
                vec![
                    String::from("Add files"),
                    String::from("Receive"),
                    String::from("End"),
                ],
            ),
        };
        scene.menu.next();
        scene
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
                        let style = style::Style::default();

                        if let Scene::AddFiles(data) = &mut self.scene {
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
                    .unwrap();
            }
            _ => todo!(),
        }
    }

    pub fn interact(&mut self) {
        let mut frame_changed = true;

        if event::poll(time::Duration::from_secs(0)).unwrap() {
            match &mut self.scene {
                Scene::AddFiles(data) => match event::read().unwrap() {
                    event::Event::Key(event) => match event.code {
                        KeyCode::Backspace
                        | KeyCode::Char(_)
                        | KeyCode::Delete
                        | KeyCode::Down
                        | KeyCode::Enter
                        | KeyCode::Up => data.file_paths.edit_selected(&event.code),
                        KeyCode::Esc => {
                            self.application
                                .send(UIMessage::Data(UIData::FilePathList(
                                    data.file_paths.clone(),
                                )));
                        }
                        _ => (),
                    },
                    _ => todo!(),
                    _ => frame_changed = self.frame_changed,
                },

                Scene::Home(data) => {
                    match event::read().unwrap() {
                        event::Event::Key(event) => match event.code {
                            KeyCode::Up => data.menu.previous(),
                            KeyCode::Down => data.menu.next(),
                            KeyCode::Enter => match data.menu.state.selected() {
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
                                AppState::AddFiles(file_list) => {
                                    Scene::AddFiles(SceneAddFiles::new(file_list.to_owned()))
                                }
                                AppState::End => Scene::End,
                                AppState::Home => Scene::Home(SceneHome::new()),
                                AppState::Initialization => todo!(),
                            }
                        }
                        UIEvent::Selection(option) => unimplemented!(),
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
