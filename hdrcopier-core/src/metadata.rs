use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;

use crate::{
    parse::{parse_ffprobe, parse_mediainfo, parse_mkvinfo},
    values::{
        color_range_to_mkvedit_prop,
        print_color_primaries,
        print_color_range,
        print_matrix_coefficients,
        print_rav1e_color_primaries,
        print_rav1e_color_range,
        print_rav1e_matrix_coefficients,
        print_rav1e_transfer_characteristics,
        print_transfer_characteristics,
        print_x265_color_primaries,
        print_x265_color_range,
        print_x265_matrix_coefficients,
        print_x265_transfer_characteristics,
    },
};

#[derive(Default)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct Metadata {
    pub basic: Option<BasicMetadata>,
    pub hdr: Option<HdrMetadata>,
}

#[derive(Default)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct BasicMetadata {
    pub matrix: u8,
    pub range: u8,
    pub transfer: u8,
    pub primaries: u8,
}

#[derive(Default)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct ColorCoordinates {
    pub red: (f64, f64),
    pub green: (f64, f64),
    pub blue: (f64, f64),
    pub white: (f64, f64),
}

#[derive(Default)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct HdrMetadata {
    pub color_coords: Option<ColorCoordinates>,
    pub max_luma: u32,
    pub min_luma: f64,
    pub max_content_light: u32,
    pub max_frame_light: u32,
}

