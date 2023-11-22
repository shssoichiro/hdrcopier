#![warn(clippy::all)]

use std::path::PathBuf;

use clap::{Arg, ArgAction, Command};

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

    match args.subcommand_name() {
        Some("copy") => {
            let sub_args = args.subcommand_matches("copy").unwrap();
            let input = PathBuf::from(sub_args.get_one::<String>("input").expect("Value required"));
            let target = PathBuf::from(
                sub_args
                    .get_one::<String>("target")
                    .expect("Value required"),
            );
            let chapters = sub_args.get_flag("chapters");

            hdrcopier_core::copy(input, target, chapters)
        }
        Some("show") => {
            let sub_args = args.subcommand_matches("show").unwrap();
            let input = PathBuf::from(sub_args.get_one::<String>("input").expect("Value required"));

            let format: Option<&String> = sub_args.get_one("format");
            hdrcopier_core::show(input, format.map(|s| s.as_str()))
        }
        _ => {
            eprintln!("Unrecognized command entered; see `hdrcopier -h` for usage");
            std::process::exit(1);
        }
    }
}
