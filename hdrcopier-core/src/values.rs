use crate::{Error, Result, metadata::ChromaLocation};

fn unsupported_value(kind: &'static str, value: impl ToString) -> Error {
    Error::UnsupportedValue {
        kind,
        value: value.to_string(),
    }
}

pub fn parse_color_range(value: &str) -> Result<u8> {
    match value.to_lowercase().as_str() {
        "limited" => Ok(1),
        "full" => Ok(0),
        _ => Err(unsupported_value("color range", value)),
    }
}

pub fn print_color_range(value: u8) -> Result<&'static str> {
    match value {
        0 => Ok("Full"),
        1 => Ok("Limited"),
        _ => Err(unsupported_value("color range", value)),
    }
}

pub fn color_range_to_mkvedit_prop(value: u8) -> u8 {
    // mkvpropedit uses "2" for "Full" instead of "0" which is what everyone else uses.
    // "0" to them means "unset"
    if value == 0 {
        return 2;
    }
    value
}

pub fn print_x265_color_range(value: u8) -> Result<&'static str> {
    match value {
        0 => Ok("full"),
        1 => Ok("limited"),
        _ => Err(unsupported_value("x265 color range", value)),
    }
}

pub fn print_rav1e_color_range(value: u8) -> Result<&'static str> {
    match value {
        0 => Ok("Full"),
        1 => Ok("Limited"),
        _ => Err(unsupported_value("rav1e color range", value)),
    }
}

pub fn print_svtav1_color_range(value: u8) -> Result<&'static str> {
    match value {
        0 => Ok("full"),
        1 => Ok("studio"),
        _ => Err(unsupported_value("svt-av1 color range", value)),
    }
}

pub fn parse_matrix_coefficients(value: &str) -> Result<u8> {
    match value.to_lowercase().as_str() {
        "rgb" => Ok(0),
        "bt.709" => Ok(1),
        "unspecified" | "unset" => Ok(2),
        "fcc" => Ok(4),
        "bt.470 bg" => Ok(5),
        "smpte 170m" | "bt.601" => Ok(6),
        "smpte 240m" => Ok(7),
        "ycgco" => Ok(8),
        "bt.2020 non-constant" => Ok(9),
        "bt.2020 constant" => Ok(10),
        // FIXME: Not sure how these two are formatted in mediainfo
        // VSC_MATRIX_CHROMATICITY_DERIVED_NCL = 12,
        // VSC_MATRIX_CHROMATICITY_DERIVED_CL = 13,
        _ => Err(unsupported_value("matrix coefficients", value)),
    }
}

pub fn print_matrix_coefficients(value: u8) -> Result<&'static str> {
    match value {
        0 => Ok("RGB"),
        1 => Ok("BT.709"),
        2 => Ok("Unspecified"),
        4 => Ok("FCC"),
        5 => Ok("BT.470 BG"),
        6 => Ok("SMPTE 170m/BT.601"),
        7 => Ok("SMPTE 240m"),
        8 => Ok("YCgCo"),
        9 => Ok("BT.2020 Non-Constant Light"),
        10 => Ok("BT.2020 Constant Light"),
        12 => Ok("Chroma-Derived Non-Constant Light"),
        13 => Ok("Chroma-Derived Constant Light"),
        _ => Err(unsupported_value("matrix coefficients", value)),
    }
}

pub fn print_x265_matrix_coefficients(value: u8) -> Result<&'static str> {
    match value {
        0 => Err(unsupported_value("x265 matrix coefficients", "RGB")),
        1 => Ok("bt709"),
        2 => Ok("unknown"),
        4 => Ok("fcc"),
        5 => Ok("bt470bg"),
        6 => Ok("smpte170m"),
        7 => Ok("smpte240m"),
        8 => Ok("ycgco"),
        9 => Ok("bt2020nc"),
        10 => Ok("bt2020c"),
        12 => Ok("chroma-derived-nc"),
        13 => Ok("chroma-derived-c"),
        // FIXME: The following are x265 options with an unknown number value
        // gbr
        // smpte2085
        // ictcp
        _ => Err(unsupported_value("x265 matrix coefficients", value)),
    }
}

pub fn print_x265_chroma_location(value: ChromaLocation) -> u8 {
    value as u8
}

pub fn print_svtav1_chroma_location(value: ChromaLocation) -> &'static str {
    // svt only supports left and top left, so for the others we should leave
    // it at unknown and let mkvmerge handle setting it on the container
    match value {
        ChromaLocation::Left => "left",
        ChromaLocation::TopLeft => "topleft",
        _ => "unknown",
    }
}