impl Metadata {
    // Why do we have to go through all three of these?
    //
    // WELL, I'm glad you asked.
    // Sometimes, exactly one of these three tools will be able
    // to extract the HDR metadata. So we have to test all three.
    // Just to be sure we didn't miss it.
    //
    // Encoding is dumb.
    pub fn parse(input: &Path) -> Result<Self> {
        let mut data = Metadata::default();
        match parse_mkvinfo(input) {
            Ok(info) => {
                data = info;
            }
            Err(e) => {
                eprintln!("Warning: {}", e);
            }
        }
        if data.basic.is_some()
            && data.hdr.is_some()
            && data.hdr.as_ref().unwrap().color_coords.is_some()
        {
            return Ok(data);
        }

        match parse_mediainfo(input) {
            Ok(info) => {
                if data.basic.is_none() && info.basic.is_some() {
                    data.basic = info.basic;
                }
                if info.hdr.is_some() {
                    data.hdr = info.hdr;
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                anyhow::bail!("Unable to parse metadata");
            }
        }
        if data.hdr.is_some() && data.hdr.as_ref().unwrap().color_coords.is_some() {
            return Ok(data);
        }

        match parse_ffprobe(input) {
            Ok(Some(info)) => {
                data.hdr = Some(info);
            }
            Ok(None) => (),
            Err(e) => {
                eprintln!("Warning: {}", e);
            }
        }

        Ok(data)
    }

    pub fn apply(&self, target: &Path, chapters: Option<&Path>) -> Result<()> {
        let mut command = self.build_mkvmerge_command(target, chapters);
        eprintln!("Running: {:?}", command);
        let status = command.status()?;
        if !status.success() {
            anyhow::bail!("Failed to mux metadata");
        }
        Ok(())
    }

    pub fn print(&self, format: Option<&str>) {
        match format {
            None => self.print_human_readable_format(),
            Some("x265") => self.print_x265_args(),
            Some("rav1e") => self.print_rav1e_args(),
            Some("mkvmerge") => self.print_mkvmerge_args(),
            _ => unreachable!("Unimplemented output format"),
        }
    }

    fn print_human_readable_format(&self) {
        if let Some(ref basic) = self.basic {
            println!("Color Range: {}", print_color_range(basic.range));
            println!(
                "Color Primaries: {}",
                print_color_primaries(basic.primaries)
            );
            println!(
                "Transfer Characteristics: {}",
                print_transfer_characteristics(basic.transfer)
            );
            println!(
                "Matrix Coefficients: {}",
                print_matrix_coefficients(basic.matrix)
            );
        }
        if let Some(ref hdr_data) = self.hdr {
            println!("Max Content Light Level: {}", hdr_data.max_content_light);
            println!(
                "Max Frame-Average Light Level: {}",
                hdr_data.max_frame_light
            );
            println!("Maximum Luminance: {}", hdr_data.max_luma);
            println!("Minimum Luminance: {}", hdr_data.min_luma);
            if let Some(ref color_coords) = hdr_data.color_coords {
                println!(
                    "Red Coordinates: {:.5}, {:.5}",
                    color_coords.red.0, color_coords.red.1
                );
                println!(
                    "Green Coordinates: {:.5}, {:.5}",
                    color_coords.green.0, color_coords.green.1
                );
                println!(
                    "Blue Coordinates: {:.5}, {:.5}",
                    color_coords.blue.0, color_coords.blue.1
                );
                println!(
                    "White Point Coordinates: {:.5}, {:.5}",
                    color_coords.white.0, color_coords.white.1
                );
            }
        }
    }

    fn print_x265_args(&self) {
        println!(
            "{}{}",
            if let Some(ref basic) = self.basic {
                format!(
                    "--range {} --colorprim {} --transfer {} --colormatrix {}",
                    print_x265_color_range(basic.range),
                    print_x265_color_primaries(basic.primaries),
                    print_x265_transfer_characteristics(basic.transfer),
                    print_x265_matrix_coefficients(basic.matrix)
                )
            } else {
                String::new()
            },
            if let Some(ref hdr_data) = self.hdr {
                format!(
                    " --max-luma {} --min-luma {:.4} --max-cll {},{} --master-display {}",
                    hdr_data.max_luma,
                    hdr_data.min_luma,
                    hdr_data.max_content_light,
                    hdr_data.max_frame_light,
                    format_master_display(
                        hdr_data.color_coords.as_ref().unwrap(),
                        hdr_data.max_luma,
                        hdr_data.min_luma
                    )
                )
            } else {
                String::new()
            }
        );
    }

    fn print_rav1e_args(&self) {
        println!(
            "{}{}",
            if let Some(ref basic) = self.basic {
                format!(
                    "--range {} --primaries {} --transfer {} --matrix {}",
                    print_rav1e_color_range(basic.range),
                    print_rav1e_color_primaries(basic.primaries),
                    print_rav1e_transfer_characteristics(basic.transfer),
                    print_rav1e_matrix_coefficients(basic.matrix)
                )
            } else {
                String::new()
            },
            if let Some(ref hdr_data) = self.hdr {
                format!(
                    " --content-light {},{}{}",
                    hdr_data.max_content_light,
                    hdr_data.max_frame_light,
                    format_master_display(
                        hdr_data.color_coords.as_ref().unwrap(),
                        hdr_data.max_luma,
                        hdr_data.min_luma
                    )
                )
            } else {
                String::new()
            }
        );
    }

    // This is a bit different and weird compared to the other print functions.
    // The reason is to reduce code duplication, since we also use mkvmerge
    // for muxing.
    fn print_mkvmerge_args(&self) {
        let output = format!("{:?}", self.build_mkvmerge_command(Path::new("NUL"), None));
        println!(
            "{}",
            output
                .replace('"', "")
                .trim_start_matches("mkvmerge -o NUL ")
                .trim_end_matches(" NUL")
        );
    }

    fn build_mkvmerge_command(&self, target: &Path, chapters: Option<&Path>) -> Command {
        let mut command = Command::new("mkvpropedit");
        command.arg("-e").arg("track:v1");
        if let Some(ref basic) = self.basic {
            command
                .arg("-s")
                .arg(format!(
                    "colour-range={}",
                    color_range_to_mkvedit_prop(basic.range)
                ))
                .arg("-s")
                .arg(format!(
                    "colour-transfer-characteristics={}",
                    basic.transfer
                ))
                .arg("-s")
                .arg(format!("colour-primaries={}", basic.primaries))
                .arg("-s")
                .arg(format!("colour-matrix-coefficients={}", basic.matrix));
        }
        if let Some(ref hdr_data) = self.hdr {
            if hdr_data.max_content_light > 0 {
                command
                    .arg("-s")
                    .arg(format!("max-content-light={}", hdr_data.max_content_light));
            }
            if hdr_data.max_frame_light > 0 {
                command
                    .arg("-s")
                    .arg(format!("max-frame-light={}", hdr_data.max_frame_light));
            }
            command
                .arg("-s")
                .arg(format!("max-luminance={}", hdr_data.max_luma))
                .arg("-s")
                .arg(format!("min-luminance={:.4}", hdr_data.min_luma));
            if let Some(ref color_coords) = hdr_data.color_coords {
                command
                    .arg("-s")
                    .arg(format!(
                        "chromaticity-coordinates-red-x={:.5}",
                        color_coords.red.0
                    ))
                    .arg("-s")
                    .arg(format!(
                        "chromaticity-coordinates-red-y={:.5}",
                        color_coords.red.1
                    ))
                    .arg("-s")
                    .arg(format!(
                        "chromaticity-coordinates-green-x={:.5}",
                        color_coords.green.0
                    ))
                    .arg("-s")
                    .arg(format!(
                        "chromaticity-coordinates-green-y={:.5}",
                        color_coords.green.1
                    ))
                    .arg("-s")
                    .arg(format!(
                        "chromaticity-coordinates-blue-x={:.5}",
                        color_coords.blue.0
                    ))
                    .arg("-s")
                    .arg(format!(
                        "chromaticity-coordinates-blue-y={:.5}",
                        color_coords.blue.1
                    ))
                    .arg("-s")
                    .arg(format!("white-coordinates-x={:.5}", color_coords.white.0))
                    .arg("-s")
                    .arg(format!("white-coordinates-y={:.5}", color_coords.white.1));
            }
        }
        if let Some(chapters) = chapters {
            command.arg("-c").arg(chapters);
        }
        command.arg(target);
        command
    }
}

fn format_master_display(coords: &ColorCoordinates, max_luma: u32, min_luma: f64) -> String {
    format!(
        "G({},{})B({},{})R({},{})WP({},{})L({},{})",
        (coords.green.0 * 50000.).round() as u32,
        (coords.green.1 * 50000.).round() as u32,
        (coords.blue.0 * 50000.).round() as u32,
        (coords.blue.1 * 50000.).round() as u32,
        (coords.red.0 * 50000.).round() as u32,
        (coords.red.1 * 50000.).round() as u32,
        (coords.white.0 * 50000.).round() as u32,
        (coords.white.1 * 50000.).round() as u32,
        max_luma * 50000,
        (min_luma * 50000.).round() as u32,
    )
}

pub fn extract_chapters(input: &Path) -> Option<PathBuf> {
    let output = input.with_extension("hdrcp_chapters.xml");
    let result = Command::new("mkvextract")
        .arg(input)
        .arg("chapters")
        .arg(&output)
        .status();
    if result.is_ok() && output.exists() && output.metadata().expect("File exists").len() > 0 {
        Some(output)
    } else {
        None
    }
}
