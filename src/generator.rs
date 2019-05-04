use crate::chunker;
use crate::exporter;
use log::{trace, warn};
use qrcodegen;
use rayon::prelude::*;
use std::{error::Error, fmt, fs::File, io, path::PathBuf};

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
        for file_path in &self.files {
            match &self.process_file(file_path) {
                Ok(_) => trace!("complete file {}", file_path.display()),
                Err(e) => warn!("{:?}", e),
            }
        }
    }

    fn process_file(&self, file_path: &PathBuf) -> Result<(), Box<Error>> {
        trace!("process file {}", file_path.display());
        let file = File::open(file_path)?;
        let reader = self.csv_reader(file);
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
                            self.out_conf.scale,
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

    fn csv_reader<R: io::Read>(&self, reader: R) -> csv::Reader<R> {
        csv::ReaderBuilder::new()
            .has_headers(self.proc_conf.has_headers)
            .trim(csv::Trim::All)
            .flexible(true)
            .from_reader(reader)
    }

    fn encode(&self, record: &csv::StringRecord) -> Option<qrcodegen::QrCode> {
        let chars: Vec<char> = record[1].chars().collect();
        let segment = qrcodegen::QrSegment::make_segments(&chars);

        for s in segment.iter() {
            trace!(
                "encoding mode = {:?},  character count = {:?}",
                match s.mode() {
                    qrcodegen::QrSegmentMode::Alphanumeric => "Alphanumeric",
                    qrcodegen::QrSegmentMode::Byte => "Byte",
                    qrcodegen::QrSegmentMode::Eci => "Eci",
                    qrcodegen::QrSegmentMode::Kanji => "Kanji",
                    qrcodegen::QrSegmentMode::Numeric => "Numeric",
                },
                s.num_chars()
            );
        }

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

#[derive(Clone, Default)]
pub struct OutputConfig {
    output: PathBuf,
    border: u8,
    format: exporter::ExportFormat,
    scale: u8,
}

impl OutputConfig {
    pub fn new(output: PathBuf, border: u8, format: exporter::ExportFormat, scale: u8) -> Self {
        OutputConfig {
            output,
            border,
            format,
            scale,
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn default_generator() -> Generator {
        Generator::new(
            Vec::new(),
            QrConfig::new(
                qrcodegen::Version::new(1),
                qrcodegen::Version::new(2),
                qrcodegen::QrCodeEcc::High,
                None,
            ),
            Default::default(),
            Default::default(),
        )
    }

    #[test]
    fn ensure_csv_is_flexible_and_reads_header() {
        let gen = default_generator();
        let buff = Cursor::new("file_name,qr_data\nfile_name,qr_data,extra");

        let reader = gen.csv_reader(buff);
        let all_ok = reader.into_records().all(|r| r.is_ok());

        assert!(all_ok);
    }

    #[test]
    fn ensure_csv_skips_header() {
        let mut gen = default_generator();
        gen.proc_conf.has_headers = true;
        let buff = Cursor::new("file_name,qr_data\nfile_name,qr_data,extra");

        let reader = gen.csv_reader(buff);
        let count = reader.into_records().count();

        assert_eq!(1, count);
    }

    #[test]
    fn emsure_csv_trims() {
        let gen = default_generator();
        let buff = Cursor::new("  file_name, qr_data ");

        let mut reader = gen.csv_reader(buff);
        let record = reader.records().next().unwrap().unwrap();

        assert_eq!("file_name", record[0].to_string());
        assert_eq!("qr_data", record[1].to_string());
    }
}
