use std::{path::Path, process::Command};

use crate::{
    metadata::{BasicMetadata, ChromaLocation, ColorCoordinates, HdrMetadata, Metadata},
    tools::run_command_output,
    values::{
        parse_color_primaries,
        parse_color_range,
        parse_matrix_coefficients,
        parse_transfer_characteristics,
    },
    Error,
    Result,
};

// MKVInfo may include data that looks like this:
//
// |    + Colour matrix coefficients: 9
// |    + Colour range: 1
// |    + Horizontal chroma siting: 2
// |    + Vertical chroma siting: 2
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
    let mut command = Command::new("mkvinfo");
    command.arg(input);
    let result = run_command_output(&mut command, "mkvinfo")?;
    let output = String::from_utf8_lossy(&result.stdout);

    let mut basic = BasicMetadata::default();
    let mut has_basic = false;
    let mut hdr = HdrMetadata::default();
    let mut has_hdr = false;
    let mut chroma_location = (0, 0);

    for line in output.lines() {
        if line.contains("Colour matrix coefficients:") {
            let value = value_after_separator("mkvinfo", line, ": ")?;
            basic.matrix = parse_int("mkvinfo", "matrix coefficients", value)?;
            has_basic = true;
            continue;
        }
        if line.contains("Colour range:") {
            let value = value_after_separator("mkvinfo", line, ": ")?;
            basic.range = parse_int("mkvinfo", "color range", value)?;
            has_basic = true;
            continue;
        }
        if line.contains("Colour transfer:") {
            let value = value_after_separator("mkvinfo", line, ": ")?;
            basic.transfer = parse_int("mkvinfo", "transfer characteristics", value)?;
            has_basic = true;
            continue;
        }
        if line.contains("Colour primaries:") {
            let value = value_after_separator("mkvinfo", line, ": ")?;
            basic.primaries = parse_int("mkvinfo", "color primaries", value)?;
            has_basic = true;
            continue;
        }
        if line.contains("Horizontal chroma siting:") {
            let value = value_after_separator("mkvinfo", line, ": ")?;
            chroma_location.0 = parse_int("mkvinfo", "horizontal chroma siting", value)?;
            has_basic = true;
            continue;
        }
        if line.contains("Vertical chroma siting:") {
            let value = value_after_separator("mkvinfo", line, ": ")?;
            chroma_location.1 = parse_int("mkvinfo", "vertical chroma siting", value)?;
            has_basic = true;
            continue;
        }

        // HDR details
        if line.contains("Video colour mastering metadata") {
            has_hdr = true;
            continue;
        }
        if line.contains("Maximum content light:") {
            let value = value_after_separator("mkvinfo", line, ": ")?;
            hdr.max_content_light = parse_int("mkvinfo", "maximum content light", value)?;
            continue;
        }
        if line.contains("Maximum frame light:") {
            let value = value_after_separator("mkvinfo", line, ": ")?;
            hdr.max_frame_light = parse_int("mkvinfo", "maximum frame light", value)?;
            continue;
        }

        if line.contains("Red colour coordinate x:") {
            let value = value_after_separator("mkvinfo", line, ": ")?;
            color_coords_mut(&mut hdr).red.0 = parse_float("mkvinfo", "red x", value)?;
            continue;
        }
        if line.contains("Red colour coordinate y:") {
            let value = value_after_separator("mkvinfo", line, ": ")?;
            color_coords_mut(&mut hdr).red.1 = parse_float("mkvinfo", "red y", value)?;
            continue;
        }
        if line.contains("Green colour coordinate x:") {
            let value = value_after_separator("mkvinfo", line, ": ")?;
            color_coords_mut(&mut hdr).green.0 = parse_float("mkvinfo", "green x", value)?;
            continue;
        }
        if line.contains("Green colour coordinate y:") {
            let value = value_after_separator("mkvinfo", line, ": ")?;
            color_coords_mut(&mut hdr).green.1 = parse_float("mkvinfo", "green y", value)?;
            continue;
        }
        if line.contains("Blue colour coordinate x:") {
            let value = value_after_separator("mkvinfo", line, ": ")?;
            color_coords_mut(&mut hdr).blue.0 = parse_float("mkvinfo", "blue x", value)?;
            continue;
        }
        if line.contains("Blue colour coordinate y:") {
            let value = value_after_separator("mkvinfo", line, ": ")?;
            color_coords_mut(&mut hdr).blue.1 = parse_float("mkvinfo", "blue y", value)?;
            continue;
        }
        if line.contains("White colour coordinate x:") {
            let value = value_after_separator("mkvinfo", line, ": ")?;
            color_coords_mut(&mut hdr).white.0 = parse_float("mkvinfo", "white x", value)?;
            continue;
        }
        if line.contains("White colour coordinate y:") {
            let value = value_after_separator("mkvinfo", line, ": ")?;
            color_coords_mut(&mut hdr).white.1 = parse_float("mkvinfo", "white y", value)?;
            continue;
        }

        if line.contains("Maximum luminance:") {
            let value = value_after_separator("mkvinfo", line, ": ")?;
            hdr.max_luma = parse_int("mkvinfo", "maximum luminance", value)?;
            continue;
        }
        if line.contains("Minimum luminance:") {
            let value = value_after_separator("mkvinfo", line, ": ")?;
            hdr.min_luma = parse_float("mkvinfo", "minimum luminance", value)?;
            continue;
        }
    }

    if has_basic {
        basic.chroma_location = match chroma_location {
            (0, 0) => {
                // This matches mpv's defaults
                match basic.range {
                    // Full
                    0 => ChromaLocation::Center,
                    // Limited
                    1 => ChromaLocation::Left,
                    _ => {
                        return Err(Error::UnsupportedValue {
                            kind: "color range",
                            value: basic.range.to_string(),
                        });
                    }
                }
            }
            (0, 1) => match basic.range {
                0 => ChromaLocation::Top,
                1 => ChromaLocation::TopLeft,
                _ => {
                    return Err(Error::UnsupportedValue {
                        kind: "color range",
                        value: basic.range.to_string(),
                    });
                }
            },
            (0, 2) => match basic.range {
                0 => ChromaLocation::Center,
                1 => ChromaLocation::Left,
                _ => {
                    return Err(Error::UnsupportedValue {
                        kind: "color range",
                        value: basic.range.to_string(),
                    });
                }
            },
            (0, 3) => match basic.range {
                0 => ChromaLocation::Bottom,
                1 => ChromaLocation::BottomLeft,
                _ => {
                    return Err(Error::UnsupportedValue {
                        kind: "color range",
                        value: basic.range.to_string(),
                    });
                }
            },
            (1, 0) => ChromaLocation::Left,
            (1, 1) => ChromaLocation::TopLeft,
            (1, 2) => ChromaLocation::Left,
            (1, 3) => ChromaLocation::BottomLeft,
            (2, 0) => ChromaLocation::Center,
            (2, 1) => ChromaLocation::Top,
            (2, 2) => ChromaLocation::Center,
            (2, 3) => ChromaLocation::Bottom,
            (x, y) => {
                return Err(Error::UnexpectedOutput {
                    tool: "mkvinfo",
                    line: format!("Unrecognized chroma location values: {x}, {y}"),
                });
            }
        }
    }

    Ok(Metadata {
        basic: if has_basic { Some(basic) } else { None },
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
// Note that MediaInfo does not print the chroma location, so we should
// always prefer mkvinfo's basic output if we have it.
pub fn parse_mediainfo(input: &Path) -> Result<Metadata> {
    let mut command = Command::new("mediainfo");
    command.arg(input);
    let result = run_command_output(&mut command, "mediainfo")?;
    let output = String::from_utf8_lossy(&result.stdout);

    let mut basic = BasicMetadata::default();
    let mut has_basic = false;
    let mut hdr = HdrMetadata::default();
    let mut has_hdr = false;

    for line in output.lines() {
        if line.contains("Matrix coefficients") {
            basic.matrix =
                parse_matrix_coefficients(value_after_separator("mediainfo", line, ": ")?)?;
            has_basic = true;
            continue;
        }
        if line.contains("Color range") {
            basic.range = parse_color_range(value_after_separator("mediainfo", line, ": ")?)?;
            has_basic = true;
            continue;
        }
        if line.contains("Transfer characteristics") {
            basic.transfer =
                parse_transfer_characteristics(value_after_separator("mediainfo", line, ": ")?)?;
            has_basic = true;
            continue;
        }
        if line.contains("Color primaries") {
            basic.primaries =
                parse_color_primaries(value_after_separator("mediainfo", line, ": ")?)?;
            has_basic = true;
            continue;
        }

        // HDR details
        if line.contains("Mastering display color primaries") {
            has_hdr = true;
            continue;
        }
        if line.contains("Maximum Content Light Level") {
            let value = value_after_separator("mediainfo", line, ": ")?
                .trim_end_matches(" cd/m2")
                .replace(' ', "");
            hdr.max_content_light = parse_int("mediainfo", "maximum content light level", &value)?;
            continue;
        }
        if line.contains("Maximum Frame-Average Light Level") {
            let value = value_after_separator("mediainfo", line, ": ")?
                .trim_end_matches(" cd/m2")
                .replace(' ', "");
            hdr.max_frame_light =
                parse_int("mediainfo", "maximum frame-average light level", &value)?;
            continue;
        }
        if line.contains("Mastering display luminance") {
            let output = value_after_separator("mediainfo", line, ": ")?;
            let (min, max) = output
                .split_once(", ")
                .ok_or_else(|| Error::UnexpectedOutput {
                    tool: "mediainfo",
                    line: line.to_string(),
                })?;
            hdr.min_luma = parse_float(
                "mediainfo",
                "minimum luminance",
                min.trim_start_matches("min: ").trim_end_matches(" cd/m2"),
            )?;
            hdr.max_luma = parse_int(
                "mediainfo",
                "maximum luminance",
                max.trim_start_matches("max: ").trim_end_matches(" cd/m2"),
            )?;
            continue;
        }

        if line.contains("Encoding settings") && line.contains("master-display") {
            let settings = value_after_separator("mediainfo", line, ": ")?;
            hdr.color_coords = Some(parse_x265_settings(settings)?);
        }
    }

    Ok(Metadata {
        basic: if has_basic { Some(basic) } else { None },
        hdr: if has_hdr { Some(hdr) } else { None },
    })
}

// Takes in a string that contains a substring in the format:
// master-display=G(13250,34499)B(7499,2999)R(34000,15999)WP(15634,16450)L(10000000,50)cll=944,143
fn parse_x265_settings(input: &str) -> Result<ColorCoordinates> {
    const MASTER_DISPLAY_HEADER: &str = "master-display=";

    let header_pos = input
        .find(MASTER_DISPLAY_HEADER)
        .ok_or_else(|| Error::UnexpectedOutput {
            tool: "mediainfo",
            line: input.to_string(),
        })?;

    let input = &input[(header_pos + MASTER_DISPLAY_HEADER.len())..];
    let (input, (gx, gy)) = parse_coordinate_pair("mediainfo", input, "G", "green")?;
    let (input, (bx, by)) = parse_coordinate_pair("mediainfo", input, "B", "blue")?;
    let (input, (rx, ry)) = parse_coordinate_pair("mediainfo", input, "R", "red")?;
    let (_, (wx, wy)) = parse_coordinate_pair("mediainfo", input, "WP", "white point")?;

    // Why 50000? Why indeed.
    Ok(ColorCoordinates {
        red: (rx as f64 / 50000., ry as f64 / 50000.),
        green: (gx as f64 / 50000., gy as f64 / 50000.),
        blue: (bx as f64 / 50000., by as f64 / 50000.),
        white: (wx as f64 / 50000., wy as f64 / 50000.),
    })
}

fn parse_coordinate_pair<'a>(
    tool: &'static str,
    input: &'a str,
    prefix: &str,
    field: &str,
) -> Result<(&'a str, (u32, u32))> {
    let input = input
        .strip_prefix(prefix)
        .ok_or_else(|| Error::UnexpectedOutput {
            tool,
            line: format!("Missing `{prefix}` prefix in `{input}`"),
        })?;
    let input = input
        .strip_prefix('(')
        .ok_or_else(|| Error::UnexpectedOutput {
            tool,
            line: format!("Missing `(` after `{prefix}` in `{input}`"),
        })?;

    let (coordinates, rest) = input
        .split_once(')')
        .ok_or_else(|| Error::UnexpectedOutput {
            tool,
            line: format!("Missing closing `)` for `{prefix}` in `{input}`"),
        })?;
    let (x, y) = coordinates
        .split_once(',')
        .ok_or_else(|| Error::UnexpectedOutput {
            tool,
            line: format!("Missing coordinate separator for `{prefix}` in `{coordinates}`"),
        })?;

    let x = parse_int(tool, format!("{field} x"), x)?;
    let y = parse_int(tool, format!("{field} y"), y)?;

    Ok((rest, (x, y)))
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
    let mut command = Command::new("ffprobe");
    command
        .arg("-v")
        .arg("quiet")
        .arg("-select_streams")
        .arg("v:0")
        .arg("-show_frames")
        .arg("-read_intervals")
        .arg("%+#1")
        .arg(input);
    let result = run_command_output(&mut command, "ffprobe")?;
    let output = String::from_utf8_lossy(&result.stdout);

    if !(output.contains("side_data_type=Mastering display metadata")
        && output.contains("side_data_type=Content light level metadata"))
    {
        return Ok(None);
    }

    let mut hdr = HdrMetadata::default();
    for line in output.lines() {
        if line.starts_with("red_x=") {
            let value = value_after_separator("ffprobe", line, "=")?;
            color_coords_mut(&mut hdr).red.0 = parse_fraction_f64("ffprobe", "red_x", value)?;
            continue;
        }
        if line.starts_with("red_y=") {
            let value = value_after_separator("ffprobe", line, "=")?;
            color_coords_mut(&mut hdr).red.1 = parse_fraction_f64("ffprobe", "red_y", value)?;
            continue;
        }
        if line.starts_with("green_x=") {
            let value = value_after_separator("ffprobe", line, "=")?;
            color_coords_mut(&mut hdr).green.0 = parse_fraction_f64("ffprobe", "green_x", value)?;
            continue;
        }
        if line.starts_with("green_y=") {
            let value = value_after_separator("ffprobe", line, "=")?;
            color_coords_mut(&mut hdr).green.1 = parse_fraction_f64("ffprobe", "green_y", value)?;
            continue;
        }
        if line.starts_with("blue_x=") {
            let value = value_after_separator("ffprobe", line, "=")?;
            color_coords_mut(&mut hdr).blue.0 = parse_fraction_f64("ffprobe", "blue_x", value)?;
            continue;
        }
        if line.starts_with("blue_y=") {
            let value = value_after_separator("ffprobe", line, "=")?;
            color_coords_mut(&mut hdr).blue.1 = parse_fraction_f64("ffprobe", "blue_y", value)?;
            continue;
        }
        if line.starts_with("white_point_x=") {
            let value = value_after_separator("ffprobe", line, "=")?;
            color_coords_mut(&mut hdr).white.0 =
                parse_fraction_f64("ffprobe", "white_point_x", value)?;
            continue;
        }
        if line.starts_with("white_point_y=") {
            let value = value_after_separator("ffprobe", line, "=")?;
            color_coords_mut(&mut hdr).white.1 =
                parse_fraction_f64("ffprobe", "white_point_y", value)?;
            continue;
        }
        if line.starts_with("min_luminance=") {
            let value = value_after_separator("ffprobe", line, "=")?;
            hdr.min_luma = parse_fraction_f64("ffprobe", "min_luminance", value)?;
            continue;
        }
        if line.starts_with("max_luminance=") {
            let value = value_after_separator("ffprobe", line, "=")?;
            hdr.max_luma = parse_fraction_u32("ffprobe", "max_luminance", value)?;
            continue;
        }

        if line.starts_with("max_content=") {
            let value = value_after_separator("ffprobe", line, "=")?;
            hdr.max_content_light = parse_int("ffprobe", "max_content", value)?;
            continue;
        }
        if line.starts_with("max_average=") {
            let value = value_after_separator("ffprobe", line, "=")?;
            hdr.max_frame_light = parse_int("ffprobe", "max_average", value)?;
            continue;
        }
    }
    Ok(Some(hdr))
}

fn color_coords_mut(hdr: &mut HdrMetadata) -> &mut ColorCoordinates {
    hdr.color_coords
        .get_or_insert_with(ColorCoordinates::default)
}

fn value_after_separator<'a>(
    tool: &'static str,
    line: &'a str,
    separator: &str,
) -> Result<&'a str> {
    line.split_once(separator)
        .map(|(_, value)| value)
        .ok_or_else(|| Error::UnexpectedOutput {
            tool,
            line: line.to_string(),
        })
}

