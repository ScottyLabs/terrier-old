mod config;
mod pdf;
mod qr;

use std::path::PathBuf;

use printpdf::{ParsedFont, PdfWarnMsg, RawImage};

use config::load_config;
use pdf::generate_table_pdf;

fn main() {
    // Determine config path: --config <path> or default to config.toml next to the binary
    let config_path = std::env::args()
        .skip(1)
        .zip(std::env::args().skip(2))
        .find(|(flag, _)| flag == "--config")
        .map(|(_, path)| PathBuf::from(path))
        .unwrap_or_else(|| PathBuf::from("config.toml"));

    println!("Loading config from {}...", config_path.display());
    let config = load_config(&config_path);

    // Load font
    println!("Loading font from {}...", config.font_path);
    let font_bytes = std::fs::read(&config.font_path)
        .unwrap_or_else(|e| panic!("Failed to read font {}: {}", config.font_path, e));
    let mut warnings = Vec::new();
    let font = ParsedFont::from_bytes(&font_bytes, 0, &mut warnings)
        .expect("Failed to parse font file");
    if !warnings.is_empty() {
        eprintln!("Font warnings: {:?}", warnings);
    }

    // Load background image if configured
    let bg_image = config.background_image.as_ref().map(|path| {
        println!("Loading background image from {}...", path);
        let img_bytes = std::fs::read(path)
            .unwrap_or_else(|e| panic!("Failed to read background image {}: {}", path, e));
        let mut img_warnings: Vec<PdfWarnMsg> = Vec::new();
        RawImage::decode_from_bytes(&img_bytes, &mut img_warnings)
            .unwrap_or_else(|e| panic!("Failed to decode background image: {}", e))
    });

    // Create output directory
    let output_dir = PathBuf::from(&config.output_dir);
    std::fs::create_dir_all(&output_dir)
        .unwrap_or_else(|e| panic!("Failed to create output dir: {}", e));

    // Determine which tables to generate
    let tables: Vec<u32> = if config.debug {
        println!("Debug mode: generating only first and last table");
        if config.table_start == config.table_end {
            vec![config.table_start]
        } else {
            vec![config.table_start, config.table_end]
        }
    } else {
        (config.table_start..=config.table_end).collect()
    };

    let total = tables.len();
    println!("Generating {} table sign(s) ({}..={})...", total, config.table_start, config.table_end);

    for (i, &table_number) in tables.iter().enumerate() {
        let pdf_bytes = generate_table_pdf(&config, table_number, &font, &font_bytes, bg_image.as_ref());
        let filename = format!("table_{:03}.pdf", table_number);
        let path = output_dir.join(&filename);
        std::fs::write(&path, &pdf_bytes)
            .unwrap_or_else(|e| panic!("Failed to write {}: {}", path.display(), e));
        println!("  [{:>3}/{}] {}", i + 1, total, filename);
    }

    println!("Done! Generated {} PDF(s) in {}/", total, config.output_dir);
}
