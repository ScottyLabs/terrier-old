use qrcodegen::{QrCode, QrCodeEcc};

pub fn make_url(template: &str, table_number: u32) -> String {
    template.replace("{table_number}", &table_number.to_string())
}

pub fn generate_qr(url: &str) -> QrCode {
    QrCode::encode_text(url, QrCodeEcc::Medium)
        .expect("Failed to encode QR code")
}
