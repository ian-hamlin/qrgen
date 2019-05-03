mod chunker;
mod exporter;
mod generator;

use env_logger::Env;
use log::{info, trace};
use qrcodegen;
use std::{env, ffi::OsStr, path::PathBuf};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    /// Input file, must be specified.
    #[structopt(name = "infile", parse(from_os_str), raw(required = "true"))]
    infile: Vec<PathBuf>,

    /// Output path, or current working directory if not specified or - provided.
    #[structopt(
        short = "o",
        long = "output",
        default_value = "-",
        parse(from_os_str = "parse_output_directory")
    )]
    output: PathBuf,

    /// The minimum version number supported in the QR Code Model 2 standard, or 1 if not specified.
    #[structopt(
        name = "QR version min",
        short = "m",
        long = "min",
        default_value = "1",
        parse(try_from_str = "parse_qr_version")
    )]
    qr_version_min: qrcodegen::Version,

    /// The maximum version number supported in the QR Code Model 2 standard, or 40 if not specified.
    #[structopt(
        name = "QR version max",
        short = "x",
        long = "max",
        default_value = "40",
        parse(try_from_str = "parse_qr_version")
    )]
    qr_version_max: qrcodegen::Version,

    /// The error correction level used in this QR Code, or High if not specified.
    /// "Low" The QR Code can tolerate about  7% erroneous codewords.
    /// "Medium" The QR Code can tolerate about 15% erroneous codewords.
    /// "Quartile" The QR Code can tolerate about 25% erroneous codewords.
    /// "High" The QR Code can tolerate about 30% erroneous codewords.
    #[structopt(
        name = "error correction",
        short = "e",
        long = "error",
        default_value = "High",
        parse(try_from_str = "parse_qr_ecc")
    )]
    error_correction: qrcodegen::QrCodeEcc,

    /// The number of lines to try and process in parallel, if not specified defaults to 1 and file is processed line by
    /// line.
    #[structopt(
        name = "chunk size",
        short = "c",
        long = "chunk",
        default_value = "1",
        parse(try_from_str = "parse_chunk_size")
    )]
    chunk_size: usize,

    /// A flag indicating if the first line of the CSV is a header and should be skipped, defaults to false if not
    /// specified.
    #[structopt(name = "has headers", short = "s", long = "skip")]
    has_headers: bool,

    /// A flag indicating if output will be logged, defaults to false if not specified.
    #[structopt(short = "l", long = "log")]
    log: bool,

    /// Verbose logging mode (-v, -vv, -vvv)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,

    /// The size of the border on the generated QR Code, defaults to 4 if not specified.
    #[structopt(short = "b", long = "border", default_value = "4")]
    border: u8,

    /// The mask value to apply to the QR Code, between 0 and 7 (inclusive).
    #[structopt(
        name = "mask",
        short = "k",
        long = "mask",
        parse(try_from_str = "parse_qr_mask")
    )]
    mask: Option<qrcodegen::Mask>,

    /// The target output format.  Defaults to SVG if not specified.
    #[structopt(
        name = "Output format type",
        short = "f",
        long = "format",
        default_value = "SVG",
        parse(try_from_str = "parse_qr_format")
    )]
    format: exporter::ExportFormat,

    /// The side length (measured in pixels, must be positive) of each module, defaults to 8.  
    /// This value only applies when using the PNG format.
    /// Must be between 1 and 255 (inclusive)
    #[structopt(
        short = "a",
        long = "scale",
        default_value = "8",
        parse(try_from_str = "parse_qr_scale")
    )]
    scale: u8,
}

fn parse_output_directory(src: &OsStr) -> PathBuf {
    if src == "-" {
        return env::current_dir().expect("Unable to access current working directory.");
    }

    PathBuf::from(src)
}

fn parse_qr_format(src: &str) -> Result<exporter::ExportFormat, String> {
    let src = src.to_uppercase();

    match src.as_ref() {
        "SVG" => Ok(exporter::ExportFormat::SVG),
        "PNG" => Ok(exporter::ExportFormat::PNG),
        _ => Err(String::from("Format must be either SVG or PNG.")),
    }
}

fn parse_qr_ecc(src: &str) -> Result<qrcodegen::QrCodeEcc, String> {
    let src = src.to_uppercase();

    match src.as_ref() {
        "HIGH" => Ok(qrcodegen::QrCodeEcc::High),
        "LOW" => Ok(qrcodegen::QrCodeEcc::Low),
        "MEDIUM" => Ok(qrcodegen::QrCodeEcc::Medium),
        "QUARTILE" => Ok(qrcodegen::QrCodeEcc::Quartile),
        _ => Err(String::from(
            "QR Code error correction level must be either High, Quartile, Medium or Low.",
        )),
    }
}

