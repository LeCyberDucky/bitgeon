use fancy_regex::Regex;
use lazy_static::lazy_static;
use std::fs::OpenOptions;
use std::path::Path;
use walkdir::WalkDir;

#[derive(PartialEq)]
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
    // Convert string to path
    let path = Path::new(path_string);
    // Check if path is valid
    // If directory: Check if it can be read and count the elements that can be read
    // If file: Check if it can be read

    if path.is_file() {
        let file = OpenOptions::new().read(true).open(path);

        match file {
            Ok(_) => return PathState::File,
            Err(error) => return PathState::Invalid,
        }
    }

    if path.is_dir() {
        return PathState::Directory(
            WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .count(),
        );
    }

    // Path is neither valid file nor directory
    PathState::Invalid
}
