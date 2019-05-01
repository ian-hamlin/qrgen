use crate::chunker;
use log::{trace, warn};
use qrcodegen;
use rayon::prelude::*;
use std::{
    error::Error,
    fmt,
    fs::{File, OpenOptions},
    io::prelude::*,
    path::PathBuf,
};

#[derive(Copy, Clone, Debug)]
pub enum Formats {
    SVG,
}

pub struct Generator {
    qr_conf: QrConfig,
    out_conf: OutputConfig,
    proc_conf: ProcessingConfig,
    files: Vec<PathBuf>,
}

impl Generator {
    pub fn new(
        files: Vec<PathBuf>,
        qr_conf: QrConfig,
        out_conf: OutputConfig,
        proc_conf: ProcessingConfig,
    ) -> Self {
        Generator {
            files,
            qr_conf,
            out_conf,
            proc_conf,
        }
    }

    pub fn generate(&self) {
        trace!("generate");
        for file_path in &self.files {
            match &self.process_file(file_path) {
                Ok(_) => trace!("complete file {}", file_path.display()),
                Err(e) => warn!("{:?}", e),
            }
        }
    }

    fn process_file(&self, file_path: &PathBuf) -> Result<(), Box<Error>> {
        trace!("process file {}", file_path.display());
        let reader = self.csv_reader(file_path)?;
        let chunks = chunker::Chunker::new(reader, self.proc_conf.chunk_size);

        for chunk in chunks {
            chunk
                .par_iter()
                .filter(|record| record.len() >= 2)
                .for_each(|record| {
                    let res = self.encode(record).and_then(Self::save);
                    if res.is_err() {
                        warn!(
                            "error generating for {} {:?}",
                            record[0].to_string(),
                            res.err()
                        );
                    }
                });
        }

        Ok(())
    }

    fn csv_reader(&self, file_path: &PathBuf) -> Result<csv::Reader<File>, Box<Error>> {
        trace!("csv_reader {}", file_path.display());

        let file = File::open(file_path)?;

        Ok(csv::ReaderBuilder::new()
            .has_headers(self.proc_conf.has_headers)
            .trim(csv::Trim::All)
            .flexible(true)
            .from_reader(file))
    }

    fn encode(&self, record: &csv::StringRecord) -> Result<GeneratedOutput, Box<Error>> {
        let chars: Vec<char> = record[1].chars().collect();
        let segment = qrcodegen::QrSegment::make_segments(&chars);

        match qrcodegen::QrCode::encode_segments_advanced(
            &segment,
            self.qr_conf.error_correction,
            self.qr_conf.qr_version_min,
            self.qr_conf.qr_version_max,
            self.qr_conf.mask,
            true,
        ) {
            Ok(qr) => Ok(GeneratedOutput::new(
                PathBuf::from(&self.out_conf.output),
                self.out_conf.border,
                qr,
                self.out_conf.format,
                record[0].to_string(),
            )),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn save(qr_gen: GeneratedOutput) -> Result<(), Box<Error>> {
        let svg = qr_gen.qr_code.to_svg_string(i32::from(qr_gen.border));

        let mut qr_file_path = qr_gen.output;
        qr_file_path.push(qr_gen.file_name);
        qr_file_path.set_extension("svg");

        trace!("Writing svg file {}", qr_file_path.display());

        let mut writer = OpenOptions::new()
            .write(true)
            .create(true)
            .append(false)
            .open(qr_file_path)?;
        writer.write_all(svg.as_bytes())?;
        writer.flush()?;

        Ok(())
    }
}

impl fmt::Display for Generator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "qr_conf = [QR Version Min:{}, QR Version Max:{}, Error Correction: {}, Mask:{}], \
             proc_conf = [Chunk Size:{}, Has CSV Header:{}], \
             out_conf: [Border:{}, Format: {:?}, Output: {}], \
             input: Files: {:?}:",
            self.qr_conf.qr_version_min.value(),
            self.qr_conf.qr_version_max.value(),
            match self.qr_conf.error_correction {
                qrcodegen::QrCodeEcc::High => "High",
                qrcodegen::QrCodeEcc::Low => "Low",
                qrcodegen::QrCodeEcc::Quartile => "Quartile",
                qrcodegen::QrCodeEcc::Medium => "Medium",
            },
            match self.qr_conf.mask {
                Some(m) => m.value().to_string(),
                _ => String::from("<Not Set>"),
            },
            self.proc_conf.chunk_size,
            self.proc_conf.has_headers,
            self.out_conf.border,
            self.out_conf.format,
            self.out_conf.output.display(),
            self.files,
        )
    }
}

pub struct QrConfig {
    qr_version_min: qrcodegen::Version,
    qr_version_max: qrcodegen::Version,
    mask: Option<qrcodegen::Mask>,
    error_correction: qrcodegen::QrCodeEcc,
}

impl QrConfig {
    pub fn new(
        qr_version_min: qrcodegen::Version,
        qr_version_max: qrcodegen::Version,
        error_correction: qrcodegen::QrCodeEcc,
        mask: Option<qrcodegen::Mask>,
    ) -> Self {
        QrConfig {
            qr_version_min,
            qr_version_max,
            mask,
            error_correction,
        }
    }
}

pub struct OutputConfig {
    output: PathBuf,
    border: u8,
    format: Formats,
}

impl OutputConfig {
    pub fn new(output: PathBuf, border: u8, format: Formats) -> Self {
        OutputConfig {
            output,
            border,
            format,
        }
    }
}

pub struct ProcessingConfig {
    chunk_size: usize,
    has_headers: bool,
}

impl ProcessingConfig {
    pub fn new(chunk_size: usize, has_headers: bool) -> Self {
        ProcessingConfig {
            chunk_size,
            has_headers,
        }
    }
}

pub struct GeneratedOutput {
    output: PathBuf,
    border: u8,
    qr_code: qrcodegen::QrCode,
    _format: Formats,
    file_name: String,
}

impl GeneratedOutput {
    fn new(
        output: PathBuf,
        border: u8,
        qr_code: qrcodegen::QrCode,
        format: Formats,
        file_name: String,
    ) -> Self {
        GeneratedOutput {
            output,
            border,
            qr_code,
            _format: format,
            file_name,
        }
    }
}
