use crate::chunker;
use crate::exporter;
use log::{trace, warn};
use qrcodegen;
use rayon::prelude::*;
use std::{error::Error, fmt, fs::File, path::PathBuf};

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
                    if let Some(qr) = self.encode(record) {
                        let mut exp = exporter::Exporter::new(
                            qr,
                            self.out_conf.output.clone(),
                            self.out_conf.border,
                            self.out_conf.format,
                            record[0].to_string(),
                        );
                        let res = exp.export();
                        if res.is_err() {
                            warn!(
                                "error generating for {} {:?}",
                                record[0].to_string(),
                                res.err()
                            );
                        }
                    }
                });
        }

        Ok(())
    }

    fn csv_reader(&self, file_path: &PathBuf) -> Result<csv::Reader<File>, Box<Error>> {
        let file = File::open(file_path)?;

        Ok(csv::ReaderBuilder::new()
            .has_headers(self.proc_conf.has_headers)
            .trim(csv::Trim::All)
            .flexible(true)
            .from_reader(file))
    }

    fn encode(&self, record: &csv::StringRecord) -> Option<qrcodegen::QrCode> {
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
            Ok(qr) => Some(qr),
            Err(e) => {
                warn!("error generating for {} {:?}", record[0].to_string(), e);
                None
            }
        }
    }

    // fn save(qr_gen: QrOutput) -> Result<(), Box<Error>> {
    //     let writer = OpenOptions::new()
    //         .write(true)
    //         .create(true)
    //         .append(false)
    //         .open(qr_file_path)?;

    //     match qr_gen.out_conf.format {
    //         Formats::SVG => Self::write_svg(writer, qr_gen.qr_code, qr_gen.out_conf.border),
    //         Formats::PNG => Self::write_png(writer, qr_gen.qr_code, qr_gen.out_conf.border),
    //     }
    // }

    // fn write_png<W: Write>(
    //     writer: W,
    //     qr_code: qrcodegen::QrCode,
    //     border: u8,
    // ) -> Result<(), Box<Error>> {
    //     // ToDo - set a scale from the opts.
    //     let scale: u32 = 10;

    //     // Width and height adding the border and scale.

    //     // let x = (qr_code.size() as u32)
    //     //     .checked_mul(u32::from(border) * 2)?
    //     //     .checked_mul(scale);

    //     // let size = (qr_code.size() + (i32::from(border) * 2)) * scale;
    //     // let size = size as u32;

    //     // println!("{:?} x {:?}", size, size);

    //     // let mut encoder = png::Encoder::new(writer, size, size);
    //     // encoder.set(png::ColorType::RGB).set(png::BitDepth::Eight);

    //     // let mut writer = encoder.write_header()?;
    //     // let data = vec![0_u8; (size * size * 3) as usize];

    //     // match writer.write_image_data(&data) {
    //     //     Err(e) => println!("{:?}", e),
    //     //     _ => {}
    //     // }

    //     // for x in 0..width {
    //     //     for y in 0..height {
    //     //         encoder.
    //     //     }
    //     // }

    //     Ok(())
    // }
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

#[derive(Clone)]
pub struct OutputConfig {
    output: PathBuf,
    border: u8,
    format: exporter::ExportFormat,
}

impl OutputConfig {
    pub fn new(output: PathBuf, border: u8, format: exporter::ExportFormat) -> Self {
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
