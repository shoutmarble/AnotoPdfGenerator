use plotters::prelude::*;
use std::error::Error;
use crate::pdf_dotpaper::gen_pdf::PdfConfig;

fn parse_hex_to_rgb(hex: &str) -> RGBColor {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        RGBColor(r, g, b)
    } else {
        BLACK
    }
}

pub fn draw_preview_image(
    bitmatrix: &ndarray::Array3<i8>,
    config: &PdfConfig,
    filename: &str
) -> Result<(), Box<dyn Error>> {
    // A4 dimensions in points (1/72 inch)
    let a4_width_pts = 595.276;
    let a4_height_pts = 841.89;
    
    // Scale factor from points to pixels
    let scale = config.dpi as f64 / 72.0;
    
    let img_width = (a4_width_pts * scale).ceil() as u32;
    let img_height = (a4_height_pts * scale).ceil() as u32;

    let root_area = BitMapBackend::new(filename, (img_width, img_height))
        .into_drawing_area();
    root_area.fill(&WHITE)?;

    // Use PDF coordinate system: 0..width, 0..height (points)
    // This assumes PDF origin is bottom-left.
    let mut chart = ChartBuilder::on(&root_area)
        .build_cartesian_2d(0f64..a4_width_pts, 0f64..a4_height_pts)?;

    let height = bitmatrix.dim().0;
    let width = bitmatrix.dim().1;
    let radius_px = (config.dot_size as f64 * scale).max(1.0) as u32;

    let grid_width = (width as f64 - 1.0) * config.grid_spacing as f64;
    let grid_height = (height as f64 - 1.0) * config.grid_spacing as f64;
    
    let margin_x = (a4_width_pts - grid_width) / 2.0;
    let margin_y = (a4_height_pts - grid_height) / 2.0;

    chart.draw_series(
        (0..height).flat_map(move |y| {
            (0..width).map(move |x| {
                let x_bit = bitmatrix[[y, x, 0]] as usize;
                let y_bit = bitmatrix[[y, x, 1]] as usize;
                let dot_type = x_bit + (y_bit << 1);
                
                let color = match dot_type {
                    0 => parse_hex_to_rgb(&config.color_up),
                    1 => parse_hex_to_rgb(&config.color_left),
                    2 => parse_hex_to_rgb(&config.color_right),
                    3 => parse_hex_to_rgb(&config.color_down),
                    _ => BLACK,
                };
                
                let x_pos = margin_x + x as f64 * config.grid_spacing as f64;
                let y_pos = margin_y + y as f64 * config.grid_spacing as f64;
                
                let (dx, dy) = match dot_type {
                    0 => (0.0, config.offset_from_origin as f64), // Up
                    1 => (-config.offset_from_origin as f64, 0.0), // Left
                    2 => (config.offset_from_origin as f64, 0.0), // Right
                    3 => (0.0, -config.offset_from_origin as f64), // Down
                    _ => (0.0, 0.0),
                };
                
                Circle::new((x_pos + dx, y_pos + dy), radius_px, color.filled())
            })
        })
    )?;

    Ok(())
}

pub fn draw_dot_on_file(
    filename: &str,
    x: f64,
    y: f64,
    matrix_height: usize,
    matrix_width: usize,
    config: &PdfConfig,
) -> Result<(), Box<dyn Error>> {
    // Load image
    let mut img = image::open(filename)?.to_rgba8();
    
    // A4 dimensions in points
    let a4_width_pts = 595.276;
    let a4_height_pts = 841.89;
    
    // Scale factor
    let scale = config.dpi as f64 / 72.0;
    
    let grid_width = (matrix_width as f64 - 1.0) * config.grid_spacing as f64;
    let grid_height = (matrix_height as f64 - 1.0) * config.grid_spacing as f64;
    
    let margin_x = (a4_width_pts - grid_width) / 2.0;
    let margin_y = (a4_height_pts - grid_height) / 2.0;
    
    let x_pos_pt = margin_x + x * config.grid_spacing as f64;
    let y_pos_pt = margin_y + y * config.grid_spacing as f64;
    
    // Convert to pixels
    // Plotters cartesian (0,0) is bottom-left. Pixel (0,0) is top-left.
    let x_px = (x_pos_pt * scale).round() as i32;
    let y_px = ((a4_height_pts - y_pos_pt) * scale).round() as i32;
    
    // Draw a red circle
    let radius = (config.dot_size as f64 * scale * 2.0).max(5.0) as i32; // Make it visible
    let color = image::Rgba([255, 0, 0, 255]);
    
    let (w, h) = img.dimensions();
    
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx*dx + dy*dy <= radius*radius {
                let px = x_px + dx;
                let py = y_px + dy;
                if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                    img.put_pixel(px as u32, py as u32, color);
                }
            }
        }
    }
    
    img.save(filename)?;
    Ok(())
}

