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
}

impl Exporter {
    pub fn new(
        qr_code: qrcodegen::QrCode,
        output: PathBuf,
        border: u8,
        format: ExportFormat,
        file_name: String,
    ) -> Self {
        Exporter {
            qr_code,
            output,
            border,
            format,
            file_name,
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
            ExportFormat::PNG => self.export_png(writer, &self.qr_code, self.border),
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
        writer.write_all(svg.as_bytes())?;
        Ok(())
    }

    fn export_png<W: Write>(
        &self,
        writer: W,
        qr_code: &qrcodegen::QrCode,
        border: u8,
    ) -> Result<(), Box<Error>> {
        // ToDo - set a scale from the opts.
        let scale: i32 = 20;

        // Get the size of the code. This is the base size before adding the borders and scale.
        let size = (qr_code.size() as u32).checked_size(scale, border);

        // Multiple by three as RGB has 3 values.
        let data_length = size.checked_length(3);

        if size.is_some() && data_length.is_some() {
            // Both are some, so this is OK.
            let size = size.unwrap();
            let data_length = data_length.unwrap();

            let mut encoder = png::Encoder::new(writer, size, size);
            encoder.set(png::ColorType::RGB).set(png::BitDepth::Eight);

            let mut writer = encoder.write_header()?;
            let mut data = vec![255_u8; data_length as usize];
            let mut offset = 0_usize;
            let border = i32::from(border);

            for x in 0..size as i32 {
                for y in 0..size as i32 {
                    if qr_code.get_module(x / scale - border, y / scale - border) {
                        data[offset] = 0;
                        data[offset + 1] = 0;
                        data[offset + 2] = 0;
                    }
                    offset += 3;
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
    fn checked_length(self, colour_depth: u32) -> Option<u32>;
}

impl CheckdLength for Option<u32> {
    fn checked_length(self, colour_depth: u32) -> Option<u32> {
        Some(self?.checked_mul(self?)?.checked_mul(colour_depth)?)
    }
}
