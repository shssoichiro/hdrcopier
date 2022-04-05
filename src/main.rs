#![warn(clippy::all)]

mod metadata;
mod parse;
mod values;

use std::{env, path::PathBuf, process::exit};

use clap::{Arg, ArgMatches, Command};

use crate::metadata::{extract_chapters, Metadata};

fn main() {
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
                        .takes_value(false),
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
                        .takes_value(true)
                        .possible_values(&["x265", "rav1e", "mkvmerge"]),
                ),
        )
        .get_matches();

    match args.subcommand_name() {
        Some("copy") => copy(args.subcommand_matches("copy").unwrap()),
        Some("show") => show(args.subcommand_matches("show").unwrap()),
        _ => {
            eprintln!("Unrecognized command entered; see `hdrcopier -h` for usage");
            exit(1);
        }
    }
}

fn copy(args: &ArgMatches) {
    let input = PathBuf::from(&args.value_of("input").expect("Value required by clap"));
    let target = PathBuf::from(&args.value_of("target").expect("Value required by clap"));
    let chapters = args.is_present("chapters");

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

fn show(args: &ArgMatches) {
    let input = PathBuf::from(&args.value_of("input").expect("Value required by clap"));

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
    metadata.print(args.value_of("format"));
}