fn parse_int<T>(tool: &'static str, field: impl Into<String>, value: &str) -> Result<T>
where
    T: std::str::FromStr<Err = std::num::ParseIntError>,
{
    let field = field.into();
    value.parse::<T>().map_err(|source| Error::ParseInt {
        tool,
        field,
        value: value.to_string(),
        source,
    })
}

fn parse_float(tool: &'static str, field: impl Into<String>, value: &str) -> Result<f64> {
    let field = field.into();
    value.parse::<f64>().map_err(|source| Error::ParseFloat {
        tool,
        field,
        value: value.to_string(),
        source,
    })
}

fn parse_fraction_f64(tool: &'static str, field: impl Into<String>, value: &str) -> Result<f64> {
    let field = field.into();
    let (numerator, denominator) =
        value
            .split_once('/')
            .ok_or_else(|| Error::UnexpectedOutput {
                tool,
                line: value.to_string(),
            })?;

    let numerator = parse_float(tool, field.clone(), numerator)?;
    let denominator = parse_float(tool, field.clone(), denominator)?;
    if denominator == 0.0 {
        return Err(Error::UnexpectedOutput {
            tool,
            line: format!("{field} has a zero denominator: {value}"),
        });
    }

    Ok(numerator / denominator)
}

fn parse_fraction_u32(tool: &'static str, field: impl Into<String>, value: &str) -> Result<u32> {
    let field = field.into();
    let (numerator, denominator) =
        value
            .split_once('/')
            .ok_or_else(|| Error::UnexpectedOutput {
                tool,
                line: value.to_string(),
            })?;

    let numerator = parse_int::<u32>(tool, field.clone(), numerator)?;
    let denominator = parse_int::<u32>(tool, field.clone(), denominator)?;
    if denominator == 0 {
        return Err(Error::UnexpectedOutput {
            tool,
            line: format!("{field} has a zero denominator: {value}"),
        });
    }

    Ok(numerator / denominator)
}
