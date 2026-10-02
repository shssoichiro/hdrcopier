use std::{path::Path, process::Command};

use crate::{
    Error, Result,
    metadata::{BasicMetadata, ChromaLocation, ColorCoordinates, HdrMetadata, Metadata},
    tools::run_command_output,
    values::{
        parse_color_primaries, parse_color_range, parse_matrix_coefficients,
        parse_transfer_characteristics,
    },
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

    parse_mkvinfo_output(&output)
}

fn parse_mkvinfo_output(output: &str) -> Result<Metadata> {
    let mut basic = BasicMetadata::default();
    let mut has_basic = false;
    let mut hdr = HdrMetadata::default();
    let mut has_hdr = false;
    let mut chroma_location = (0, 0);

    // Stop at the end of the first video subtree, even if it has no color metadata.
    let video_lines = output
        .lines()
        .skip_while(|line| *line != "|  + Video track")
        .skip(1)
        .take_while(|line| line.starts_with("|   "));
    for line in video_lines {
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

    parse_mediainfo_output(&output)
}

fn parse_mediainfo_output(output: &str) -> Result<Metadata> {
    let mut basic = BasicMetadata::default();
    let mut has_basic = false;
    let mut hdr = HdrMetadata::default();
    let mut has_hdr = false;

    // Attachments and later videos must not supply or override the first video's fields.
    let video_lines = output
        .lines()
        .skip_while(|line| {
            let line = line.trim();
            line != "Video"
                && !line.strip_prefix("Video #").is_some_and(|number| {
                    !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit())
                })
        })
        .skip(1)
        .take_while(|line| !line.trim().is_empty());
    for line in video_lines {
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
        // Uppercase V excludes attached pictures, thumbnails, and cover art.
        .arg("V:0")
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

#[cfg(test)]
mod tests {
    use super::*;

    const MEDIAINFO_FIELDS: &str = "Color range : Limited
Color primaries : BT.2020
Transfer characteristics : PQ
Matrix coefficients : BT.2020 non-constant
Mastering display color primaries : Display P3
Mastering display luminance : min: 0.0010 cd/m2, max: 1000 cd/m2
Maximum Content Light Level : 349 cd/m2
Maximum Frame-Average Light Level : 120 cd/m2
Encoding settings : master-display=G(13250,34500)B(7500,3000)R(34000,16000)WP(15635,16450)L(10000000,10)";

    const MKVINFO_FIELDS: &str = "|   + Video colour information
|    + Colour matrix coefficients: 9
|    + Colour range: 1
|    + Colour transfer: 16
|    + Colour primaries: 9
|    + Horizontal chroma siting: 1
|    + Vertical chroma siting: 1
|    + Maximum content light: 349
|    + Maximum frame light: 120
|    + Video colour mastering metadata
|     + Red colour coordinate x: 0.68
|     + Red colour coordinate y: 0.32
|     + Green colour coordinate x: 0.265
|     + Green colour coordinate y: 0.69
|     + Blue colour coordinate x: 0.15
|     + Blue colour coordinate y: 0.06
|     + White colour coordinate x: 0.3127
|     + White colour coordinate y: 0.329
|     + Maximum luminance: 1000
|     + Minimum luminance: 0.001";

    fn assert_first_video_metadata(metadata: Metadata, chroma: ChromaLocation) {
        let basic = metadata.basic.expect("first video has basic metadata");
        assert_eq!(
            (basic.range, basic.primaries, basic.transfer, basic.matrix),
            (1, 9, 16, 9)
        );
        assert_eq!(basic.chroma_location as u8, chroma as u8);
        let hdr = metadata.hdr.expect("first video has HDR metadata");
        assert_eq!(
            (hdr.max_luma, hdr.max_content_light, hdr.max_frame_light),
            (1000, 349, 120)
        );
        assert!((hdr.min_luma - 0.001).abs() < 1e-10);
        let coords = hdr
            .color_coords
            .expect("first video has mastering coordinates");
        for (actual, expected) in [
            (coords.red, (0.68, 0.32)),
            (coords.green, (0.265, 0.69)),
            (coords.blue, (0.15, 0.06)),
            (coords.white, (0.3127, 0.329)),
        ] {
            assert!((actual.0 - expected.0).abs() < 1e-10);
            assert!((actual.1 - expected.1).abs() < 1e-10);
        }
    }

    #[test]
    fn mediainfo_uses_only_first_video() {
        for heading in ["Video", "Video #1"] {
            for suffix in [
                "",
                "\n\nVideo #2\nDefault : Yes\nColor range : Full\nColor primaries : BT.709\nTransfer characteristics : BT.709\nMatrix coefficients : BT.709\nMastering display luminance : min: 1 cd/m2, max: 4000 cd/m2\nMaximum Content Light Level : 999 cd/m2\nMaximum Frame-Average Light Level : 500 cd/m2\nEncoding settings : master-display=G(1,2)B(3,4)R(5,6)WP(7,8)",
                "\n \t\nImage\nColor range : Full\nColor primaries : BT.709\nTransfer characteristics : sRGB/sYCC\nMatrix coefficients : Identity",
                "\n\nAudio\nTransfer characteristics : BT.709\n\nText\nEncoding settings : master-display=invalid",
            ] {
                let output = format!(
                    "General\nTitle : Video #1\n\nImage\nTransfer characteristics : sRGB/sYCC\n\n{heading}\n{MEDIAINFO_FIELDS}{suffix}"
                );
                for output in [&output, &output.replace('\n', "\r\n")] {
                    let metadata =
                        parse_mediainfo_output(output).expect("ignore all non-primary sections");
                    assert_first_video_metadata(metadata, ChromaLocation::Left);
                }
            }
        }

        // An ordinary single-video report needs neither a General section nor a trailing separator.
        assert_first_video_metadata(
            parse_mediainfo_output(&format!("Video\n{MEDIAINFO_FIELDS}")).expect("single video"),
            ChromaLocation::Left,
        );
        for output in [
            String::new(),
            format!("Image\n{MEDIAINFO_FIELDS}"),
            format!("Video #1 extra\n{MEDIAINFO_FIELDS}"),
            format!("General\nTitle : Video\n{MEDIAINFO_FIELDS}"),
            format!("Video #1\nFormat : HEVC\n\nVideo #2\nDefault : Yes\n{MEDIAINFO_FIELDS}"),
            format!("Video\nFormat : HEVC\n\nImage\n{MEDIAINFO_FIELDS}"),
        ] {
            let metadata = parse_mediainfo_output(&output).expect("no selected-video metadata");
            assert!(metadata.basic.is_none(), "{output}");
            assert!(metadata.hdr.is_none(), "{output}");
        }
        let metadata = parse_mediainfo_output(&format!(
            "Video\nTransfer characteristics : PQ\n\nImage\n{MEDIAINFO_FIELDS}"
        ))
        .expect("image must not supply HDR");
        assert_eq!(metadata.basic.expect("video transfer").transfer, 16);
        assert!(metadata.hdr.is_none());
    }

    #[test]
    fn mkvinfo_uses_only_first_video() {
        for suffix in [
            "",
            "\n| + Track\n|  + Track type: video\n|  + Video track\n|   + Colour transfer: 1\n|   + Colour primaries: 1\n|   + Colour range: 0\n|   + Horizontal chroma siting: 2\n|   + Maximum content light: 999\n|   + Maximum frame light: 500\n|   + Maximum luminance: 4000\n|   + Minimum luminance: 1\n|   + Red colour coordinate x: 0.1",
            "\n| + Track\n|  + Track type: audio\n|  + Audio track\n|   + Maximum luminance: invalid",
            "\n|+ Attachments\n| + Attached\n|  + Maximum luminance: invalid",
            "\n|+ Tags\n| + Tag\n|  + Colour transfer: invalid",
        ] {
            let output = format!(
                "|+ Tracks\n| + Track\n|  + Track type: audio\n|  + Audio track\n|   + Colour transfer: 1\n| + Track\n|  + Track type: video\n|  + Video track\n{MKVINFO_FIELDS}{suffix}"
            );
            let metadata = parse_mkvinfo_output(&output).expect("ignore non-primary subtrees");
            assert_first_video_metadata(metadata, ChromaLocation::TopLeft);
        }
        assert_first_video_metadata(
            parse_mkvinfo_output(&format!("| + Track\n|  + Video track\n{MKVINFO_FIELDS}"))
                .expect("single video"),
            ChromaLocation::TopLeft,
        );
        for output in [
            String::new(),
            format!("| + Track\n|  + Audio track\n{MKVINFO_FIELDS}"),
            format!(
                "| + Track\n|  + Video track\n|   + Pixel width: 1920\n| + Track\n|  + Video track\n{MKVINFO_FIELDS}"
            ),
            format!(
                "| + Track\n|  + Video track\n|   + Pixel width: 1920\n|+ Tags\n{MKVINFO_FIELDS}"
            ),
        ] {
            let metadata = parse_mkvinfo_output(&output).expect("no selected-video metadata");
            assert!(metadata.basic.is_none(), "{output}");
            assert!(metadata.hdr.is_none(), "{output}");
        }
        let metadata = parse_mkvinfo_output(&format!("| + Track\n|  + Video track\n|   + Colour transfer: 16\n| + Track\n|  + Video track\n{MKVINFO_FIELDS}"))
            .expect("second video must not supply HDR");
        assert_eq!(metadata.basic.expect("video transfer").transfer, 16);
        assert!(metadata.hdr.is_none());
    }

    #[test]
    fn selected_video_validation_is_preserved() {
        let invalid_transfer = "Transfer characteristics : sRGB/sYCC";
        assert!(matches!(
            parse_mediainfo_output(&format!("Video\n{invalid_transfer}")),
            Err(Error::UnsupportedValue { kind: "transfer characteristics", value }) if value == "sRGB/sYCC"
        ));
        let ignored = format!(
            "Image\n{invalid_transfer}\n\nVideo\nTransfer characteristics : PQ\n\nVideo #2\n{invalid_transfer}"
        );
        assert_eq!(
            parse_mediainfo_output(&ignored)
                .expect("ignore invalid other tracks")
                .basic
                .expect("video transfer")
                .transfer,
            16
        );

        let invalid_luma = "|   + Maximum luminance: invalid";
        assert!(matches!(
            parse_mkvinfo_output(&format!("| + Track\n|  + Video track\n{invalid_luma}")),
            Err(Error::ParseInt { tool: "mkvinfo", field, value, .. }) if field == "maximum luminance" && value == "invalid"
        ));
        let ignored = format!(
            "| + Track\n|  + Audio track\n{invalid_luma}\n| + Track\n|  + Video track\n{MKVINFO_FIELDS}\n| + Track\n|  + Video track\n{invalid_luma}"
        );
        assert_first_video_metadata(
            parse_mkvinfo_output(&ignored).expect("ignore invalid other tracks"),
            ChromaLocation::TopLeft,
        );
    }
}