fn parse_qr_version(src: &str) -> Result<qrcodegen::Version, String> {
    let input = src.parse::<u8>();

    match input {
        Ok(x) if x > 0 && x < 41 => Ok(qrcodegen::Version::new(x)),
        _ => Err(String::from(
            "QR Code Model 2 version number must be between 1 and 40 inclusive.",
        )),
    }
}

fn parse_qr_mask(src: &str) -> Result<qrcodegen::Mask, String> {
    let input = src.parse::<u8>();

    match input {
        Ok(x) if x < 8 => Ok(qrcodegen::Mask::new(x)),
        _ => Err(String::from("QR mask must be between 0 and 7 inclusive.")),
    }
}

fn parse_chunk_size(src: &str) -> Result<usize, String> {
    let input = src.parse::<usize>();

    match input {
        Ok(x) if x > 0 => Ok(x),
        _ => Err(String::from("Chunk size must be a number greater than 0.")),
    }
}

fn parse_qr_scale(src: &str) -> Result<u8, String> {
    let input = src.parse::<u8>();

    match input {
        Ok(x) if x > 0 => Ok(x),
        _ => Err(String::from(
            "The module scale must be a number between 1 and 255 inclusive.",
        )),
    }
}

impl Opt {
    fn into_generator(self) -> generator::Generator {
        generator::Generator::new(
            self.infile,
            generator::QrConfig::new(
                self.qr_version_min,
                self.qr_version_max,
                self.error_correction,
                self.mask,
            ),
            generator::OutputConfig::new(self.output, self.border, self.format, self.scale),
            generator::ProcessingConfig::new(self.chunk_size, self.has_headers),
        )
    }
}

