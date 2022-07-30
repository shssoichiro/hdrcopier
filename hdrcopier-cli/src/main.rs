#![warn(clippy::all)]

use std::path::PathBuf;

use clap::{Arg, Command};

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
        Some("copy") => {
            let sub_args = args.subcommand_matches("copy").unwrap();
            let input = PathBuf::from(&sub_args.value_of("input").expect("Value required by clap"));
            let target =
                PathBuf::from(&sub_args.value_of("target").expect("Value required by clap"));
            let chapters = sub_args.is_present("chapters");

            hdrcopier_core::copy(input, target, chapters)
        }
        Some("show") => {
            let sub_args = args.subcommand_matches("show").unwrap();
            let input = PathBuf::from(&sub_args.value_of("input").expect("Value required by clap"));

            hdrcopier_core::show(input, sub_args.value_of("format"))
        }
        _ => {
            eprintln!("Unrecognized command entered; see `hdrcopier -h` for usage");
            std::process::exit(1);
        }
    }
}
