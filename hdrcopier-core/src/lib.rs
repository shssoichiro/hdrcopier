#![warn(clippy::all)]

mod error;
mod metadata;
mod parse;
mod tools;
mod values;

use std::{path::PathBuf, process::Command};

pub use crate::error::{Error, Result};
use crate::{
    metadata::{Metadata, extract_chapters},
    tools::{ensure_tools_in_path, run_command_output},
};

/// Copy metadata and optionally chapters
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

/// Copies only the chapters without copying metadata
pub fn copy_chapters(input: PathBuf, target: PathBuf) -> Result<()> {
    let chapters = extract_chapters(&input)?;

    if let Some(chapters) = chapters {
        ensure_tools_in_path(&["mkvpropedit"])?;
        let mut command = Command::new("mkvpropedit");
        command.arg("-c").arg(chapters);
        command.arg(target);
        run_command_output(&mut command, "mkvpropedit")?;
    }

    Ok(())
}

/// Display the metadata for a file
pub fn show(input: PathBuf, formatting: Option<&str>) -> Result<()> {
    if !input.is_file() {
        return Err(Error::InputNotAFile { path: input });
    }

    let metadata = Metadata::parse(&input)?;
    metadata.print(formatting)
}
