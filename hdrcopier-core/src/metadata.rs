use std::{
    fmt::Display,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;

use crate::{
    parse::{parse_ffprobe, parse_mediainfo, parse_mkvinfo},
    values::{
        color_range_to_mkvedit_prop, print_color_primaries, print_color_range,
        print_matrix_coefficients, print_rav1e_color_primaries, print_rav1e_color_range,
        print_rav1e_matrix_coefficients, print_rav1e_transfer_characteristics,
        print_svtav1_chroma_location, print_svtav1_color_primaries, print_svtav1_color_range,
        print_svtav1_matrix_coefficients, print_svtav1_transfer_characteristics,
        print_transfer_characteristics, print_x265_chroma_location, print_x265_color_primaries,
        print_x265_color_range, print_x265_matrix_coefficients,
        print_x265_transfer_characteristics,
    },
};

#[derive(Debug, Clone, Copy, Default)]
pub struct Metadata {
    pub basic: Option<BasicMetadata>,
    pub hdr: Option<HdrMetadata>,
}

#[derive(Debug, Clone, Copy)]
pub struct BasicMetadata {
    pub matrix: u8,
    pub transfer: u8,
    pub primaries: u8,
    pub range: u8,
    pub chroma_location: ChromaLocation,
}

impl Default for BasicMetadata {
    fn default() -> Self {
        Self {
            matrix: 2,
            transfer: 2,
            primaries: 2,
            range: 1,
            chroma_location: ChromaLocation::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ChromaLocation {
    #[default]
    Left = 0,
    Center = 1,
    TopLeft = 2,
    Top = 3,
    BottomLeft = 4,
    Bottom = 5,
}

impl ChromaLocation {
    pub fn get_horiz(&self) -> u8 {
        // 0 = unspecified
        // 1 = left collocated
        // 2 = half
        match self {
            ChromaLocation::Left | ChromaLocation::TopLeft | ChromaLocation::BottomLeft => 1,
            ChromaLocation::Center | ChromaLocation::Top | ChromaLocation::Bottom => 2,
        }
    }

    pub fn get_vert(&self) -> u8 {
        // 0 = unspecified
        // 1 = top collocated
        // 2 = half
        // 3 = bottom collocated -- I'm not entirely certain if the mkv spec supports this?
        match self {
            ChromaLocation::TopLeft | ChromaLocation::Top => 1,
            ChromaLocation::Left | ChromaLocation::Center => 2,
            ChromaLocation::BottomLeft | ChromaLocation::Bottom => 3,
        }
    }
}

impl Display for ChromaLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ChromaLocation::Left => "Left",
                ChromaLocation::Center => "Center",
                ChromaLocation::TopLeft => "Top-Left",
                ChromaLocation::Top => "Top",
                ChromaLocation::BottomLeft => "Bottom-Left",
                ChromaLocation::Bottom => "Bottom",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ColorCoordinates {
    pub red: (f64, f64),
    pub green: (f64, f64),
    pub blue: (f64, f64),
    pub white: (f64, f64),
}

#[derive(Debug, Clone, Copy, Default)]
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
            Some("svt-av1") => self.print_svtav1_args(),
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
            println!("Chroma Position: {}", basic.chroma_location);
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
                    "--range {} --colorprim {} --transfer {} --colormatrix {} --chromaloc {}",
                    print_x265_color_range(basic.range),
                    print_x265_color_primaries(basic.primaries),
                    print_x265_transfer_characteristics(basic.transfer),
                    print_x265_matrix_coefficients(basic.matrix),
                    print_x265_chroma_location(basic.chroma_location),
                )
            } else {
                String::new()
            },
            if let Some(ref hdr_data) = self.hdr {
                format!(
                    " {}{} --max-cll {},{} --master-display {}",
                    if hdr_data.max_luma > 0 {
                        format!("--max-luma {} ", hdr_data.max_luma)
                    } else {
                        String::new()
                    },
                    if hdr_data.min_luma > 0.0 {
                        format!("--min-luma {} ", hdr_data.min_luma.round() as u32)
                    } else {
                        String::new()
                    },
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

    fn print_svtav1_args(&self) {
        println!(
            "{}{}",
            if let Some(ref basic) = self.basic {
                format!(
                    "--color-range {} --color-primaries {} --transfer-characteristics {} \
                     --matrix-coefficients {} --chroma-sample-position {}",
                    print_svtav1_color_range(basic.range),
                    print_svtav1_color_primaries(basic.primaries),
                    print_svtav1_transfer_characteristics(basic.transfer),
                    print_svtav1_matrix_coefficients(basic.matrix),
                    print_svtav1_chroma_location(basic.chroma_location),
                )
            } else {
                String::new()
            },
            if let Some(ref hdr_data) = self.hdr {
                format!(
                    " --content-light {},{} --mastering-display \
                     G({},{})B({},{})R({},{})WP({},{})L({},{})",
                    hdr_data.max_content_light,
                    hdr_data.max_frame_light,
                    hdr_data.color_coords.as_ref().unwrap().green.0,
                    hdr_data.color_coords.as_ref().unwrap().green.1,
                    hdr_data.color_coords.as_ref().unwrap().blue.0,
                    hdr_data.color_coords.as_ref().unwrap().blue.1,
                    hdr_data.color_coords.as_ref().unwrap().red.0,
                    hdr_data.color_coords.as_ref().unwrap().red.1,
                    hdr_data.color_coords.as_ref().unwrap().white.0,
                    hdr_data.color_coords.as_ref().unwrap().white.1,
                    hdr_data.max_luma,
                    hdr_data.min_luma,
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
                // rav1e does not support a chroma location parameter
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
                    " --content-light {},{} --mastering-display {}",
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
            if basic.transfer != 2 {
                command.arg("-s").arg(format!(
                    "colour-transfer-characteristics={}",
                    basic.transfer
                ));
            }
            if basic.primaries != 2 {
                command
                    .arg("-s")
                    .arg(format!("colour-primaries={}", basic.primaries));
            }
            if basic.matrix != 2 {
                command
                    .arg("-s")
                    .arg(format!("colour-matrix-coefficients={}", basic.matrix));
            }
            command.arg("-s").arg(format!(
                "colour-range={}",
                color_range_to_mkvedit_prop(basic.range)
            ));
            command
                .arg("-s")
                .arg(format!(
                    "chroma-siting-horizontal={}",
                    basic.chroma_location.get_horiz()
                ))
                .arg("-s")
                .arg(format!(
                    "chroma-siting-vertical={}",
                    basic.chroma_location.get_vert()
                ));
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
        max_luma * 10000,
        (min_luma * 10000.).round() as u32,
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
