use std::fs::OpenOptions;
use std::path::Path;

use anyhow::{Context, Result};
use fancy_regex::Regex;
// use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use path_absolutize::*;
use walkdir::{DirEntry, WalkDir};

#[derive(Clone, PartialEq)]
pub enum PathState {
    Directory(usize), // Holds number of files in directory
    File,
    Invalid,
    Unchecked,
}

pub fn parse_paths(path_string: &str) -> Result<Vec<String>> {
    let trim_characters = [';', '\"'];
    let mut paths: Vec<String> = vec![];

    // Regex magic built using regexr.com. Don't touch.
    // ((?:[A-z]:.+?(?=[A-z]:|$|\n|;))|(?:.+?(?=;|$|\n|[A-z]:)))
    // Used to split strings containing multiple file paths into individual paths. Relative paths should be separated using semicolons. Absolute paths need only be separated by spaces (at least for absolute windows-file paths)
    // Example: The following string should be split into four paths:
    // r"C:\Users\USERNAME\images\ferris.jpg C:\Users\USERNAME\images; /images/; /images/ferris.jpg"
    // lazy_static! {
    //     static ref RE: Regex =
    //         Regex::new(r"((?:[A-z]:.+?(?=[A-z]:|$|\n|;))|(?:.+?(?=;|$|\n|[A-z]:)))").unwrap(); // unwrap is fine because if the regex compiles once, it will always compile
    // }
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"((?:[A-z]:.+?(?=[A-z]:|$|\n|;))|(?:.+?(?=;|$|\n|[A-z]:)))").unwrap()
        // unwrap is fine because if the regex compiles once, it should always compile
    });

    let mut capture_pos = 0;
    while capture_pos < path_string.len() {
        let result = RE
            .captures_from_pos(&path_string, capture_pos)
            .with_context(|| format!("Error running regex"))?;

        match result {
            Some(captures) => {
                let group = captures.get(0).with_context(|| format!("No regex group"))?;
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

    Ok(paths)
}

pub fn check_path(path_string: &str) -> Result<PathState> {
    let mut path = path_string.to_string();
    if path.trim().is_empty() {
        return Ok(PathState::Invalid);
    }

    // If the path is relative, trim it and add "./" to the beginning
    let trim_characters = ['\\', '/', '.'];
    if Path::new(&path).is_relative() && path.len() > 0 {
        let first_character = path.chars().next().unwrap(); // At least one character is contained, as given by the check above
        if first_character != '.' {
            path = path
                .trim_start_matches(|c: char| c.is_whitespace() || trim_characters.contains(&c))
                .to_string();
            path.insert_str(0, "./");
        }
    }

    let path = Path::new(&path);
    let path = path.absolutize().with_context(|| {
        format!(
            "Error turning \"{}\" into absolute path",
            path.to_str().unwrap()
        )
    })?;

    if path.is_file() {
        let file = OpenOptions::new().read(true).open(path);

        match file {
            Ok(_) => return Ok(PathState::File),
            Err(_error) => return Ok(PathState::Invalid),
        }
    }

    if path.is_dir() {
        let mut element_count = 0;

        // TODO: Speed this up
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if !is_hidden_path(&entry) && !entry.metadata()?.is_dir() {
                element_count += 1;
            }
        }

        return Ok(PathState::Directory(element_count));
    }

    // Path is neither valid file nor directory
    Ok(PathState::Invalid)
}

// Returns whether a directory entry points to a hidden file or directory
fn is_hidden_path(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_paths() {
        let input = r"C:\Users\USERNAME\images\ferris.jpg C:\Users\USERNAME\images; /images/; /images/ferris.jpg";

        let output = parse_paths(input);

        println!("{:#?}", output);

        let expected_output = vec![
            "C:\\Users\\USERNAME\\images\\ferris.jpg",
            "C:\\Users\\USERNAME\\images",
            "/images/",
            "/images/ferris.jpg",
        ];

        assert_eq!(output.unwrap(), expected_output);

        let input = r"aklæsjdagklsdjfskhgdælaC:\Users\USERNAME\OneDrive\Documents\USER home\Programming\Projects\bitgeon\src\bin\ui.rs";

        let output = parse_paths(input);

        println!("{:#?}", output);

        let expected_output = vec![
            r"aklæsjdagklsdjfskhgdæla",
            r"C:\Users\USERNAME\OneDrive\Documents\USER home\Programming\Projects\bitgeon\src\bin\ui.rs",
        ];

        assert_eq!(output.unwrap(), expected_output);
    }
}
