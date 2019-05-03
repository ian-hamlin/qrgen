use itertools::Itertools;
use log::trace;
use png::HasParameters;
use std::convert::TryFrom;
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
        // Make everything a bit simpler to work with.
        let scale: i32 = i32::from(scale);
        let border: i32 = i32::from(border);

        // Set the colour type and get the samples per pixel.
        let colour_type = png::ColorType::RGB;
        let colour_type_samples = colour_type.samples();

        // Get the size of the code.
        let size = Some(qr_code.size()).checked_size(scale, border);

        // Multiple by the colour sample length.
        let data_length = size.checked_length(colour_type_samples);

        if size.is_some() && data_length.is_some() {
            // Both are some, so this is OK.
            let size = size.unwrap();
            let data_length = data_length.unwrap();

            let mut encoder = png::Encoder::new(writer, size as u32, size as u32);
            encoder.set(colour_type).set(png::BitDepth::Eight);

            let mut writer = encoder.write_header()?;
            let mut data = vec![255_u8; data_length as usize];

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

            let offset_fn = |x: i32, y: i32, s: i32, cts: usize| {
                (x as usize * cts) + (y as usize * (s as usize * cts))
            };

            // this does not combine with itself so zip with (size,size).
            let points = (0..size)
                .tuple_combinations::<(_, _)>()
                .chain((0..size).zip(0..size));

            for point in points {
                // TODO - I can probably make this into a macro?
                let y = point.0;
                let x = point.1;
                let offset = offset_fn(x, y, size, colour_type_samples);

                if qr_code.get_module(x / scale - border, y / scale - border) {
                    // ToDo - this needs to change based on the colour sample level.
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
        } else {
            Err("size or data length are out of bounds.")?
        }

        Ok(())
    }
}

trait Checked {
    fn checked_size(self, scale: i32, border: i32) -> Option<i32>;
    fn checked_length(self, colour_depth: usize) -> Option<i32>;
}

impl Checked for Option<i32> {
    fn checked_size(self, scale: i32, border: i32) -> Option<i32> {
        if let Some(b) = border.checked_mul(2) {
            Some(self?.checked_add(b)?.checked_mul(scale)?)
        } else {
            None
        }
    }

    fn checked_length(self, colour_depth: usize) -> Option<i32> {
        if let Ok(cd) = i32::try_from(colour_depth) {
            Some(self?.checked_mul(self?)?.checked_mul(cd)?)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_length_should_return_none_for_large_colour_depth() {
        let s = Some(1_i32);
        let res = s.checked_length(usize::max_value());

        assert_eq!(None, res);
    }

    #[test]
    fn checked_length_should_return_none_for_large_self() {
        let s = Some(i32::max_value());
        let res = s.checked_length(2_usize);

        assert_eq!(None, res);
    }

    #[test]
    fn checked_length_should_return_none() {
        let s = Some(22);
        let res = s.checked_length(i32::max_value() as usize);

        assert_eq!(None, res);
    }

    #[test]
    fn checked_size_should_return_none_for_large_border() {
        let s = Some(1_i32);
        let res = s.checked_size(1_i32, i32::max_value());

        assert_eq!(None, res);
    }

    #[test]
    fn checked_size_should_return_none_for_large_add() {
        let s = Some(i32::max_value());
        let res = s.checked_size(1_i32, i32::max_value() - 1);

        assert_eq!(None, res);
    }

    #[test]
    fn checked_size_should_return_none_for_large_scale() {
        let s = Some(2);
        let res = s.checked_size(i32::max_value(), 2);

        assert_eq!(None, res);
    }
}