fn main() {
    let opt = Opt::from_args();

    // Initialize logger
    if opt.log {
        env_logger::Builder::from_env(Env::default().default_filter_or(match opt.verbose {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        }))
        .init();
    }

    info!("qrgen start");
    let generator = opt.into_generator();
    trace!("{}", generator);
    generator.generate();
    info!("qrgen end");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_output_directory_to_cwd() {
        let expect = env::current_dir().unwrap();
        let actual = parse_output_directory(OsStr::new("-"));

        assert_eq!(expect, actual);
    }

    #[test]
    fn should_parse_qr_format_to_png() {
        let res = parse_qr_format("png").unwrap();
        assert_eq!(exporter::ExportFormat::PNG, res);
    }

    #[test]
    fn should_parse_qr_format_to_svg() {
        let res = parse_qr_format("svg").unwrap();
        assert_eq!(exporter::ExportFormat::SVG, res);
    }

    #[test]
    fn should_parse_qr_format_to_error() {
        let res = parse_qr_format("error").err();
        assert_eq!(Some("Format must be either SVG or PNG.".to_string()), res);
    }

    #[test]
    fn should_parse_qr_ecc_to_high() {
        let res = parse_qr_ecc("high").unwrap();

        match res {
            qrcodegen::QrCodeEcc::High => {}
            _ => panic!("unexpected ecc"),
        }
    }

    #[test]
    fn should_parse_qr_ecc_to_low() {
        let res = parse_qr_ecc("low").unwrap();

        match res {
            qrcodegen::QrCodeEcc::Low => {}
            _ => panic!("unexpected ecc"),
        }
    }

    #[test]
    fn should_parse_qr_ecc_to_medium() {
        let res = parse_qr_ecc("medium").unwrap();

        match res {
            qrcodegen::QrCodeEcc::Medium => {}
            _ => panic!("unexpected ecc"),
        }
    }

    #[test]
    fn should_parse_qr_ecc_to_quartile() {
        let res = parse_qr_ecc("quartile").unwrap();

        match res {
            qrcodegen::QrCodeEcc::Quartile => {}
            _ => panic!("unexpected ecc"),
        }
    }

    #[test]
    fn should_parse_qr_ecc_to_error() {
        let res = parse_qr_ecc("error").err();
        assert_eq!(
            Some(
                "QR Code error correction level must be either High, Quartile, Medium or Low."
                    .to_string()
            ),
            res
        );
    }

    #[test]
    fn should_parse_qr_version_to_error_low() {
        let res = parse_qr_version("0").err();
        assert_eq!(
            Some("QR Code Model 2 version number must be between 1 and 40 inclusive.".to_string()),
            res
        );
    }

    #[test]
    fn should_parse_qr_version_to_error_high() {
        let res = parse_qr_version("41").err();
        assert_eq!(
            Some("QR Code Model 2 version number must be between 1 and 40 inclusive.".to_string()),
            res
        );
    }

    #[test]
    fn should_parse_qr_scale_to_error_low() {
        let res = parse_qr_scale("0").err();
        assert_eq!(
            Some("The module scale must be a number between 1 and 255 inclusive.".to_string()),
            res
        );
    }

    #[test]
    fn should_parse_qr_scale_to_error_high() {
        let res = parse_qr_scale("256").err();
        assert_eq!(
            Some("The module scale must be a number between 1 and 255 inclusive.".to_string()),
            res
        );
    }

    #[test]
    fn should_parse_qr_mask_to_error_high() {
        let res = parse_qr_mask("8").err();
        assert_eq!(
            Some("QR mask must be between 0 and 7 inclusive.".to_string()),
            res
        );
    }

    #[test]
    fn should_parse_chunk_size_to_error() {
        let res = parse_chunk_size("0").err();
        assert_eq!(
            Some("Chunk size must be a number greater than 0.".to_string()),
            res
        );
    }

    #[test]
    fn should_parse_chunk_size() {
        let res = parse_chunk_size("10").unwrap();
        assert_eq!(10, res);
    }

    macro_rules! parse_qr_mask_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (input, expected) = $value;
                assert_eq!(expected, parse_qr_mask(input).unwrap().value());
            }
        )*
        }
    }

    parse_qr_mask_tests! {
        should_parse_qr_mask_0: ("0", 0),
        should_parse_qr_mask_1: ("1", 1),
        should_parse_qr_mask_2: ("2", 2),
        should_parse_qr_mask_3: ("3", 3),
        should_parse_qr_mask_4: ("4", 4),
        should_parse_qr_mask_5: ("5", 5),
        should_parse_qr_mask_6: ("6", 6),
        should_parse_qr_mask_7: ("7", 7),
    }

    macro_rules! parse_qr_version_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (input, expected) = $value;
                assert_eq!(expected, parse_qr_version(input).unwrap().value());
            }
        )*
        }
    }

    parse_qr_version_tests! {
    should_parse_qr_version_1: ("1", 1),
    should_parse_qr_version_2: ("2", 2),
    should_parse_qr_version_3: ("3", 3),
    should_parse_qr_version_4: ("4", 4),
    should_parse_qr_version_5: ("5", 5),
    should_parse_qr_version_6: ("6", 6),
    should_parse_qr_version_7: ("7", 7),
    should_parse_qr_version_8: ("8", 8),
    should_parse_qr_version_9: ("9", 9),
    should_parse_qr_version_10: ("10", 10),
    should_parse_qr_version_11: ("11", 11),
    should_parse_qr_version_12: ("12", 12),
    should_parse_qr_version_13: ("13", 13),
    should_parse_qr_version_14: ("14", 14),
    should_parse_qr_version_15: ("15", 15),
    should_parse_qr_version_16: ("16", 16),
    should_parse_qr_version_17: ("17", 17),
    should_parse_qr_version_18: ("18", 18),
    should_parse_qr_version_19: ("19", 19),
    should_parse_qr_version_20: ("20", 20),
    should_parse_qr_version_21: ("21", 21),
    should_parse_qr_version_22: ("22", 22),
    should_parse_qr_version_23: ("23", 23),
    should_parse_qr_version_24: ("24", 24),
    should_parse_qr_version_25: ("25", 25),
    should_parse_qr_version_26: ("26", 26),
    should_parse_qr_version_27: ("27", 27),
    should_parse_qr_version_28: ("28", 28),
    should_parse_qr_version_29: ("29", 29),
    should_parse_qr_version_30: ("30", 30),
    should_parse_qr_version_31: ("31", 31),
    should_parse_qr_version_32: ("32", 32),
    should_parse_qr_version_33: ("33", 33),
    should_parse_qr_version_34: ("34", 34),
    should_parse_qr_version_35: ("35", 35),
    should_parse_qr_version_36: ("36", 36),
    should_parse_qr_version_37: ("37", 37),
    should_parse_qr_version_38: ("38", 38),
    should_parse_qr_version_39: ("39", 39),
    should_parse_qr_version_40: ("40", 40),
        }
}
