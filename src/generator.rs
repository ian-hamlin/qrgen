use crate::chunker;
use crate::formats;
use log::{trace, warn};
use qrcodegen;
use rayon::prelude::*;
use std::{error::Error, fmt, fs::File, path::PathBuf};

pub struct Generator {
    qr_opts: QrOpts,
    generator_opts: GeneratorOpts,
    files: Vec<PathBuf>,
}

impl Generator {
    pub fn new(files: Vec<PathBuf>, qr_opts: QrOpts, generator_opts: GeneratorOpts) -> Self {
        Generator {
            files,
            qr_opts,
            generator_opts,
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
        let chunks = chunker::Chunker::new(reader, self.generator_opts.chunk_size);

        for chunk in chunks {
            chunk
                .par_iter()
                .filter(|record| record.len() >= 2)
                .for_each(|record| {
                    let res = self.encode(record).and_then(Self::qr_code_write);
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
            .has_headers(self.generator_opts.has_headers)
            .trim(csv::Trim::All)
            .flexible(true)
            .from_reader(file))
    }

    fn encode(&self, record: &csv::StringRecord) -> Result<GeneratedOutput, Box<Error>> {
        let chars: Vec<char> = record[1].chars().collect();
        let segment = qrcodegen::QrSegment::make_segments(&chars);

        match qrcodegen::QrCode::encode_segments_advanced(
            &segment,
            self.qr_opts.error_correction,
            self.qr_opts.qr_version_min,
            self.qr_opts.qr_version_max,
            self.qr_opts.mask,
            true,
        ) {
            Ok(qr) => Ok(GeneratedOutput::new(
                PathBuf::from(&self.generator_opts.output),
                self.generator_opts.border,
                qr,
                self.generator_opts.format,
            )),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn qr_code_write(qr_gen: GeneratedOutput) -> Result<(), Box<Error>> {
        trace!("qr_code_write {}", qr_gen.output.display());
        Ok(())
    }
}

impl fmt::Display for Generator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Output:{}, QR Version Min:{}, QR Version Max:{}, Error Correction: {}, Chunk Size:{}, \
            Has CSV Header:{}, Border:{}, Mask:{}, Files: {:?}",
            self.generator_opts.output.display(),
            self.qr_opts.qr_version_min.value(),
            self.qr_opts.qr_version_max.value(),
            match self.qr_opts.error_correction {
                qrcodegen::QrCodeEcc::High => "High",
                qrcodegen::QrCodeEcc::Low => "Low",
                qrcodegen::QrCodeEcc::Quartile => "Quartile",
                qrcodegen::QrCodeEcc::Medium => "Medium",
            },
            self.generator_opts.chunk_size,
            self.generator_opts.has_headers,
            self.generator_opts.border,
            match self.qr_opts.mask {
                Some(m) => m.value().to_string(),
                _ => String::from("<Not Set>"),
            },
            self.files,
        )
    }
}

pub struct QrOpts {
    qr_version_min: qrcodegen::Version,
    qr_version_max: qrcodegen::Version,
    mask: Option<qrcodegen::Mask>,
    error_correction: qrcodegen::QrCodeEcc,
}

impl QrOpts {
    pub fn new(
        qr_version_min: qrcodegen::Version,
        qr_version_max: qrcodegen::Version,
        error_correction: qrcodegen::QrCodeEcc,
        mask: Option<qrcodegen::Mask>,
    ) -> Self {
        QrOpts {
            qr_version_min,
            qr_version_max,
            mask,
            error_correction,
        }
    }
}

pub struct GeneratorOpts {
    output: PathBuf,
    chunk_size: usize,
    has_headers: bool,
    border: u8,
    format: formats::Formats,
}

impl GeneratorOpts {
    pub fn new(
        output: PathBuf,
        chunk_size: usize,
        has_headers: bool,
        border: u8,
        format: formats::Formats,
    ) -> Self {
        GeneratorOpts {
            output,
            chunk_size,
            has_headers,
            border,
            format,
        }
    }
}

pub struct GeneratedOutput {
    output: PathBuf,
    border: u8,
    qr_code: qrcodegen::QrCode,
    format: formats::Formats,
}

impl GeneratedOutput {
    fn new(
        output: PathBuf,
        border: u8,
        qr_code: qrcodegen::QrCode,
        format: formats::Formats,
    ) -> Self {
        GeneratedOutput {
            output,
            border,
            qr_code,
            format,
        }
    }
}

// fn write_svg(
//     qr: qrcodegen::QrCode,
//     file_name: &str,
//     settings: &settings::Settings,
// ) -> Result<(), Box<Error>> {
//     let svg = qr.to_svg_string(i32::from(settings.border()));

//     let mut qr_file_path = PathBuf::from(&settings.output());
//     qr_file_path.push(file_name);
//     qr_file_path.set_extension("svg");

//     trace!("Writing svg file {}", qr_file_path.display());

//     let mut writer = OpenOptions::new()
//         .write(true)
//         .create(true)
//         .append(false)
//         .open(qr_file_path)?;
//     writer.write_all(svg.as_bytes())?;
//     writer.flush()?;

//     Ok(())
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn should_give_chunks() {
//         let input = "12\n34\n56\n78\n90".as_bytes();
//         let mut reader = csv::ReaderBuilder::new()
//             .has_headers(false)
//             .from_reader(input);

//         let chunk = next_chunk(2, &mut reader).unwrap();
//         assert_eq!(chunk.len(), 2);
//         assert_eq!(csv::StringRecord::from(vec!["12"]), chunk[0]);
//         assert_eq!(csv::StringRecord::from(vec!["34"]), chunk[1]);

//         let chunk = next_chunk(2, &mut reader).unwrap();
//         assert_eq!(chunk.len(), 2);
//         assert_eq!(csv::StringRecord::from(vec!["56"]), chunk[0]);
//         assert_eq!(csv::StringRecord::from(vec!["78"]), chunk[1]);

//         let chunk = next_chunk(2, &mut reader).unwrap();
//         assert_eq!(chunk.len(), 1);
//         assert_eq!(csv::StringRecord::from(vec!["90"]), chunk[0]);

//         assert_eq!(None, next_chunk(2, &mut reader));
//     }
// }
