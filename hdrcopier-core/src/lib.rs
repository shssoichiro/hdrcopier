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
