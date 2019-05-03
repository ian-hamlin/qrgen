use itertools::Itertools;
use log::trace;
use png::HasParameters;
use std::{error::Error, fs::OpenOptions, io::prelude::*, path::PathBuf};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ExportFormat {
    SVG,
    PNG,
}

pub struct Exporter {
    qr_code: qrcodegen::QrCode,
    output: PathBuf,
    border: u8,
    format: ExportFormat,
    file_name: String,
    scale: u8,
}

impl Exporter {
    pub fn new(
        qr_code: qrcodegen::QrCode,
        output: PathBuf,
        border: u8,
        format: ExportFormat,
        file_name: String,
        scale: u8,
    ) -> Self {
        Exporter {
            qr_code,
            output,
            border,
            format,
            file_name,
            scale,
        }
    }

    pub fn export(&mut self) -> Result<(), Box<Error>> {
        self.output.push(&self.file_name);

        match self.format {
            ExportFormat::SVG => {
                self.output.set_extension("svg");
                trace!("Writing svg file {}", self.output.display());
            }
            ExportFormat::PNG => {
                self.output.set_extension("png");
                trace!("Writing png file {}", self.output.display());
            }
        }

        let writer = OpenOptions::new()
            .write(true)
            .create(true)
            .append(false)
            .open(&self.output)?;

        match self.format {
            ExportFormat::SVG => self.export_svg(writer, &self.qr_code, self.border),
            ExportFormat::PNG => self.export_png(writer, &self.qr_code, self.border, self.scale),
        }?;

        Ok(())
    }

    fn export_svg<W: Write>(
        &self,
        mut writer: W,
        qr_code: &qrcodegen::QrCode,
        border: u8,
    ) -> Result<(), Box<Error>> {
        let svg = qr_code.to_svg_string(i32::from(border));

        trace!(
            "version = {:?}, errorcorrectionlevel = {:?}, mask = {:?}",
            qr_code.version().value(),
            match qr_code.error_correction_level() {
                qrcodegen::QrCodeEcc::High => "High",
                qrcodegen::QrCodeEcc::Low => "Low",
                qrcodegen::QrCodeEcc::Quartile => "Quartile",
                qrcodegen::QrCodeEcc::Medium => "Medium",
            },
            qr_code.mask().value(),
        );

        writer.write_all(svg.as_bytes())?;
        Ok(())
    }

    fn export_png<W: Write>(
        &self,
        writer: W,
        qr_code: &qrcodegen::QrCode,
        border: u8,
        scale: u8,
    ) -> Result<(), Box<Error>> {
        let scale: i32 = i32::from(scale);
        let colour_type = png::ColorType::RGB;
        let colour_type_samples = colour_type.samples();

        // Get the size of the code.
        let size = (qr_code.size() as u32).checked_size(scale, border);

        // Multiple by three as RGB has 3 values to get the data length for the PNG library.
        let data_length = size.checked_length(colour_type_samples);

        if size.is_some() && data_length.is_some() {
            // Both are some, so this is OK.
            let size = size.unwrap();
            let data_length = data_length.unwrap();

            let mut encoder = png::Encoder::new(writer, size, size);
            encoder.set(colour_type).set(png::BitDepth::Eight);

            let mut writer = encoder.write_header()?;
            let mut data = vec![255_u8; data_length as usize];
            let border = i32::from(border);

            trace!(
                "version = {:?}, errorcorrectionlevel = {:?}, mask = {:?}, size = {}, data length = {}",
                qr_code.version().value(),
                match qr_code.error_correction_level() {
                    qrcodegen::QrCodeEcc::High => "High",
                    qrcodegen::QrCodeEcc::Low => "Low",
                    qrcodegen::QrCodeEcc::Quartile => "Quartile",
                    qrcodegen::QrCodeEcc::Medium => "Medium",
                },
                qr_code.mask().value(),
                size,
                data_length,
            );

            let offset_fn = |x: i32, y: i32, s: u32, cts: usize| {
                (x as usize * cts) + (y as usize * (s as usize * cts))
            };

            // this does not combine with itself so zip with (size,size).
            let points = (0..size as i32)
                .tuple_combinations::<(i32, i32)>()
                .chain((0..size as i32).zip(0..size as i32));
            for point in points {
                let y = point.0;
                let x = point.1;
                let offset = offset_fn(x, y, size, colour_type_samples);

                if qr_code.get_module(x / scale - border, y / scale - border) {
                    data[offset] = 0;
                    data[offset + 1] = 0;
                    data[offset + 2] = 0;
                }

                let y = point.1;
                let x = point.0;
                let offset = offset_fn(x, y, size, colour_type_samples);

                if qr_code.get_module(x / scale - border, y / scale - border) {
                    data[offset] = 0;
                    data[offset + 1] = 0;
                    data[offset + 2] = 0;
                }
            }

            writer.write_image_data(&data)?
        }

        Ok(())
    }
}

trait CheckedSize {
    fn checked_size(self, scale: i32, border: u8) -> Option<u32>;
}

impl CheckedSize for u32 {
    fn checked_size(self, scale: i32, border: u8) -> Option<u32> {
        Some(
            self.checked_add(u32::from(border) * 2)?
                .checked_mul(scale as u32)?,
        )
    }
}

trait CheckdLength {
    fn checked_length(self, colour_depth: usize) -> Option<u32>;
}

impl CheckdLength for Option<u32> {
    fn checked_length(self, colour_depth: usize) -> Option<u32> {
        Some(self?.checked_mul(self?)?.checked_mul(colour_depth as u32)?)
    }
}
