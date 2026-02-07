use printpdf::*;
use ttf_parser::Face;

use crate::config::Config;
use crate::qr;

/// Create a filled rectangle as a Polygon (DrawRectangle has a bug in printpdf 0.9.0
/// where it ignores PaintMode and always clips instead of filling).
fn filled_rect(x: Mm, y: Mm, w: Mm, h: Mm) -> Op {
    let p = |px: Mm, py: Mm| LinePoint {
        p: Point::new(px, py),
        bezier: false,
    };
    Op::DrawPolygon {
        polygon: Polygon {
            rings: vec![PolygonRing {
                points: vec![
                    p(x, y),
                    p(x + w, y),
                    p(x + w, y + h),
                    p(x, y + h),
                ],
            }],
            mode: PaintMode::Fill,
            winding_order: WindingOrder::NonZero,
        },
    }
}

fn rgb(c: [f32; 3]) -> Color {
    Color::Rgb(Rgb::new(c[0], c[1], c[2], None))
}

/// Measure text width in mm using actual font metrics via ttf-parser.
fn measure_text_width(text: &str, font_bytes: &[u8], font_size_pt: f32) -> f32 {
    let face = Face::parse(font_bytes, 0).expect("Failed to parse font for metrics");
    let units_per_em = face.units_per_em() as f32;
    let scale = font_size_pt / units_per_em; // pt per font unit

    let total_advance_pt: f32 = text
        .chars()
        .filter_map(|c| {
            let gid = face.glyph_index(c)?;
            let advance = face.glyph_hor_advance(gid)?;
            Some(advance as f32 * scale)
        })
        .sum();

    // Convert pt to mm (1pt = 0.3528mm)
    total_advance_pt * 0.3528
}

pub fn generate_table_pdf(
    config: &Config,
    table_number: u32,
    font: &ParsedFont,
    font_bytes: &[u8],
    bg_image: Option<&RawImage>,
) -> Vec<u8> {
    let mut doc = PdfDocument::new(&format!("Table {}", table_number));

    let page_width_mm = config.page_width_mm;
    let page_height_mm = config.page_height_mm;
    let page_w = Mm(page_width_mm);
    let page_h = Mm(page_height_mm);

    let font_id = doc.add_font(font);
    let font_handle = PdfFontHandle::External(font_id);

    let font_size_pt = config.table_number_font_size_pt;
    let font_size_mm = font_size_pt * 0.3528;

    let text = table_number.to_string();
    let text_width_mm = measure_text_width(&text, font_bytes, font_size_pt);
    let text_height_mm = font_size_mm;

    // Generate QR code
    let url = qr::make_url(&config.url_template, table_number);
    let qr_code = qr::generate_qr(&url);
    let qr_size = qr_code.size();
    let qr_total_mm = qr_size as f32 * config.qr_module_size_mm;

    // Vertical centering of the group (text + spacing + QR)
    let group_height = text_height_mm + config.spacing_mm + qr_total_mm;
    let vertical_offset_mm = page_height_mm * (config.vertical_offset_pct / 100.0);
    let group_center_y = page_height_mm / 2.0 - vertical_offset_mm;
    let group_top_y = group_center_y + group_height / 2.0;

    let mut ops: Vec<Op> = Vec::new();

    // 1. Background image (scaled to fill page) or solid color
    if let Some(img) = bg_image {
        let img_w = img.width as f32;
        let img_h = img.height as f32;
        // Compute DPI so image fills the configured page dimensions
        let dpi_x = img_w * 25.4 / page_width_mm;
        let scale_y = (img_h * 25.4 / page_height_mm) / dpi_x;
        let xobj = doc.add_image(img);
        ops.push(Op::UseXobject {
            id: xobj,
            transform: XObjectTransform {
                translate_x: Some(Mm(0.0).into()),
                translate_y: Some(Mm(0.0).into()),
                scale_x: Some(1.0),
                scale_y: Some(scale_y),
                dpi: Some(dpi_x),
                ..Default::default()
            },
        });
    } else {
        ops.push(Op::SetFillColor { col: rgb(config.background_color) });
        ops.push(filled_rect(Mm(0.0), Mm(0.0), page_w, page_h));
    }

    // 2. Table number text (centered horizontally)
    let text_x_mm = (page_width_mm - text_width_mm) / 2.0;
    let text_baseline_y_mm = group_top_y - text_height_mm;

    ops.push(Op::SetFillColor { col: rgb(config.table_number_color) });
    ops.push(Op::StartTextSection);
    ops.push(Op::SetFont {
        font: font_handle.clone(),
        size: Pt(font_size_pt),
    });
    ops.push(Op::SetTextCursor {
        pos: Point::new(Mm(text_x_mm), Mm(text_baseline_y_mm)),
    });
    ops.push(Op::ShowText {
        items: vec![TextItem::Text(text)],
    });
    ops.push(Op::EndTextSection);

    // 3. QR code as filled polygons
    let qr_left_x_mm = (page_width_mm - qr_total_mm) / 2.0;
    let qr_top_y_mm = group_top_y - text_height_mm - config.spacing_mm;

    ops.push(Op::SetFillColor { col: rgb(config.qr_color) });

    let module_mm = config.qr_module_size_mm;
    for qr_y in 0..qr_size {
        for qr_x in 0..qr_size {
            if qr_code.get_module(qr_x, qr_y) {
                let x_mm = qr_left_x_mm + (qr_x as f32 * module_mm);
                // PDF Y is bottom-up, QR Y is top-down
                let y_mm = qr_top_y_mm - ((qr_y + 1) as f32 * module_mm);
                ops.push(filled_rect(Mm(x_mm), Mm(y_mm), Mm(module_mm), Mm(module_mm)));
            }
        }
    }

    let page = PdfPage::new(page_w, page_h, ops);
    let mut warnings = Vec::new();

    let save_options = PdfSaveOptions {
        image_optimization: Some(ImageOptimizationOptions {
            format: Some(ImageCompression::Flate),
            max_image_size: None,
            quality: None,
            ..Default::default()
        }),
        ..Default::default()
    };

    doc.with_pages(vec![page])
        .save(&save_options, &mut warnings)
}
