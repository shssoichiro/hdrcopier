#![warn(clippy::all)]

mod error;
mod metadata;
mod parse;
mod tools;
mod values;

use std::path::PathBuf;

pub use crate::error::{Error, Result};
use crate::metadata::{Metadata, extract_chapters};

pub fn copy(input: PathBuf, target: PathBuf, chapters: bool) -> Result<()> {
    if !input.is_file() {
        return Err(Error::InputNotAFile { path: input });
    }

    if !target.is_file() {
        return Err(Error::TargetNotAFile { path: target });
    }

    let metadata = Metadata::parse(&input)?;
    let chapters = if chapters {
        extract_chapters(&input)?
    } else {
        None
    };
    metadata.apply(&target, chapters.as_deref())?;

    Ok(())
}

pub fn show(input: PathBuf, formatting: Option<&str>) -> Result<()> {
    if !input.is_file() {
        return Err(Error::InputNotAFile { path: input });
    }

    let metadata = Metadata::parse(&input)?;
    metadata.print(formatting)
}
