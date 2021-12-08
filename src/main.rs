use std::{
    env,
    path::{Path, PathBuf},
    process::{exit, Command},
};

use anyhow::Result;
use dialoguer::Confirm;
use nom::{
    bytes::complete::{tag, take, take_until},
    character::complete::{char, digit1},
    combinator::map,
    error::ErrorKind,
    sequence::{delimited, preceded, separated_pair},
    IResult,
};

fn main() {
    let args = env::args().skip(1).take(3).collect::<Vec<_>>();

    if args.len() != 3 || args[0] == "-h" || args[0] == "--help" {
        eprintln!("Usage: hdrcopier [input] [target] [output]");
        eprintln!("input = file to copy metadata from");
        eprintln!("target = file to copy metadata to");
        eprintln!("output = location to place the resulting file");
        exit(1);
    }

    let input = PathBuf::from(&args[0]);
    let target = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);

    if !input.is_file() {
        eprintln!("Input file {:?} does not exist", input);
        exit(1);
    }
    if !target.is_file() {
        eprintln!("Target file {:?} does not exist", target);
        exit(1);
    }
    if output.is_dir() {
        eprintln!("Output location already exists as a directory--cannot proceed, exiting");
        exit(1);
    }
    if output.is_file()
        && !Confirm::new()
            .with_prompt("Output file already exists. Overwrite?")
            .interact()
            .unwrap()
    {
        exit(1);
    }

    let metadata = match Metadata::parse(&input) {
        Ok(metadata) => metadata,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };
    if let Err(e) = metadata.apply(&target, &output) {
        eprintln!("{}", e);
        exit(1);
    };

    eprintln!("Done!");
}

#[derive(Default)]
struct Metadata {
    pub basic: BasicMetadata,
    pub hdr: Option<HdrMetadata>,
}

#[derive(Default)]
struct BasicMetadata {
    pub matrix: u8,
    pub range: u8,
    pub transfer: u8,
    pub primaries: u8,
}

#[derive(Default)]
struct HdrMetadata {
    pub red: (f64, f64),
    pub green: (f64, f64),
    pub blue: (f64, f64),
    pub white: (f64, f64),
    pub max_luma: u32,
    pub min_luma: f64,
    pub max_content_light: u32,
    pub max_frame_light: u32,
}

impl Metadata {
    pub fn parse(input: &Path) -> Result<Self> {
        let data;
        match parse_mkvinfo(input) {
            Ok(info) => {
                data = info;
            }
            Err(e) => {
                eprintln!("Warning: {}", e);
                data = Metadata::default();
            }
        }
        if data.hdr.is_some() {
            return Ok(data);
        }
        match parse_mediainfo(input) {
            Ok(info) => {
                data = info;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                anyhow::bail!("Unable to parse metadata");
            }
        }
        Ok(data)
    }

    pub fn apply(&self, target: &Path, output: &Path) -> Result<()> {
        let command = Command::new("mkvmerge")
            .arg("-o")
            .arg(output)
            .arg("--colour-range")
            .arg(format!("0:{}", self.basic.range))
            .arg("--colour-transfer-characteristics")
            .arg(format!("0:{}", self.basic.transfer))
            .arg("--colour-primaries")
            .arg(format!("0:{}", self.basic.primaries))
            .arg("--colour-matrix-coefficients")
            .arg(format!("0:{}", self.basic.matrix));
        if let Some(hdr_data) = self.hdr {
            command
                .arg("--max-content-light")
                .arg(format!("0:{}", hdr_data.max_content_light))
                .arg("--max-frame-light")
                .arg(format!("0:{}", hdr_data.max_frame_light))
                .arg("--max-luminance")
                .arg(format!("0:{}", hdr_data.max_luma))
                .arg("--min-luminance")
                .arg(format!("0:{:.4}", hdr_data.min_luma))
                .arg("--chromaticity-coordinates")
                .arg(format!(
                    "0:{:.5},{:.5},{:.5},{:.5},{:.5},{:.5}",
                    hdr_data.red.0,
                    hdr_data.red.1,
                    hdr_data.green.0,
                    hdr_data.green.1,
                    hdr_data.blue.0,
                    hdr_data.blue.1
                ))
                .arg("--white-colour-coordinates")
                .arg(format!("0:{:.5},{:.5}", hdr_data.white.0, hdr_data.white.1));
        }
        command.arg(target);
        eprintln!("Running: {:?}", command);
        let status = command.status()?;
        if !status.success() {
            anyhow::bail!("Failed to mux metadata");
        }
        Ok(())
    }
}

