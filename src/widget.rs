use anyhow::{self, Result};

use crossterm::event::KeyCode;

use tui::{self, widgets::ListState};

use crate::file_processing;
use file_processing::PathState;

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
    pub path: String,
    pub display_path: String,
    pub state: PathState,
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
    pub heading: String,
    pub paths: Vec<StyledFilePath>,
    pub state: ListState,
}

impl StyledPathList {
    pub fn new(heading: String, paths: Vec<StyledFilePath>) -> StyledPathList {
        StyledPathList {
            heading,
            paths,
            state: ListState::default(),
        }
    }

    pub fn edit_selected(&mut self, key: &KeyCode) -> Result<()> {
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
                self.parse_selected()?;
                self.next()?;
            }
            KeyCode::Enter => {
                if let Some(index) = self.state.selected() {
                    self.parse_selected()?;
                    self.insert_empty_element(index);
                    self.paths[index].select();
                }
            }
            KeyCode::Up => {
                self.parse_selected()?;
                self.previous()?;
            }
            KeyCode::Esc => {
                self.parse_selected()?;
            }
            _ => (),
        }
        Ok(())
    }

    pub fn export(&mut self) -> Result<StyledPathList> {
        // Making sure that everything is parsed and checked before exporting
        self.previous()?;
        self.next()?;
        Ok(self.clone())
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

    pub fn previous(&mut self) -> Result<()> {
        let i = match self.state.selected() {
            Some(i) => {
                if self.paths[i].state == PathState::Unchecked {
                    self.parse_selected()?;
                } else {
                    self.paths[i].deselect();
                }
                (i as isize - 1).rem_euclid(self.paths.len() as isize) as usize
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.paths[i].select();
        Ok(())
    }

    pub fn selected(&mut self) -> Option<&mut StyledFilePath> {
        if let Some(index) = self.state.selected() {
            return Some(&mut self.paths[index]);
        }
        None
    }

    pub fn select_first(&mut self) {
        if self.paths.is_empty() {
            self.insert_empty_element(0);
        }

        self.paths[0].select();
        self.state.select(Some(0));
    }
}
