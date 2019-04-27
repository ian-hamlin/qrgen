use crate::settings;
use log::{info, trace, warn};
use qrcodegen;
use rayon::prelude::*;
use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::prelude::*,
    path::PathBuf,
};

pub fn from_files(files: &[PathBuf], settings: settings::Settings) {
    for file_path in files {
        match from_file(&file_path, &settings) {
            Ok(_) => trace!("complete file {}", file_path.display()),
            Err(e) => warn!("{:?}", e),
        }
    }
}

pub fn from_file(file_path: &PathBuf, settings: &settings::Settings) -> Result<(), Box<Error>> {
    info!("process file {}", file_path.display());

    // Open the file and reader.
    let file = File::open(file_path)?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(settings.has_headers())
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(file);

    // Get the file as chunks.
    while let Some(chunks) = next_chunk(settings.chunk_size(), &mut reader) {
        chunks.par_iter().for_each(|record| {
            if record.len() >= 2 {
                if let Some(qr) = generate_qr(&record[1], &settings) {
                    let res = write_svg(qr, &record[0], &settings);
                    if res.is_err() {
                        warn!(
                            "Error generating for {} {:?}",
                            record[0].to_string(),
                            res.err()
                        );
                    }
                }
            }
        });
    }

    Ok(())
}

fn write_svg(
    qr: qrcodegen::QrCode,
    file_name: &str,
    settings: &settings::Settings,
) -> Result<(), Box<Error>> {
    let svg = qr.to_svg_string(i32::from(settings.border()));

    let mut qr_file_path = PathBuf::from(&settings.output());
    qr_file_path.push(file_name);
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

fn next_chunk<T>(chunk_size: usize, reader: &mut csv::Reader<T>) -> Option<Vec<csv::StringRecord>>
where
    T: std::io::Read,
{
    let mut chunks = Vec::new();

    for (total, result) in reader.records().enumerate() {
        match result {
            Ok(r) => chunks.push(r),
            Err(e) => warn!("{:?}", e),
        }

        // Exit reading at this stage if we reached the chunk size.
        if total == chunk_size - 1 {
            break;
        }
    }

    if chunks.is_empty() {
        return None;
    }

    Some(chunks)
}

fn generate_qr(data: &str, settings: &settings::Settings) -> Option<qrcodegen::QrCode> {
    let chars: Vec<char> = data.chars().collect();
    let segment = qrcodegen::QrSegment::make_segments(&chars);

    match qrcodegen::QrCode::encode_segments_advanced(
        &segment,
        settings.error_correction(),
        settings.qr_version_min(),
        settings.qr_version_max(),
        settings.mask(),
        true,
    ) {
        Ok(qr) => Some(qr),
        Err(e) => {
            warn!("{:?}", e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_give_chunks() {
        let input = "12\n34\n56\n78\n90".as_bytes();
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(input);

        let chunk = next_chunk(2, &mut reader).unwrap();
        assert_eq!(chunk.len(), 2);
        assert_eq!(csv::StringRecord::from(vec!["12"]), chunk[0]);
        assert_eq!(csv::StringRecord::from(vec!["34"]), chunk[1]);

        let chunk = next_chunk(2, &mut reader).unwrap();
        assert_eq!(chunk.len(), 2);
        assert_eq!(csv::StringRecord::from(vec!["56"]), chunk[0]);
        assert_eq!(csv::StringRecord::from(vec!["78"]), chunk[1]);

        let chunk = next_chunk(2, &mut reader).unwrap();
        assert_eq!(chunk.len(), 1);
        assert_eq!(csv::StringRecord::from(vec!["90"]), chunk[0]);

        assert_eq!(None, next_chunk(2, &mut reader));
    }
}
