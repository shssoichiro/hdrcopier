#![warn(clippy::all)]

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::{Arg, ArgAction, Command};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Command::new("hdrcopier")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            Command::new("copy")
                .about("Merges the metadata from one file with the media streams from another")
                .arg(
                    Arg::new("input")
                        .help("file to copy metadata from")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("target")
                        .help("file to copy metadata to; must be a matroska file")
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::new("chapters")
                        .help("Also copy chapters from input to output")
                        .long("chapters")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("show")
                .about("Displays the metadata to the user")
                .arg(
                    Arg::new("input")
                        .help("file to parse metadata from")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("format")
                        .help("display output in a CLI-compatible format")
                        .long("format")
                        .short('f')
                        .value_parser(["x265", "svt-av1", "rav1e", "mkvmerge"]),
                ),
        )
        .get_matches();

    match args.subcommand() {
        Some(("copy", sub_args)) => {
            let input = PathBuf::from(
                sub_args
                    .get_one::<String>("input")
                    .context("missing required argument: input")?,
            );
            let target = PathBuf::from(
                sub_args
                    .get_one::<String>("target")
                    .context("missing required argument: target")?,
            );
            let chapters = sub_args.get_flag("chapters");

            hdrcopier_core::copy(input, target, chapters)?;
            eprintln!("Done!");
        }
        Some(("show", sub_args)) => {
            let input = PathBuf::from(
                sub_args
                    .get_one::<String>("input")
                    .context("missing required argument: input")?,
            );

            let format: Option<&String> = sub_args.get_one("format");
            hdrcopier_core::show(input, format.map(|s| s.as_str()))?;
        }
        Some((command, _)) => {
            return Err(anyhow!(
                "Unrecognized command entered: {command}; see `hdrcopier -h` for usage"
            ));
        }
        None => {
            return Err(anyhow!("No command entered; see `hdrcopier -h` for usage"));
        }
    }

    Ok(())
}
