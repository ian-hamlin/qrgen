mod generator;
mod settings;

use env_logger::Env;
use log::{debug, info};
use qrcodegen::QrCodeEcc;
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
    qr_version_min: u8,

    /// The maximum version number supported in the QR Code Model 2 standard, or 40 if not specified.
    #[structopt(
        name = "QR version max",
        short = "x",
        long = "max",
        default_value = "40",
        parse(try_from_str = "parse_qr_version")
    )]
    qr_version_max: u8,

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
    error_correction: QrCodeEcc,

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
    mask: Option<u8>,
}

fn parse_output_directory(src: &OsStr) -> PathBuf {
    if src == "-" {
        return env::current_dir().expect("Unable to access current working directory.");
    }

    PathBuf::from(src)
}

fn parse_qr_ecc(src: &str) -> Result<QrCodeEcc, String> {
    let src = src.to_uppercase();

    match src.as_ref() {
        "HIGH" => Ok(QrCodeEcc::High),
        "LOW" => Ok(QrCodeEcc::Low),
        "MEDIUM" => Ok(QrCodeEcc::Medium),
        "QUARTILE" => Ok(QrCodeEcc::Quartile),
        _ => Err(String::from(
            "QR Code error correction level must be either High, Quartile, Medium or Low.",
        )),
    }
}

fn parse_qr_version(src: &str) -> Result<u8, String> {
    let input = src.parse::<u8>();

    match input {
        Ok(x) if x > 0_u8 && x < 41_u8 => Ok(x),
        _ => Err(String::from(
            "QR Code Model 2 version number must be between 1 and 40 inclusive.",
        )),
    }
}

fn parse_qr_mask(src: &str) -> Result<u8, String> {
    let input = src.parse::<u8>();

    match input {
        Ok(x) if x < 8_u8 => Ok(x),
        _ => Err(String::from("QR mask must be between 1 and 7 inclusive.")),
    }
}

fn parse_chunk_size(src: &str) -> Result<usize, String> {
    let input = src.parse::<usize>();

    match input {
        Ok(x) if x > 0 => Ok(x),
        _ => Err(String::from("Chunk size must be a number greater than 0.")),
    }
}

impl Opt {
    fn into_parts(self) -> (Vec<PathBuf>, settings::Settings) {
        (
            self.infile,
            settings::Settings::new(
                self.output,
                self.qr_version_min,
                self.qr_version_max,
                self.error_correction,
                self.chunk_size,
                self.has_headers,
                self.border,
                self.mask,
            ),
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
    let (files, settings) = opt.into_parts();
    debug!("Files {:?} | {}", files, settings);
    generator::from_files(&files, settings);
    info!("qrgen end");
}
