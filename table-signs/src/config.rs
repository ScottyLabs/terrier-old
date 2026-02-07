use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_table_start")]
    pub table_start: u32,
    #[serde(default = "default_table_end")]
    pub table_end: u32,
    #[serde(default = "default_url_template")]
    pub url_template: String,
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    #[serde(default = "default_page_width")]
    pub page_width_mm: f32,
    #[serde(default = "default_page_height")]
    pub page_height_mm: f32,
    #[serde(default = "default_bg_color")]
    pub background_color: [f32; 3],
    pub background_image: Option<String>,
    #[serde(default = "default_font_path")]
    pub font_path: String,
    #[serde(default = "default_font_size")]
    pub table_number_font_size_pt: f32,
    #[serde(default = "default_dark_color")]
    pub table_number_color: [f32; 3],
    #[serde(default = "default_qr_module_size")]
    pub qr_module_size_mm: f32,
    #[serde(default = "default_black")]
    pub qr_color: [f32; 3],
    #[serde(default = "default_spacing")]
    pub spacing_mm: f32,
    #[serde(default)]
    pub vertical_offset_pct: f32,
    #[serde(default)]
    pub debug: bool,
}

fn default_table_start() -> u32 { 0 }
fn default_table_end() -> u32 { 150 }
fn default_url_template() -> String {
    "https://terrier.scottylabs.org/h/tartanhacks-2026/table-checkin/{table_number}".to_string()
}
fn default_output_dir() -> String { "output".to_string() }
fn default_page_width() -> f32 { 279.4 }
fn default_page_height() -> f32 { 215.9 }
fn default_bg_color() -> [f32; 3] { [1.0, 1.0, 1.0] }
fn default_font_path() -> String { "fonts/PPRader-Regular.otf".to_string() }
fn default_font_size() -> f32 { 144.0 }
// #272727 = rgb(39, 39, 39) = [0.153, 0.153, 0.153]
fn default_dark_color() -> [f32; 3] { [0.153, 0.153, 0.153] }
fn default_black() -> [f32; 3] { [0.0, 0.0, 0.0] }
fn default_qr_module_size() -> f32 { 1.0 }
fn default_spacing() -> f32 { 15.0 }

pub fn load_config(config_path: &Path) -> Config {
    let content = std::fs::read_to_string(config_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", config_path.display(), e));
    toml::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", config_path.display(), e))
}
