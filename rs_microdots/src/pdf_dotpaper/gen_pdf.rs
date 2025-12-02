use oxidize_pdf::{Document, Page, Color, Result};
use ndarray::Array3;



#[derive(Clone, Copy)]
enum AnotoDot {
    Up,
    Down,
    Left,
    Right,
}

pub fn gen_anoto_pdf_from_generated(height: usize, width: usize, sect_u: i32, sect_v: i32) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let codec = crate::anoto_matrix::anoto_6x6_a4_fixed();
    let bitmatrix = codec.encode_bitmatrix((height, width), (sect_u, sect_v));
    let filename = format!("G__{}__{}__{}__{}.pdf", height, width, sect_u, sect_v);
    gen_pdf_from_matrix_data(&bitmatrix, &filename)
}

pub fn gen_anoto_pdf_from_json(json_path: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let bitmatrix_2d = crate::persist_json::load_from_json(json_path)?;
    // Convert to 3D
    let (height, width) = bitmatrix_2d.dim();
    let mut bitmatrix = Array3::<i32>::zeros((height, width, 2));
    for i in 0..height {
        for j in 0..width {
            let bit = bitmatrix_2d[[i, j]];
            bitmatrix[[i, j, 0]] = bit;
            bitmatrix[[i, j, 1]] = bit;
        }
    }
    let filename = format!("{}.pdf", json_path.trim_end_matches(".json"));
    gen_pdf_from_matrix_data(&bitmatrix, &filename)
}

#[allow(dead_code)]
fn convert_bitmatrix(bitmatrix: Array3<i32>) -> Vec<Vec<i32>> {
    let mut data = Vec::new();
    for row in bitmatrix.outer_iter() {
        let mut row_data = Vec::new();
        for col in row.outer_iter() {
            let x_bit = col[0];
            let y_bit = col[1];
            let dot_type = x_bit + (y_bit << 1);
            row_data.push(dot_type);
        }
        data.push(row_data);
    }
    data
}

pub fn gen_pdf_from_matrix_data(bitmatrix: &ndarray::Array3<i32>, filename: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::new();
    doc.set_title("Anoto PDF");
    doc.set_author("Rust");

    let mut page = oxidize_pdf::Page::a4();

    let height = bitmatrix.dim().0;
    let width = bitmatrix.dim().1;

    for y in 0..height {
        for x in 0..width {
            let x_pos = x as f64 * 10.0;
            let y_pos = y as f64 * 10.0;
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
            draw_anoto_dot(&mut page, x_pos, y_pos, direction);
        }
    }

    doc.add_page(page);
    let path = std::env::current_dir().unwrap().join("output").join(filename);
    doc.save(path)?;
    Ok(())
}

#[allow(dead_code)]
pub fn gen_all_dots_anoto_pdf() -> Result<()> {
    // Create a new document
    let mut doc = Document::new();
    doc.set_title("My First PDF");
    doc.set_author("Rust Developer");
    
    // Create a page
    let mut page = Page::a4();
    
    let page_width = page.width();
    let page_height = page.height();
    for x in (0..page_width as u32).step_by(10) {

        for y in (0..page_height as u32).step_by(10) {

            draw_anoto_dot(&mut page, x as f64, y as f64, AnotoDot::Up);
            draw_anoto_dot(&mut page, x as f64, y as f64, AnotoDot::Down);
            draw_anoto_dot(&mut page, x as f64, y as f64, AnotoDot::Left);
            draw_anoto_dot(&mut page, x as f64, y as f64, AnotoDot::Right);

            // draw_grid_lines(&mut page, 10.0);


        }
    }

    println!("page width={} height={}", page_width as i32, page_height);
    println!("number anoto  dotts width={} height={}", (page_width/10.0) as i32, (page_height/10.0) as i32);
   
    // Add the page and save
    doc.add_page(page);
    let path = std::env::current_dir().unwrap().join("output").join("anoto.pdf");
    doc.save(path)?;
    
    Ok(())
}

#[allow(dead_code)]
fn draw_grid_lines(page: &mut Page, spacing: f64) {
    let page_width = page.width();
    let page_height = page.height();

    // Draw horizontal lines
    for y in (0..page_height as u32).step_by(spacing as usize) {
        page.graphics()
            .set_opacity(1.0)
            .set_stroke_color(Color::Gray(0.5))
            .set_line_width(0.5)
            .move_to(0.0, y as f64)
            .line_to(page_width, y as f64)
            .stroke();
    }

    // Draw vertical lines
    for x in (0..page_width as u32).step_by(spacing as usize) {
        page.graphics()
            .set_opacity(1.0)
            .set_stroke_color(Color::Gray(0.5))
            .set_line_width(0.5)
            .move_to(x as f64, 0.0)
            .line_to(x as f64, page_height)
            .stroke();
    }
}

#[allow(dead_code)]
fn draw_anoto_dot(page: &mut Page, x: f64, y: f64, direction: AnotoDot) {

    let radius = 1.0;

    match direction {
        AnotoDot::Up => {
            let y_up = y + 3.0;
            page.graphics()
                .set_fill_color(Color::rgb(100.0 / 255.0, 156.0 / 255.0, 54.0 / 255.0))
                .circle(x, y_up, radius)
                .fill();
        },
        AnotoDot::Down => {
            let y_down = y - 3.0;
            page.graphics()
                .set_fill_color(Color::rgb(1.0, 0.647, 0.0))
                .circle(x, y_down, radius)
                .fill();
        },
        AnotoDot::Left => {
            let x_left = x - 3.0;
            page.graphics()
                .set_fill_color(Color::blue())
                .circle(x_left, y, radius)
                .fill();
        },
        AnotoDot::Right => {
            let x_right = x + 3.0;
            page.graphics()
                .set_fill_color(Color::magenta())
                .circle(x_right, y, radius)
                .fill();
        },

    }

}