pub fn print_svtav1_matrix_coefficients(value: u8) -> Result<&'static str> {
    match value {
        0 => Ok("identity"),
        1 => Ok("bt709"),
        2 => Ok("unspecified"),
        4 => Ok("fcc"),
        5 => Ok("bt470bg"),
        6 => Ok("bt601"),
        7 => Ok("smpte240"),
        8 => Ok("ycgco"),
        9 => Ok("bt2020-ncl"),
        10 => Ok("bt2020-cl"),
        11 => Ok("smpte2085"),
        12 => Ok("chroma-ncl"),
        13 => Ok("chroma-cl"),
        14 => Ok("ictcp"),
        _ => Err(unsupported_value("svt-av1 matrix coefficients", value)),
    }
}

pub fn print_rav1e_matrix_coefficients(value: u8) -> Result<&'static str> {
    match value {
        0 => Err(unsupported_value("rav1e matrix coefficients", "RGB")),
        1 => Ok("BT709"),
        2 => Ok("Unspecified"),
        4 => Ok("FCC"),
        5 => Ok("BT470BG"),
        6 => Ok("BT601"),
        7 => Ok("SMPTE240"),
        8 => Ok("YCgCo"),
        9 => Ok("BT2020NCL"),
        10 => Ok("BT2020CL"),
        12 => Ok("ChromatNCL"),
        13 => Ok("ChromatCL"),
        // FIXME: The following are rav1e options with an unknown number value
        // Identity
        // SMPTE2085
        // ICtCp
        _ => Err(unsupported_value("rav1e matrix coefficients", value)),
    }
}

pub fn parse_transfer_characteristics(value: &str) -> Result<u8> {
    match value.to_lowercase().as_str() {
        "bt.709" => Ok(1),
        "unspecified" | "unset" => Ok(2),
        "bt.470 m" => Ok(4),
        "bt.470 bg" => Ok(5),
        "bt.601" => Ok(6),
        "smpte 240m" => Ok(7),
        "linear" => Ok(8),
        "log 100" => Ok(9),
        "log 316" => Ok(10),
        "iec 61966-2-4" => Ok(11),
        "iec 61966-2-1" => Ok(13),
        "bt.2020 10-bit" => Ok(14),
        "bt.2020 12-bit" => Ok(15),
        "pq" | "smpte 2084" => Ok(16),
        "arib b67" => Ok(18),
        "hlg" => Ok(19),
        _ => Err(unsupported_value("transfer characteristics", value)),
    }
}

pub fn print_transfer_characteristics(value: u8) -> Result<&'static str> {
    match value {
        1 => Ok("BT.709"),
        2 => Ok("Unspecified"),
        4 => Ok("BT.470 M"),
        5 => Ok("BT.470 BG"),
        6 => Ok("SMPTE 170m/BT.601"),
        7 => Ok("SMPTE 240m"),
        8 => Ok("Linear"),
        9 => Ok("Log 100"),
        10 => Ok("Log 316"),
        11 => Ok("IEC 61966-2-4"),
        13 => Ok("IEC 61966-2-1"),
        14 => Ok("BT.2020 10-bit"),
        15 => Ok("BT.2020 12-bit"),
        16 => Ok("PQ/SMPTE 2084"),
        18 => Ok("ARIB B67"),
        19 => Ok("HLG"),
        _ => Err(unsupported_value("transfer characteristics", value)),
    }
}

pub fn print_x265_transfer_characteristics(value: u8) -> Result<&'static str> {
    match value {
        1 => Ok("bt709"),
        2 => Ok("unknown"),
        4 => Ok("bt470m"),
        5 => Ok("bt470bg"),
        6 => Ok("smpte170m"),
        7 => Ok("smpte240m"),
        8 => Ok("linear"),
        9 => Ok("log100"),
        10 => Ok("log316"),
        11 => Ok("iec61966-2-4"),
        13 => Ok("iec61966-2-1"),
        14 => Ok("bt2020-10"),
        15 => Ok("bt2020-12"),
        16 => Ok("smpte2084"),
        18 => Ok("arib-std-b67"),
        // FIXME: The following are x265 options with an unknown number value
        // bt1361e
        // smpte428
        _ => Err(unsupported_value("x265 transfer characteristics", value)),
    }
}

pub fn print_svtav1_transfer_characteristics(value: u8) -> Result<&'static str> {
    match value {
        1 => Ok("bt709"),
        2 => Ok("unspecified"),
        4 => Ok("bt470m"),
        5 => Ok("bt470bg"),
        6 => Ok("bt601"),
        7 => Ok("smpte240"),
        8 => Ok("linear"),
        9 => Ok("log100"),
        10 => Ok("log100-sqrt10"),
        11 => Ok("iec61966"),
        12 => Ok("bt1361"),
        13 => Ok("srgb"),
        14 => Ok("bt2020-10"),
        15 => Ok("bt2020-12"),
        16 => Ok("smpte2084"),
        17 => Ok("smpte428"),
        19 => Ok("hlg"),
        _ => Err(unsupported_value("svt-av1 transfer characteristics", value)),
    }
}

