use log::warn;
use std::{fmt, path::PathBuf};

pub struct Settings {
    output: PathBuf,
    qr_version_min: qrcodegen::Version,
    qr_version_max: qrcodegen::Version,
    error_correction: qrcodegen::QrCodeEcc,
    chunk_size: usize,
    has_headers: bool,
    border: u8,
    mask: Option<qrcodegen::Mask>,
}

impl Settings {
    pub fn output(&self) -> &PathBuf {
        &self.output
    }

    pub fn border(&self) -> u8 {
        self.border
    }

    pub fn error_correction(&self) -> qrcodegen::QrCodeEcc {
        self.error_correction
    }

    pub fn qr_version_min(&self) -> qrcodegen::Version {
        self.qr_version_min
    }

    pub fn qr_version_max(&self) -> qrcodegen::Version {
        self.qr_version_max
    }

    pub fn mask(&self) -> Option<qrcodegen::Mask> {
        self.mask
    }

    pub fn has_headers(&self) -> bool {
        self.has_headers
    }

    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    pub fn new(
        output: PathBuf,
        qr_version_min: u8,
        qr_version_max: u8,
        error_correction: qrcodegen::QrCodeEcc,
        chunk_size: usize,
        has_headers: bool,
        border: u8,
        mask: Option<u8>,
    ) -> Self {
        let mut max_final = qr_version_max;
        if qr_version_min > qr_version_max {
            max_final = qr_version_min;
            warn!("QR Version Min is higher than QR Version Max so Max has been reset.")
        }

        Settings {
            output,
            qr_version_max: qrcodegen::Version::new(max_final),
            qr_version_min: qrcodegen::Version::new(qr_version_min),
            error_correction,
            chunk_size,
            has_headers,
            border,
            mask: match mask {
                Some(m) => Some(qrcodegen::Mask::new(m)),
                _ => None,
            },
        }
    }
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Output:{}, QR Version Min:{}, QR Version Max:{}, Error Correction: {}, Chunk Size:{}, \
            Has CSV Header:{}, Border:{}, Mask:{}",
            self.output.display(),
            self.qr_version_min.value(),
            self.qr_version_max.value(),
            match self.error_correction {
                qrcodegen::QrCodeEcc::High => "High",
                qrcodegen::QrCodeEcc::Low => "Low",
                qrcodegen::QrCodeEcc::Quartile => "Quartile",
                qrcodegen::QrCodeEcc::Medium => "Medium",
            },
            self.chunk_size,
            self.has_headers,
            self.border,
            match self.mask {
                Some(m) => m.value().to_string(),
                _ => String::from("<Not Set>"),
            },
        )
    }
}
