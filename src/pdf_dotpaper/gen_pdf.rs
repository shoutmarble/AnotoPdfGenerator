use oxidize_pdf::{Color, Document, Page};

#[derive(Clone, Debug)]
pub struct PdfConfig {
    pub dpi: f32,
    pub color_up: String,
    pub color_down: String,
    pub color_left: String,
    pub color_right: String,
    pub dot_size: f32,
    pub offset_from_origin: f32,
    pub grid_spacing: f32,
}

impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            dpi: 600.0,
            color_up: "#649037".to_string(),
            color_down: "#FEA501".to_string(),
            color_left: "#4041FE".to_string(),
            color_right: "#FF00FF".to_string(),
            dot_size: 1.0,
            offset_from_origin: 3.0,
            grid_spacing: 10.0,
        }
    }
}

fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64 / 255.0;
        Color::rgb(r, g, b)
    } else {
        Color::Gray(0.0)
    }
}

#[derive(Clone, Copy)]
struct PdfColors {
    up: Color,
    down: Color,
    left: Color,
    right: Color,
}

#[derive(Clone, Copy)]
enum AnotoDot {
    Up,
    Down,
    Left,
    Right,
}

pub fn gen_pdf_from_matrix_data(
    bitmatrix: &ndarray::Array3<i32>,
    filename: &str,
    config: &PdfConfig,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::new();
    doc.set_title("Anoto PDF");
    doc.set_author("Rust");

    let mut page = oxidize_pdf::Page::a4();

    let height = bitmatrix.dim().0;
    let width = bitmatrix.dim().1;

    let page_width = page.width();
    let page_height = page.height();

    let grid_width = (width as f64 - 1.0) * config.grid_spacing as f64;
    let grid_height = (height as f64 - 1.0) * config.grid_spacing as f64;

    let margin_x = (page_width - grid_width) / 2.0;
    let margin_y = (page_height - grid_height) / 2.0;

    // Parse colors once (this used to be done per-dot).
    let colors = PdfColors {
        up: parse_hex_color(&config.color_up),
        down: parse_hex_color(&config.color_down),
        left: parse_hex_color(&config.color_left),
        right: parse_hex_color(&config.color_right),
    };

    for y in 0..height {
        for x in 0..width {
            let x_pos = margin_x + x as f64 * config.grid_spacing as f64;
            let y_pos = margin_y + y as f64 * config.grid_spacing as f64;
            let x_bit = bitmatrix[[y, x, 0]];
            let y_bit = bitmatrix[[y, x, 1]];
            let dot_type = x_bit + (y_bit << 1);
            let direction = match dot_type {
                0 => AnotoDot::Up,
                1 => AnotoDot::Left,
                2 => AnotoDot::Right,
                3 => AnotoDot::Down,
                _ => AnotoDot::Up,
            };
            draw_anoto_dot(&mut page, x_pos, y_pos, direction, config, &colors);
        }
    }

    doc.add_page(page);

    let output_dir = std::env::current_dir()?.join("output");
    std::fs::create_dir_all(&output_dir)?;
    let path = output_dir.join(filename);
    doc.save(path)?;
    Ok(())
}

fn draw_anoto_dot(
    page: &mut Page,
    x: f64,
    y: f64,
    direction: AnotoDot,
    config: &PdfConfig,
    colors: &PdfColors,
) {
    let radius = config.dot_size as f64;
    let offset = config.offset_from_origin as f64;

    match direction {
        AnotoDot::Up => {
            let y_up = y + offset;
            page.graphics()
                .set_fill_color(colors.up)
                .circle(x, y_up, radius)
                .fill();
        }
        AnotoDot::Down => {
            let y_down = y - offset;
            page.graphics()
                .set_fill_color(colors.down)
                .circle(x, y_down, radius)
                .fill();
        }
        AnotoDot::Left => {
            let x_left = x - offset;
            page.graphics()
                .set_fill_color(colors.left)
                .circle(x_left, y, radius)
                .fill();
        }
        AnotoDot::Right => {
            let x_right = x + offset;
            page.graphics()
                .set_fill_color(colors.right)
                .circle(x_right, y, radius)
                .fill();
        }
    }
}