pub fn print_rav1e_transfer_characteristics(value: u8) -> Result<&'static str> {
    match value {
        1 => Ok("BT709"),
        2 => Ok("Unspecified"),
        4 => Ok("BT470M"),
        5 => Ok("BT470BG"),
        6 => Ok("BT601"),
        7 => Ok("SMPTE240"),
        8 => Ok("Linear"),
        9 => Ok("Log100"),
        10 => Ok("Log100Sqrt10"),
        11 => Ok("IEC61966"),
        13 => Ok("SRGB"),
        14 => Ok("BT2020_10Bit"),
        15 => Ok("BT2020_12Bit"),
        16 => Ok("SMPTE2084"),
        18 => Err(unsupported_value(
            "rav1e transfer characteristics",
            "ARIB B67",
        )),
        // FIXME: The following are rav1e options with an unknown number value
        // BT1361
        // SMPTE428
        // HLG
        _ => Err(unsupported_value("rav1e transfer characteristics", value)),
    }
}

pub fn parse_color_primaries(value: &str) -> Result<u8> {
    match value.to_lowercase().as_str() {
        "bt.709" => Ok(1),
        "unspecified" | "unset" => Ok(2),
        "bt.470 m" => Ok(4),
        "bt.470 bg" => Ok(5),
        "smpte 170m" | "bt.601" | "bt.601 ntsc" => Ok(6),
        "smpte 240m" => Ok(7),
        "film" | "ntsc" => Ok(8),
        "bt.2020" => Ok(9),
        "smpte 428" => Ok(10),
        "smpte 431.2" => Ok(11),
        "smpte 432.1" => Ok(12),
        "ebu 3213 e" => Ok(22),
        _ => Err(unsupported_value("color primaries", value)),
    }
}

pub fn print_color_primaries(value: u8) -> Result<&'static str> {
    match value {
        1 => Ok("BT.709"),
        2 => Ok("Unspecified"),
        4 => Ok("BT.470 M"),
        5 => Ok("BT.470 BG"),
        6 => Ok("SMPTE 170m/BT.601"),
        7 => Ok("SMPTE 240m"),
        8 => Ok("Film"),
        9 => Ok("BT.2020"),
        10 => Ok("SMPTE 428"),
        11 => Ok("SMPTE 431.2"),
        12 => Ok("SMPTE 432.1"),
        22 => Ok("EBU 3213 E"),
        _ => Err(unsupported_value("color primaries", value)),
    }
}

pub fn print_x265_color_primaries(value: u8) -> Result<&'static str> {
    match value {
        1 => Ok("bt709"),
        2 => Ok("unknown"),
        4 => Ok("bt470m"),
        5 => Ok("bt470bg"),
        6 => Ok("smpte170m"),
        7 => Ok("smpte240m"),
        8 => Ok("film"),
        9 => Ok("bt2020"),
        10 => Ok("smpte428"),
        11 => Ok("smpte431"),
        12 => Ok("smpte432"),
        22 => Err(unsupported_value("x265 color primaries", "EBU 3213 E")),
        _ => Err(unsupported_value("x265 color primaries", value)),
    }
}

pub fn print_svtav1_color_primaries(value: u8) -> Result<&'static str> {
    match value {
        1 => Ok("bt709"),
        2 => Ok("unspecified"),
        4 => Ok("bt470m"),
        5 => Ok("bt470bg"),
        6 => Ok("bt601"),
        7 => Ok("smpte240"),
        8 => Ok("film"),
        9 => Ok("bt2020"),
        10 => Ok("xyz"),
        11 => Ok("smpte431"),
        12 => Ok("smpte432"),
        22 => Ok("ebu3213"),
        _ => Err(unsupported_value("svt-av1 color primaries", value)),
    }
}

pub fn print_rav1e_color_primaries(value: u8) -> Result<&'static str> {
    match value {
        1 => Ok("BT709"),
        2 => Ok("Unspecified"),
        4 => Ok("BT470M"),
        5 => Ok("BT470BG"),
        6 => Ok("BT601"),
        7 => Ok("SMPTE240"),
        8 => Ok("GenericFilm"),
        9 => Ok("BT2020"),
        10 => Ok("XYZ"),
        11 => Ok("SMPTE431"),
        12 => Ok("SMPTE432"),
        22 => Ok("EBU3213"),
        // FIXME: The following are rav1e options with an unknown number value
        // XYZ
        _ => Err(unsupported_value("rav1e color primaries", value)),
    }
}
