pub fn parse_color_range(value: &str) -> u8 {
    match value.to_lowercase().as_str() {
        "limited" => 1,
        "full" => 0,
        _ => panic!("Unrecognized color range"),
    }
}

pub fn print_color_range(value: u8) -> &'static str {
    match value {
        0 => "Full",
        1 => "Limited",
        _ => panic!("Unrecognized color range"),
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

pub fn print_x265_color_range(value: u8) -> &'static str {
    match value {
        0 => "full",
        1 => "limited",
        _ => panic!("Unrecognized color range"),
    }
}

pub fn print_rav1e_color_range(value: u8) -> &'static str {
    match value {
        0 => "Full",
        1 => "Limited",
        _ => panic!("Unrecognized color range"),
    }
}

pub fn parse_matrix_coefficients(value: &str) -> u8 {
    match value.to_lowercase().as_str() {
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
    }
}

pub fn print_matrix_coefficients(value: u8) -> &'static str {
    match value {
        0 => "RGB",
        1 => "BT.709",
        2 => "Unspecified",
        4 => "FCC",
        5 => "BT.470 BG",
        6 => "SMPTE 170m/BT.601",
        7 => "SMPTE 240m",
        8 => "YCgCo",
        9 => "BT.2020 Non-Constant Light",
        10 => "BT.2020 Constant Light",
        12 => "Chroma-Derived Non-Constant Light",
        13 => "Chroma-Derived Constant Light",
        _ => panic!("Unrecognized matrix coefficients"),
    }
}

pub fn print_x265_matrix_coefficients(value: u8) -> &'static str {
    match value {
        0 => panic!("RGB not supported by x265"),
        1 => "bt709",
        2 => "unknown",
        4 => "fcc",
        5 => "bt470bg",
        6 => "smpte170m",
        7 => "smpte240m",
        8 => "ycgco",
        9 => "bt2020nc",
        10 => "bt2020c",
        12 => "chroma-derived-nc",
        13 => "chroma-derived-c",
        // FIXME: The following are x265 options with an unknown number value
        // gbr
        // smpte2085
        // ictcp
        _ => panic!("Unrecognized matrix coefficients"),
    }
}

pub fn print_rav1e_matrix_coefficients(value: u8) -> &'static str {
    match value {
        0 => panic!("RGB not supported by rav1e"),
        1 => "BT709",
        2 => "Unspecified",
        4 => "FCC",
        5 => "BT470BG",
        6 => "BT601",
        7 => "SMPTE240",
        8 => "YCgCo",
        9 => "BT2020NCL",
        10 => "BT2020CL",
        12 => "ChromatNCL",
        13 => "ChromatCL",
        // FIXME: The following are rav1e options with an unknown number value
        // Identity
        // SMPTE2085
        // ICtCp
        _ => panic!("Unrecognized matrix coefficients"),
    }
}

pub fn parse_transfer_characteristics(value: &str) -> u8 {
    match value.to_lowercase().as_str() {
        "bt.709" => 1,
        "unspecified" | "unset" => 2,
        "bt.470 m" => 4,
        "bt.470 bg" => 5,
        "bt.601" => 6,
        "smpte 240m" => 7,
        "linear" => 8,
        "log 100" => 9,
        "log 316" => 10,
        "iec 61966-2-4" => 11,
        "iec 61966-2-1" => 13,
        "bt.2020 10-bit" => 14,
        "bt.2020 12-bit" => 15,
        "pq" | "smpte 2084" => 16,
        "arib b67" => 18,
        _ => panic!("Unrecognized transfer characteristics"),
    }
}

pub fn print_transfer_characteristics(value: u8) -> &'static str {
    match value {
        1 => "BT.709",
        2 => "Unspecified",
        4 => "BT.470 M",
        5 => "BT.470 BG",
        6 => "SMPTE 170m/BT.601",
        7 => "SMPTE 240m",
        8 => "Linear",
        9 => "Log 100",
        10 => "Log 316",
        11 => "IEC 61966-2-4",
        13 => "IEC 61966-2-1",
        14 => "BT.2020 10-bit",
        15 => "BT.2020 12-bit",
        16 => "PQ/SMPTE 2084",
        18 => "ARIB B67",
        _ => panic!("Unrecognized transfer characteristics"),
    }
}

pub fn print_x265_transfer_characteristics(value: u8) -> &'static str {
    match value {
        1 => "bt709",
        2 => "unknown",
        4 => "bt470m",
        5 => "bt470bg",
        6 => "smpte170m",
        7 => "smpte240m",
        8 => "linear",
        9 => "log100",
        10 => "log316",
        11 => "iec61966-2-4",
        13 => "iec61966-2-1",
        14 => "bt2020-10",
        15 => "bt2020-12",
        16 => "smpte2084",
        18 => "arib-std-b67",
        // FIXME: The following are x265 options with an unknown number value
        // bt1361e
        // smpte428
        _ => panic!("Unrecognized transfer characteristics"),
    }
}

pub fn print_rav1e_transfer_characteristics(value: u8) -> &'static str {
    match value {
        1 => "BT709",
        2 => "Unspecified",
        4 => "BT470M",
        5 => "BT470BG",
        6 => "BT601",
        7 => "SMPTE240",
        8 => "Linear",
        9 => "Log100",
        10 => "Log100Sqrt10",
        11 => "IEC61966",
        13 => "SRGB",
        14 => "BT2020_10Bit",
        15 => "BT2020_12Bit",
        16 => "SMPTE2084",
        18 => panic!("ARIB B67 not supported by rav1e"),
        // FIXME: The following are rav1e options with an unknown number value
        // BT1361
        // SMPTE428
        // HLG
        _ => panic!("Unrecognized transfer characteristics"),
    }
}

pub fn parse_color_primaries(value: &str) -> u8 {
    match value.to_lowercase().as_str() {
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
    }
}

pub fn print_color_primaries(value: u8) -> &'static str {
    match value {
        1 => "BT.709",
        2 => "Unspecified",
        4 => "BT.470 M",
        5 => "BT.470 BG",
        6 => "SMPTE 170m/BT.601",
        7 => "SMPTE 240m",
        8 => "Film",
        9 => "BT.2020",
        10 => "SMPTE 428",
        11 => "SMPTE 431.2",
        12 => "SMPTE 432.1",
        22 => "EBU 3213 E",
        _ => panic!("Unrecognized color primaries"),
    }
}

pub fn print_x265_color_primaries(value: u8) -> &'static str {
    match value {
        1 => "bt709",
        2 => "unknown",
        4 => "bt470m",
        5 => "bt470bg",
        6 => "smpte170m",
        7 => "smpte240m",
        8 => "film",
        9 => "bt2020",
        10 => "smpte428",
        11 => "smpte431",
        12 => "smpte432",
        22 => panic!("EBU 3213 E not supported by x265"),
        _ => panic!("Unrecognized color primaries"),
    }
}

pub fn print_rav1e_color_primaries(value: u8) -> &'static str {
    match value {
        1 => "BT709",
        2 => "Unspecified",
        4 => "BT470M",
        5 => "BT470BG",
        6 => "BT601",
        7 => "SMPTE240",
        8 => "GenericFilm",
        9 => "BT2020",
        10 => "XYZ",
        11 => "SMPTE431",
        12 => "SMPTE432",
        22 => "EBU3213",
        // FIXME: The following are rav1e options with an unknown number value
        // XYZ
        _ => panic!("Unrecognized color primaries"),
    }
}
