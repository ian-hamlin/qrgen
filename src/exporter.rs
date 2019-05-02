use crate::generator;
use log::trace;
use std::{error::Error, fs::OpenOptions, io::prelude::*, path::PathBuf};

pub struct Exporter {
    qr_code: qrcodegen::QrCode,
    output: PathBuf,
    border: u8,
    format: generator::Formats,
    file_name: String,
}

impl Exporter {
    pub fn new(
        qr_code: qrcodegen::QrCode,
        output: PathBuf,
        border: u8,
        format: generator::Formats,
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
            generator::Formats::SVG => {
                self.output.set_extension("svg");
                trace!("Writing svg file {}", self.output.display());
            }
            generator::Formats::PNG => {
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
            generator::Formats::SVG => self.export_svg(writer, &self.qr_code, self.border),
            generator::Formats::PNG => self.export_png(writer, &self.qr_code, self.border),
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
        mut _writer: W,
        qr_code: &qrcodegen::QrCode,
        border: u8,
    ) -> Result<(), Box<Error>> {
        // ToDo - set a scale from the opts.
        let scale: i32 = 10;

        // Get the size of the code. This is the base size before adding the borders and scale.
        let size = qr_code.size().checked(scale, border);

        trace!("check size {:?}", size);

        // Add the border.
        //let size = size.checked_add(i32::from(border) * 2);

        // if size.is_none() {
        //     return Err(Box::new(String::from(
        //         "QR mask must be between 0 and 7 inclusive.",
        //     )));
        // }

        Ok(())
    }
}

trait CheckedSize {
    fn checked(self, scale: i32, border: u8) -> Option<i32>;
}

impl CheckedSize for i32 {
    fn checked(self, scale: i32, border: u8) -> Option<i32> {
        Some(
            self.checked_add(i32::from(border) * 2)?
                .checked_mul(scale)?,
        )
    }
}
