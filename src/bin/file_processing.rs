use fancy_regex::Regex;
use lazy_static::lazy_static;
use path_absolutize::*;
use std::fs::OpenOptions;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

#[derive(Clone, PartialEq)]
pub enum PathState {
    Directory(usize), // Holds number of files in directory
    File,
    Invalid,
    Unchecked,
}

pub fn parse_paths(path_string: &str) -> Vec<String> {
    let trim_characters = [';', '\"'];
    let mut paths: Vec<String> = vec![];

    // Regex magic built using regexr.com. Don't touch.
    // ((?:[A-z]:.+?(?=[A-z]:|$|\n|;))|(?:.+?(?=;|$|\n|[A-z]:)))
    // /((?:[A-z]:.+?(?=[A-z]:|$|\n|;))|(?:.+?(?=;|$|\n|[A-z]:)))/g
    lazy_static! {
        static ref re: Regex =
            Regex::new(r"((?:[A-z]:.+?(?=[A-z]:|$|\n|;))|(?:.+?(?=;|$|\n|[A-z]:)))").unwrap();
    }

    let mut capture_pos = 0;
    while capture_pos < path_string.len() {
        let result = re
            .captures_from_pos(&path_string, capture_pos)
            .expect("Error running regex");

        match result {
            Some(captures) => {
                let group = captures.get(0).expect("No regex group");
                let path = group.as_str();
                let path = path
                    .trim_matches(|c: char| c.is_whitespace() || trim_characters.contains(&c))
                    .to_string();

                if !path.is_empty() {
                    paths.push(path);
                }

                capture_pos = group.end();
            }
            None => break,
        }
    }

    paths
}

pub fn check_path(path_string: &str) -> PathState {
    let mut path = path_string.to_string();
    if path.trim().is_empty() {
        return PathState::Invalid;
    }

    let trim_characters = ['\\', '/', '.'];
    if Path::new(&path).is_relative() && path.len() > 0 {
        let first_character = path.chars().next().unwrap();
        if first_character != '.' {
            path = path
                .trim_left_matches(|c: char| c.is_whitespace() || trim_characters.contains(&c))
                .to_string();
            path.insert_str(0, "./");
        }
    }

    let path = Path::new(&path);
    let path = path.absolutize().unwrap();

    if path.is_file() {
        let file = OpenOptions::new().read(true).open(path);

        match file {
            Ok(_) => return PathState::File,
            Err(error) => return PathState::Invalid,
        }
    }

    if path.is_dir() {
        let mut element_count = 0;

        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if !is_hidden_path(&entry) && !entry.metadata().unwrap().is_dir() {
                element_count += 1;
            }
        }

        return PathState::Directory(element_count);
    }

    // Path is neither valid file nor directory
    PathState::Invalid
}

// Returns whether a directory entry points to a hidden file or directory
fn is_hidden_path(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}