pub fn draw_dots_on_file(
    filename: &str,
    points: &[(f64, f64)],
    matrix_height: usize,
    matrix_width: usize,
    config: &PdfConfig,
) -> Result<(), Box<dyn Error>> {
    // Load image
    let mut img = image::open(filename)?.to_rgba8();
    
    // A4 dimensions in points
    let a4_width_pts = 595.276;
    let a4_height_pts = 841.89;
    
    // Scale factor
    let scale = config.dpi as f64 / 72.0;
    
    let grid_width = (matrix_width as f64 - 1.0) * config.grid_spacing as f64;
    let grid_height = (matrix_height as f64 - 1.0) * config.grid_spacing as f64;
    
    let margin_x = (a4_width_pts - grid_width) / 2.0;
    let margin_y = (a4_height_pts - grid_height) / 2.0;
    
    let (w, h) = img.dimensions();
    let color = image::Rgba([255, 0, 0, 255]);
    let radius = (config.dot_size as f64 * scale * 2.0).max(5.0) as i32;

    for (x, y) in points {
        let x_pos_pt = margin_x + x * config.grid_spacing as f64;
        let y_pos_pt = margin_y + y * config.grid_spacing as f64;
        
        // Convert to pixels
        // Plotters cartesian (0,0) is bottom-left. Pixel (0,0) is top-left.
        let x_px = (x_pos_pt * scale).round() as i32;
        let y_px = ((a4_height_pts - y_pos_pt) * scale).round() as i32;
        
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx*dx + dy*dy <= radius*radius {
                    let px = x_px + dx;
                    let py = y_px + dy;
                    if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                        img.put_pixel(px as u32, py as u32, color);
                    }
                }
            }
        }
    }
    
    img.save(filename)?;
    Ok(())
}

