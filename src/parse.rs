use std::{path::Path, process::Command};

use anyhow::Result;
use nom::{
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::map,
    sequence::{delimited, preceded, separated_pair},
    IResult,
};

use crate::{
    metadata::{BasicMetadata, ColorCoordinates, HdrMetadata, Metadata},
    values::{
        parse_color_primaries,
        parse_color_range,
        parse_matrix_coefficients,
        parse_transfer_characteristics,
    },
};

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
pub fn parse_mkvinfo(input: &Path) -> Result<Metadata> {
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
            hdr.color_coords.red.0 = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("Red colour coordinate y:") {
            hdr.color_coords.red.1 = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("Green colour coordinate x:") {
            hdr.color_coords.green.0 = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("Green colour coordinate y:") {
            hdr.color_coords.green.1 = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("Blue colour coordinate x:") {
            hdr.color_coords.blue.0 = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("Blue colour coordinate y:") {
            hdr.color_coords.blue.1 = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("White colour coordinate x:") {
            hdr.color_coords.white.0 = line.split_once(": ").unwrap().1.parse()?;
            continue;
        }
        if line.contains("White colour coordinate y:") {
            hdr.color_coords.white.1 = line.split_once(": ").unwrap().1.parse()?;
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
pub fn parse_mediainfo(input: &Path) -> Result<Metadata> {
    let result = Command::new("mediainfo").arg(input).output()?;
    let output = String::from_utf8_lossy(&result.stdout);

    let mut basic = BasicMetadata::default();
    let mut hdr = HdrMetadata::default();
    let mut has_hdr = false;
    for line in output.lines() {
        if line.contains("Matrix coefficients") {
            basic.matrix = parse_matrix_coefficients(line.split_once(": ").unwrap().1);
            continue;
        }
        if line.contains("Color range") {
            basic.range = parse_color_range(line.split_once(": ").unwrap().1);
            continue;
        }
        if line.contains("Transfer characteristics") {
            basic.transfer = parse_transfer_characteristics(line.split_once(": ").unwrap().1);
            continue;
        }
        if line.contains("Color primaries") {
            basic.primaries = parse_color_primaries(line.split_once(": ").unwrap().1);
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
            hdr.max_luma = max
                .trim_start_matches("max: ")
                .trim_end_matches(" cd/m2")
                .parse()?;
            continue;
        }

        if line.contains("Encoding settings") && line.contains("master-display") {
            let settings = line.split_once(": ").unwrap().1;
            hdr.color_coords = parse_x265_settings(settings)?;
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
// Also using unwrap here because I don't want to fight the borrow checker anymore.
fn parse_x265_settings(input: &str) -> Result<ColorCoordinates> {
    const MASTER_DISPLAY_HEADER: &str = "master-display=";
    let header_pos = input
        .find(MASTER_DISPLAY_HEADER)
        .ok_or(anyhow::anyhow!("Failed to find master display header"))?;
    let input = &input[(header_pos + MASTER_DISPLAY_HEADER.len())..];
    let (input, (gx, gy)) = preceded(char('G'), get_coordinate_pair)(input).unwrap();
    let (input, (bx, by)) = preceded(char('B'), get_coordinate_pair)(input).unwrap();
    let (input, (rx, ry)) = preceded(char('R'), get_coordinate_pair)(input).unwrap();
    let (_, (wx, wy)) = preceded(tag("WP"), get_coordinate_pair)(input).unwrap();

    // Why 50000? Why indeed.
    Ok(ColorCoordinates {
        red: (rx as f64 / 50000., ry as f64 / 50000.),
        green: (gx as f64 / 50000., gy as f64 / 50000.),
        blue: (bx as f64 / 50000., by as f64 / 50000.),
        white: (wx as f64 / 50000., wy as f64 / 50000.),
    })
}

fn get_coordinate_pair(input: &str) -> IResult<&str, (u32, u32)> {
    map(
        delimited(
            char('('),
            separated_pair(digit1, char(','), digit1),
            char(')'),
        ),
        |(x, y): (&str, &str)| (x.parse::<u32>().unwrap(), y.parse::<u32>().unwrap()),
    )(input)
}

// And then there are some videos where the data only shows in ffprobe.
//
// Like so:
//
// [SIDE_DATA]
// side_data_type=Mastering display metadata
// red_x=34000/50000
// red_y=15999/50000
// green_x=13250/50000
// green_y=34499/50000
// blue_x=7499/50000
// blue_y=2999/50000
// white_point_x=15634/50000
// white_point_y=16450/50000
// min_luminance=50/10000
// max_luminance=10000000/10000
// [/SIDE_DATA]
// [SIDE_DATA]
// side_data_type=Content light level metadata
// max_content=944
// max_average=143
// [/SIDE_DATA]
//
// This only looks at HDR data, because at least one of mediainfo
// or mkvinfo should have found the color primary data.
// Or your source is badly broken.
pub fn parse_ffprobe(input: &Path) -> Result<Option<HdrMetadata>> {
    let result = Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-show_frames")
        .arg("-read_intervals")
        .arg("%+#1")
        .arg(input)
        .output()?;
    let output = String::from_utf8_lossy(&result.stdout);

    if !(output.contains("side_data_type=Mastering display metadata")
        && output.contains("side_data_type=Content light level metadata"))
    {
        return Ok(None);
    }

    let mut hdr = HdrMetadata::default();
    for line in output.lines() {
        if line.starts_with("red_x=") {
            let (num, denom) = line.split_once('=').unwrap().1.split_once('/').unwrap();
            hdr.color_coords.red.0 = num.parse::<f64>()? / denom.parse::<f64>()?;
            continue;
        }
        if line.starts_with("red_y=") {
            let (num, denom) = line.split_once('=').unwrap().1.split_once('/').unwrap();
            hdr.color_coords.red.1 = num.parse::<f64>()? / denom.parse::<f64>()?;
            continue;
        }
        if line.starts_with("green_x=") {
            let (num, denom) = line.split_once('=').unwrap().1.split_once('/').unwrap();
            hdr.color_coords.green.0 = num.parse::<f64>()? / denom.parse::<f64>()?;
            continue;
        }
        if line.starts_with("green_y=") {
            let (num, denom) = line.split_once('=').unwrap().1.split_once('/').unwrap();
            hdr.color_coords.green.1 = num.parse::<f64>()? / denom.parse::<f64>()?;
            continue;
        }
        if line.starts_with("blue_x=") {
            let (num, denom) = line.split_once('=').unwrap().1.split_once('/').unwrap();
            hdr.color_coords.blue.0 = num.parse::<f64>()? / denom.parse::<f64>()?;
            continue;
        }
        if line.starts_with("blue_y=") {
            let (num, denom) = line.split_once('=').unwrap().1.split_once('/').unwrap();
            hdr.color_coords.blue.1 = num.parse::<f64>()? / denom.parse::<f64>()?;
            continue;
        }
        if line.starts_with("white_point_x=") {
            let (num, denom) = line.split_once('=').unwrap().1.split_once('/').unwrap();
            hdr.color_coords.white.0 = num.parse::<f64>()? / denom.parse::<f64>()?;
            continue;
        }
        if line.starts_with("white_point_y=") {
            let (num, denom) = line.split_once('=').unwrap().1.split_once('/').unwrap();
            hdr.color_coords.white.1 = num.parse::<f64>()? / denom.parse::<f64>()?;
            continue;
        }
        if line.starts_with("min_luminance=") {
            let (num, denom) = line.split_once('=').unwrap().1.split_once('/').unwrap();
            hdr.min_luma = num.parse::<f64>()? / denom.parse::<f64>()?;
            continue;
        }
        if line.starts_with("max_luminance=") {
            let (num, denom) = line.split_once('=').unwrap().1.split_once('/').unwrap();
            hdr.max_luma = num.parse::<u32>()? / denom.parse::<u32>()?;
            continue;
        }

        if line.starts_with("max_content=") {
            hdr.max_content_light = line.split_once('=').unwrap().1.parse()?;
            continue;
        }
        if line.starts_with("max_average=") {
            hdr.max_frame_light = line.split_once('=').unwrap().1.parse()?;
            continue;
        }
    }
    Ok(Some(hdr))
}
