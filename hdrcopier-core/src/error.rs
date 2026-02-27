use std::{
    io,
    num::{ParseFloatError, ParseIntError},
    path::PathBuf,
};

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Input file {path:?} does not exist")]
    InputNotAFile { path: PathBuf },

    #[error("Target file {path:?} does not exist")]
    TargetNotAFile { path: PathBuf },

    #[error("Required tool(s) not found in PATH: {}", .tools.join(", "))]
    MissingRequiredTools { tools: Vec<&'static str> },

    #[error("Required tool `{tool}` not found in PATH")]
    ToolNotFound { tool: &'static str },

    #[error("Failed to start `{tool}`: {source}")]
    ToolSpawn {
        tool: &'static str,
        #[source]
        source: io::Error,
    },

    #[error("`{tool}` failed with exit code {code:?}: {stderr}")]
    ToolFailed {
        tool: &'static str,
        code: Option<i32>,
        stderr: String,
    },

    #[error("Unexpected output from `{tool}`: {line}")]
    UnexpectedOutput { tool: &'static str, line: String },

    #[error("Failed to parse integer field `{field}` from `{tool}` output: `{value}`")]
    ParseInt {
        tool: &'static str,
        field: String,
        value: String,
        #[source]
        source: ParseIntError,
    },

    #[error("Failed to parse float field `{field}` from `{tool}` output: `{value}`")]
    ParseFloat {
        tool: &'static str,
        field: String,
        value: String,
        #[source]
        source: ParseFloatError,
    },

    #[error("Unsupported output format `{format}`")]
    UnsupportedFormat { format: String },

    #[error("Unsupported value for {kind}: `{value}`")]
    UnsupportedValue { kind: &'static str, value: String },

    #[error("Missing HDR mastering display coordinates for {format}")]
    MissingHdrColorCoordinates { format: &'static str },

    #[error("I/O error while accessing {path:?}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