// Drawing function using plotters
pub fn draw_dots(
    bitmatrix: &ndarray::Array3<i8>,
    _grid_size: f64,
    base_filename: &str) -> Result<(), Box<dyn Error>> {

    // Persist the bitmatrix
    // crate::persist_json::save_bitmatrix_text(bitmatrix, &format!("{}.txt", base_filename))?;
    // crate::persist_json::save_bitmatrix_json(bitmatrix, &format!("{}.json", base_filename))?;

    let filename = format!("output/{}__X.png", base_filename);
    draw_dots_y_axis(bitmatrix, _grid_size, &format!("output/{}__Y.png", base_filename))?;

    let root_area = BitMapBackend::new(&filename, (800, 400))
    .into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    let num_rows = bitmatrix.dim().0 as i32;
    let num_cols = bitmatrix.dim().1 as i32;

    let mut ctx = ChartBuilder::on(&root_area)
        .margin(15)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .caption("Anoto Dots", ("sans-serif", 40))
        .build_cartesian_2d(-10_i32..(num_cols * 10), -10_i32..(num_rows * 10))
        .unwrap();

   ctx.configure_mesh()
        .x_labels(num_cols as usize + 1)
        .x_label_formatter(&|v| format!("{}", (v / 10) ))
        .y_labels(num_rows as usize + 1)
        .y_label_formatter(&|v| format!("{}", (v / 10) ))
        .draw().unwrap();

    // Draw circles based on bitmatrix values
    // Draw circles based on bitmatrix values
    ctx.draw_series(
        (0..bitmatrix.dim().0).flat_map(|y| {
            (0..bitmatrix.dim().1).map(move |x| {
                let x_bit = bitmatrix[[y, x, 0]] as usize;
                let y_bit = bitmatrix[[y, x, 1]] as usize;
                let dot_type = x_bit + (y_bit << 1);
                let orange = RGBColor(255, 165, 0);
                let custom_green = RGBColor(100, 156, 54);
                let color = match dot_type {
                    0 => &custom_green, // UP
                    1 => &BLUE,    // LEFT
                    2 => &MAGENTA, // RIGHT
                    3 => &orange,  // DOWN
                    _ => &BLACK,
                };
                let mut x_x :i32 = x as i32;
                let mut y_y :i32 = bitmatrix.dim().0 as i32 - 1 - y as i32;
                match dot_type {
                    0 => { // UP
                        x_x *= 10;
                        y_y = y_y * 10 + 2;
                    }
                    1 => { // LEFT
                        x_x = (x_x * 10) - 2;
                        y_y *= 10;
                    },
                    2 => { // RIGHT
                        x_x = x_x * 10 + 2;
                        y_y *= 10;
                    }
                    3 => { // DOWN
                        x_x *= 10;
                        y_y = (y_y * 10) - 2;
                    },
                    _ => {}
                };

                Circle::new((x_x, y_y), 5, color.filled())
            })
        })
    ).unwrap();

    Ok(())
}


// Drawing function using plotters
pub fn draw_dots_y_axis(
    bitmatrix: &ndarray::Array3<i8>,
    _grid_size: f64,
    filename: &str) -> Result<(), Box<dyn Error>> {

    let root_area = BitMapBackend::new(filename, (800, 400))
    .into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    let num_rows = bitmatrix.dim().0 as i32;
    let num_cols = bitmatrix.dim().1 as i32;

    let mut ctx = ChartBuilder::on(&root_area)
        .margin(15)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .caption("Anoto Dots", ("sans-serif", 40))
        .build_cartesian_2d(-10_i32..(num_cols * 10), (num_rows * 10)..(-10_i32))
        .unwrap();

   ctx.configure_mesh()
        .x_labels(num_cols as usize + 1)
        .x_label_formatter(&|v| format!("{}", (v / 10) ))
        .y_labels(num_rows as usize + 1)
        .y_label_formatter(&|v| format!("{}", (v / 10) ))
        .draw().unwrap();

    // Draw circles based on bitmatrix values
    ctx.draw_series(
        (0..bitmatrix.dim().0).flat_map(|y| {
            (0..bitmatrix.dim().1).map(move |x| {
                let x_bit = bitmatrix[[y, x, 0]] as usize;
                let y_bit = bitmatrix[[y, x, 1]] as usize;
                let dot_type = x_bit + (y_bit << 1);
                let orange = RGBColor(255, 165, 0);
                let custom_green = RGBColor(100, 156, 54);
                let color = match dot_type {
                    0 => &custom_green, // UP
                    1 => &BLUE,    // LEFT
                    2 => &MAGENTA, // RIGHT
                    3 => &orange,  // DOWN
                    _ => &BLACK,
                };
                let mut x_x :i32 = x as i32;
                let mut y_y :i32 = (num_rows - 1) - y as i32;
                match dot_type {
                    0 => { // UP
                        x_x *= 10;
                        y_y = y_y * 10 + 2;
                    }
                    1 => { // LEFT
                        x_x = (x_x * 10) - 2;
                        y_y *= 10;
                    },
                    2 => { // RIGHT
                        x_x = x_x * 10 + 2;
                        y_y *= 10;
                    }
                    3 => { // DOWN
                        x_x *= 10;
                        y_y = (y_y * 10) - 2;
                    },
                    _ => {}
                };

                Circle::new((x_x, y_y), 5, color.filled())
            })
        })
    ).unwrap();

    Ok(())
}
