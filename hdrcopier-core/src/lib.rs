#![warn(clippy::all)]

mod metadata;
mod parse;
mod values;

use std::{path::PathBuf, process::exit};

use crate::metadata::{extract_chapters, Metadata};

pub fn copy(input: PathBuf, target: PathBuf, chapters: bool) {
    if !input.is_file() {
        eprintln!("Input file {:?} does not exist", input);
        exit(1);
    }
    if !target.is_file() {
        eprintln!("Target file {:?} does not exist", target);
        exit(1);
    }

    let metadata = match Metadata::parse(&input) {
        Ok(metadata) => metadata,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };
    let chapters = if chapters {
        extract_chapters(&input)
    } else {
        None
    };
    if let Err(e) = metadata.apply(&target, chapters.as_deref()) {
        eprintln!("{}", e);
        exit(1);
    };

    eprintln!("Done!");
}

#[cfg(feature = "save")]
pub fn save(input: PathBuf, target: PathBuf, chapters: bool) {
    if !input.is_file() {
        eprintln!("Input file {:?} does not exist", input);
        exit(1);
    }
    if input.exists() && !input.is_file() {
        eprintln!("Target {:?} is not a file", target);
        exit(1);
    }

    let metadata = match Metadata::parse(&input) {
        Ok(metadata) => metadata,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };
    let chapters = if chapters {
        extract_chapters(&input)
    } else {
        None
    };
    let json = match serde_json::to_string_pretty(&(metadata, chapters)) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };
    if let Err(e) = std::fs::write(target, json) {
        eprintln!("{}", e);
        exit(1);
    };

    eprint!("Done!");
}

#[cfg(feature = "save")]
pub fn restore(input: PathBuf, target: PathBuf, chapters: bool) {
    if !input.is_file() {
        eprintln!("Input file {:?} does not exist", input);
        exit(1);
    }
    if !target.is_file() {
        eprintln!("Target file {:?} does not exist", target);
        exit(1);
    }
    let file = match std::fs::File::open(input) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };

    let (metadata, saved_chapters) =
        match serde_json::from_reader::<_, (Metadata, Option<PathBuf>)>(file) {
            Ok(metadata) => metadata,
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        };
    let saved_chapters = if chapters { saved_chapters } else { None };
    if let Err(e) = metadata.apply(&target, saved_chapters.as_deref()) {
        eprintln!("{}", e);
        exit(1);
    };

    eprintln!("Done!");
}

pub fn show(input: PathBuf, formatting: Option<&str>) {
    if !input.is_file() {
        eprintln!("Input file {:?} does not exist", input);
        exit(1);
    }

    let metadata = match Metadata::parse(&input) {
        Ok(metadata) => metadata,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };
    metadata.print(formatting);
}