// MKVInfo may include data that looks like this:
//
// |    + Colour matrix coefficients: 9
// |    + Colour range: 1
// |    + Colour transfer: 16
// |    + Colour primaries: 9
// |    + Maximum content light: 944
// |    + Maximum frame light: 143
// |    + Video colour mastering metadata
// |     + Red colour coordinate x: 0.6800000071525574
// |     + Red colour coordinate y: 0.3199799954891205
// |     + Green colour coordinate x: 0.26499998569488525
// |     + Green colour coordinate y: 0.6899799704551697
// |     + Blue colour coordinate x: 0.15000000596046448
// |     + Blue colour coordinate y: 0.05998000130057335
// |     + White colour coordinate x: 0.3126800060272217
// |     + White colour coordinate y: 0.32899999618530273
// |     + Maximum luminance: 1000
// |     + Minimum luminance: 0.004999999888241291
//
// This is the case if the metadata was muxed into the MKV headers.
fn parse_mkvinfo(input: &Path) -> Result<Metadata> {
    let result = Command::new("mkvinfo").arg(input).output()?;
    let output = String::from_utf8_lossy(&result.stdout);

    let mut basic = BasicMetadata::default();
    let mut hdr = HdrMetadata::default();
    let mut has_hdr = false;
    for line in output.lines() {
        if line.contains("Colour matrix coefficients:") {
            basic.matrix = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("Colour range:") {
            basic.range = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("Colour transfer:") {
            basic.transfer = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("Colour primaries:") {
            basic.primaries = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }

        // HDR details
        if line.contains("Video colour mastering metadata") {
            has_hdr = true;
            continue;
        }
        if line.contains("Maximum content light:") {
            hdr.max_content_light = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("Maximum frame light:") {
            hdr.max_frame_light = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("Red colour coordinate x:") {
            hdr.red.0 = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("Red colour coordinate y:") {
            hdr.red.1 = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("Green colour coordinate x:") {
            hdr.green.0 = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("Green colour coordinate y:") {
            hdr.green.1 = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("Blue colour coordinate x:") {
            hdr.blue.0 = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("Blue colour coordinate y:") {
            hdr.blue.1 = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("White colour coordinate x:") {
            hdr.white.0 = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("White colour coordinate y:") {
            hdr.white.1 = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("Maximum luminance:") {
            hdr.max_luma = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("Minimum luminance:") {
            hdr.min_luma = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
    }

    Ok(Metadata {
        basic,
        hdr: if has_hdr { Some(hdr) } else { None },
    })
}

// MediaInfo may include the following pieces of data:
//
// In the x265 headers: master-display=G(13250,34499)B(7499,2999)R(34000,15999)WP(15634,16450)L(10000000,50)cll=944,143
//
// In the video info:
//
// Color range                              : Limited
// Color primaries                          : BT.2020
// Transfer characteristics                 : PQ
// Matrix coefficients                      : BT.2020 non-constant
// Mastering display color primaries        : Display P3
// Mastering display luminance              : min: 0.0050 cd/m2, max: 1000 cd/m2
// Maximum Content Light Level              : 944 cd/m2
// Maximum Frame-Average Light Level        : 143 cd/m2
//
// We need this if the metadata was encoded into the video stream by x265.
fn parse_mediainfo(input: &Path) -> Result<Metadata> {
    let result = Command::new("mediainfo").arg(input).output()?;
    let output = String::from_utf8_lossy(&result.stdout);

    let mut basic = BasicMetadata::default();
    let mut hdr = HdrMetadata::default();
    let mut has_hdr = false;
    for line in output.lines() {
        if line.contains("Matrix coefficients") {
            basic.matrix = match line.split_once(": ").unwrap().1.to_lowercase().as_str() {
                "rgb" => 0,
                "bt.709" => 1,
                "unspecified" | "unset" => 2,
                "fcc" => 4,
                "bt.470 bg" => 5,
                "smpte 170m" | "bt.601" => 6,
                "smpte 240m" => 7,
                "ycgco" => 8,
                "bt.2020 non-constant" => 9,
                "bt.2020 constant" => 10,
                // FIXME: Not sure how these two are formatted in mediainfo
                // VSC_MATRIX_CHROMATICITY_DERIVED_NCL = 12,
                // VSC_MATRIX_CHROMATICITY_DERIVED_CL = 13,
                _ => panic!("Unrecognized matrix coefficients"),
            };
            continue;
        }
        if line.contains("Color range") {
            basic.range = match line.split_once(": ").unwrap().1 {
                "Limited" => 1,
                "Full" => 0,
                _ => panic!("Unrecognized color range"),
            };
            continue;
        }
        if line.contains("Transfer characteristics") {
            basic.transfer = match line.split_once(": ").unwrap().1.to_lowercase().as_str() {
                "bt.709" => 1,
                "unspecified" | "unset" => 2,
                "bt.470 m" => 4,
                "bt.470 bg" => 5,
                "bt.601" => 6,
                "smpte 240m" => 7,
                "linear" => 8,
                "log 100" => 9,
                "log 316" => 10,
                "iec 61966 2.4" => 11,
                "iec 61966 2.1" => 13,
                "bt.2020 10-bit" => 14,
                "bt.2020 12-bit" => 15,
                "pq" | "smpte 2084" => 16,
                "arib b67" => 18,
                _ => panic!("Unrecognized transfer characteristics"),
            };
            continue;
        }
        if line.contains("Color primaries") {
            basic.primaries = match line.split_once(": ").unwrap().1.to_lowercase().as_str() {
                "bt.709" => 1,
                "unspecified" | "unset" => 2,
                "bt.470 m" => 4,
                "bt.470 bg" => 5,
                "smpte 170m" | "bt.601" => 6,
                "smpte 240m" => 7,
                "film" | "ntsc" => 8,
                "bt.2020" => 9,
                "smpte 428" => 10,
                "smpte 431.2" => 11,
                "smpte 432.1" => 12,
                "ebu 3213 e" => 22,
                _ => panic!("Unrecognized color primaries"),
            };
            continue;
        }

        // HDR details
        if line.contains("Mastering display color primaries") {
            has_hdr = true;
            continue;
        }
        if line.contains("Maximum Content Light Level") {
            hdr.max_content_light = line
                .split_once(": ")
                .unwrap()
                .1
                .trim_end_matches(" cd/m2")
                .parse()?;
            continue;
        }
        if line.contains("Maximum Frame-Average Light Level") {
            hdr.max_frame_light = line
                .split_once(": ")
                .unwrap()
                .1
                .trim_end_matches(" cd/m2")
                .parse()?;
            continue;
        }
        if line.contains("Mastering display luminance") {
            let output = line.split_once(": ").unwrap().1;
            let (min, max) = output.split_once(", ").unwrap();
            hdr.min_luma = min
                .trim_start_matches("min: ")
                .trim_end_matches(" cd/m2")
                .parse()?;
            hdr.max_luma = min
                .trim_start_matches("max: ")
                .trim_end_matches(" cd/m2")
                .parse()?;
            continue;
        }

        if line.contains("Encoding settings") && line.contains("master-display") {
            let settings = line.split_once(": ").unwrap().1;
            let settings = parse_x265_settings(&settings)?;
            // Why 50000? Why indeed. Ask the x265 maintainers.
            hdr.red.0 = (settings.0).0 as f64 / 50000.;
            hdr.red.0 = (settings.0).1 as f64 / 50000.;
            hdr.green.0 = (settings.1).0 as f64 / 50000.;
            hdr.green.0 = (settings.1).1 as f64 / 50000.;
            hdr.blue.0 = (settings.2).0 as f64 / 50000.;
            hdr.blue.0 = (settings.2).1 as f64 / 50000.;
            hdr.white.0 = (settings.3).0 as f64 / 50000.;
            hdr.white.0 = (settings.3).1 as f64 / 50000.;
        }
    }

    Ok(Metadata {
        basic,
        hdr: if has_hdr { Some(hdr) } else { None },
    })
}

// Takes in a string that contains a substring in the format:
// master-display=G(13250,34499)B(7499,2999)R(34000,15999)WP(15634,16450)L(10000000,50)cll=944,143
//
// Returns ((Rx, Ry), (Gx, Gy), (Bx, By), (Wx, Wy))
//
// Also using unwrap here because who cares?
fn parse_x265_settings(input: &str) -> Result<((u32, u32), (u32, u32), (u32, u32), (u32, u32))> {
    const MASTER_DISPLAY_HEADER: &str = "master-display=";
    let header_pos = input
        .find(MASTER_DISPLAY_HEADER)
        .ok_or(anyhow::anyhow!("Failed to find master display header"))?;
    let input = &input[(header_pos + MASTER_DISPLAY_HEADER.len())..];
    let (input, (gx, gy)) = preceded(char('G'), get_coordinate_pair)(input)?;
    let (input, (bx, by)) = preceded(char('B'), get_coordinate_pair)(input)?;
    let (input, (rx, ry)) = preceded(char('R'), get_coordinate_pair)(input)?;
    let (_, (wx, wy)) = preceded(tag("WP"), get_coordinate_pair)(input)?;
    Ok(((rx, ry), (gx, gy), (bx, by), (wx, wy)))
}

fn get_coordinate_pair(input: &str) -> IResult<&str, (u32, u32)> {
    map(
        delimited(
            char('('),
            separated_pair(digit1, char(','), digit1),
            char(')'),
        ),
        |(x, y): (&str, &str)| {
            // Should I use unwrap here? No. Do I care? Also no.
            (x.parse::<u32>().unwrap(), y.parse::<u32>().unwrap())
        },
    )(input)
}
