use std::{
    io::ErrorKind,
    process::{Command, Output},
};

use crate::{Error, Result};

pub fn ensure_tools_in_path(tools: &[&'static str]) -> Result<()> {
    let missing_tools: Vec<&'static str> = tools
        .iter()
        .copied()
        .filter(|tool| which::which(tool).is_err())
        .collect();

    if missing_tools.is_empty() {
        Ok(())
    } else {
        Err(Error::MissingRequiredTools {
            tools: missing_tools,
        })
    }
}

pub fn run_command_output(command: &mut Command, tool: &'static str) -> Result<Output> {
    let output = command.output().map_err(|source| {
        if source.kind() == ErrorKind::NotFound {
            Error::ToolNotFound { tool }
        } else {
            Error::ToolSpawn { tool, source }
        }
    })?;

    if !output.status.success() {
        return Err(Error::ToolFailed {
            tool,
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    Ok(output)
}
