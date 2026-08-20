use qrcode::render::unicode::Dense1x2;
use qrcode::{EcLevel, QrCode};

#[derive(Clone)]
pub struct TerminalQrCode {
    code: QrCode,
}

impl TerminalQrCode {
    pub fn from_bytes<D: AsRef<[u8]>>(data: D) -> Result<TerminalQrCode, anyhow::Error> {
        let code = QrCode::with_error_correction_level(data, EcLevel::L)?;
        Ok(TerminalQrCode { code })
    }

    pub fn print(&self) {
        let image = self
            .code
            .render::<Dense1x2>()
            // Render a black code on a solid white background even when the
            // terminal itself uses a dark theme. The renderer also supplies
            // the QR-standard four-module quiet zone.
            .dark_color(Dense1x2::Light)
            .light_color(Dense1x2::Dark)
            .build();
        println!("{image}");
    }
}

#[cfg(test)]
mod tests {
    use super::TerminalQrCode;

    #[test]
    fn uses_the_smallest_fitting_qr_version() {
        let code = TerminalQrCode::from_bytes(b"https://example.com/login?token=short").unwrap();
        assert!(code.code.width() < 97);
    }
}
