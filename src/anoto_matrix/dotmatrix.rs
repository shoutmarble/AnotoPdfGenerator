use ndarray::{Array3, s};
use std::error::Error;
use std::fmt;

// Custom error type for decoding errors
#[derive(Debug)]
pub struct DecodingError {
    message: String,
}

impl fmt::Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Decoding error: {}", self.message)
    }
}

impl Error for DecodingError {}

// Anoto codec structure
#[derive(Debug, Clone)]
pub struct AnotoCodec {
    pub mns: Vec<i8>,
    pub mns_length: usize,
    pub a_sequences: Vec<Vec<i8>>,
    pub a_orders: Vec<i32>,
    pub page_size: (i32, i32),
    pub sns_lengths: Vec<usize>,
    pub sns_cyclic: Vec<Vec<i8>>,
    pub bases: Vec<i32>,
    pub pfactors: Vec<i32>,
    pub delta_range: (i32, i32),
}

impl AnotoCodec {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mns: Vec<i8>,
        mns_length: usize,
        a_sequences: Vec<Vec<i8>>,
        a_orders: Vec<i32>,
        page_size: (i32, i32),
        sns_lengths: Vec<usize>,
        sns_cyclic: Vec<Vec<i8>>,
        bases: Vec<i32>,
        pfactors: Vec<i32>,
        delta_range: (i32, i32),
    ) -> Self {
        AnotoCodec {
            mns,
            mns_length,
            a_sequences,
            a_orders,
            page_size,
            sns_lengths,
            sns_cyclic,
            bases,
            pfactors,
            delta_range,
        }
    }

    fn reconstruct(&self, coeffs: &[i8]) -> i32 {
        coeffs
            .iter()
            .enumerate()
            .map(|(i, &c)| c as i32 * self.bases[i])
            .sum()
    }

    fn _delta(&self, pos: i32) -> i32 {
        let rs: Vec<i32> = self
            .sns_lengths
            .iter()
            .map(|&len| pos % len as i32)
            .collect();
        let coeffs: Vec<i8> = rs
            .iter()
            .enumerate()
            .map(|(i, &r)| self.sns_cyclic[i][r as usize])
            .collect();
        self.reconstruct(&coeffs) + self.delta_range.0
    }

    fn _next_roll(&self, pos: i32, prev_roll: i32) -> i32 {
        if pos == 0 {
            prev_roll
        } else {
            (prev_roll + self._delta(pos - 1)) % self.mns_length as i32
        }
    }

    fn roll_mns(&self, roll: i32) -> Vec<i8> {
        let shift = roll;
        let len = self.mns_length;
        let abs_shift = shift.unsigned_abs() as usize % len;
        let mut s = self.mns.clone();
        if shift >= 0 {
            s.rotate_left((len - abs_shift) % len);
        } else {
            s.rotate_left(abs_shift);
        }
        s
    }

    pub fn encode_bitmatrix(&self, shape: (usize, usize), section: (i32, i32)) -> Array3<i32> {
        let (h, w) = shape;
        let (sect_u, sect_v) = section;
        let mns_len = self.mns_length as i32;
        let mshape = (
            ((h as f64 / mns_len as f64).ceil() as usize * mns_len as usize),
            ((w as f64 / mns_len as f64).ceil() as usize * mns_len as usize),
        );
        let mut m = Array3::<i32>::zeros((mshape.0, mshape.1, 2));
        let ytiles = mshape.0 / mns_len as usize;
        let xtiles = mshape.1 / mns_len as usize;

        // x-direction
        let mut roll = sect_u % mns_len;
        for x in 0..mshape.1 {
            roll = self._next_roll(x as i32, roll);
            let s = self.roll_mns(-roll);
            for ty in 0..ytiles {
                for i in 0..mns_len as usize {
                    m[(ty * mns_len as usize + i, x, 0)] = s[i] as i32;
                }
            }
        }

        // y-direction
        roll = sect_v % mns_len;
        for y in 0..mshape.0 {
            roll = self._next_roll(y as i32, roll);
            let s = self.roll_mns(-roll);
            for tx in 0..xtiles {
                for i in 0..mns_len as usize {
                    m[(y, tx * mns_len as usize + i, 1)] = s[i] as i32;
                }
            }
        }

        m.slice(s![0..h, 0..w, ..]).to_owned()
    }

    #[allow(dead_code)]
    fn encode_position(&self, x: i32, y: i32) -> i8 {
        // Calculate the MNS sequence for the position
        let mns_value = self.calculate_mns(x, y);

        // Calculate the A sequences for the position
        let a_values = self.calculate_a_sequences(x, y);

        // Combine MNS and A values
        self.combine_sequences(mns_value, a_values)
    }

    #[allow(dead_code)]
    fn calculate_mns(&self, x: i32, y: i32) -> Vec<i8> {
        let mut mns_seq = Vec::new();
        let mut current_x = x;
        let mut current_y = y;

        for _ in 0..self.mns_length {
            let index = (current_y % 6) * 6 + (current_x % 6);
            mns_seq.push(self.mns[index as usize]);
            current_x /= 6;
            current_y /= 6;
        }

        mns_seq
    }

    #[allow(dead_code)]
    fn calculate_a_sequences(&self, x: i32, y: i32) -> Vec<Vec<i8>> {
        let mut a_seqs = Vec::new();

        for (i, seq) in self.a_sequences.iter().enumerate() {
            let order = self.a_orders[i] as usize;
            let mut a_seq = Vec::new();
            let mut current_x = x;
            let mut current_y = y;

            for _ in 0..order {
                let index = (current_y % 6) * 6 + (current_x % 6);
                a_seq.push(seq[index as usize % seq.len()]);
                current_x /= 6;
                current_y /= 6;
            }

            a_seqs.push(a_seq);
        }

        a_seqs
    }

    #[allow(dead_code)]
    fn combine_sequences(&self, mns: Vec<i8>, _a_seqs: Vec<Vec<i8>>) -> i8 {
        // For simplicity, just return the first MNS value
        // In a real implementation, this would combine all sequences
        mns[0]
    }

    pub fn decode_position(&self, section: &Array3<i8>) -> Option<(i64, i64)> {
        if section.dim() != (6, 6, 2) {
            return None;
        }

        // Extract the x-direction pattern from the first row
        let x_pattern: Vec<i32> = (0..6).map(|i| section[[0, i, 0]] as i32).collect();

        // Extract the y-direction pattern from the first column
        let y_pattern: Vec<i32> = (0..6).map(|i| section[[i, 0, 1]] as i32).collect();

        // Find the roll value for x-direction
        let x_roll = self.find_roll(&x_pattern)?;

        // Find the roll value for y-direction
        let y_roll = self.find_roll(&y_pattern)?;

        // Convert roll values back to section coordinates
        // This is a simplified version - in practice, you'd need to reverse the _next_roll logic
        Some((x_roll as i64, y_roll as i64))
    }

    fn find_roll(&self, pattern: &[i32]) -> Option<i32> {
        let mns_len = self.mns_length as i32;

        // Try all possible roll values
        for roll in 0..mns_len {
            let rolled_mns = self.roll_mns(roll);
            let rolled_pattern: Vec<i32> = rolled_mns.iter().take(6).map(|&x| x as i32).collect();

            if rolled_pattern == pattern {
                return Some(roll);
            }
        }

        None
    }
}

// Default codec configurations
pub fn anoto_6x6_a4_fixed() -> AnotoCodec {
    // Actual Anoto sequences from the patents
    let mns = vec![
        0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1,
        0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 1, 1, 1, 1, 0, 0,
        0, 1, 1,
    ];

    let a1 = vec![
        0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 2, 0, 1, 0, 0, 1, 0, 1, 0, 0, 2, 0, 0, 0, 1, 1, 0, 0, 0, 1,
        2, 0, 0, 1, 0, 2, 0, 0, 2, 0, 2, 0, 1, 1, 0, 1, 0, 1, 1, 0, 2, 0, 1, 2, 0, 1, 0, 1, 2, 0,
        2, 1, 0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 0, 2, 1, 0, 1, 0, 2, 1, 1, 0, 0, 1, 2, 1, 0, 1, 1, 2,
        0, 0, 0, 2, 1, 0, 2, 0, 2, 1, 1, 1, 0, 0, 2, 1, 2, 0, 1, 1, 1, 2, 0, 2, 0, 0, 1, 1, 2, 1,
        0, 0, 0, 2, 2, 0, 1, 0, 2, 2, 0, 0, 1, 2, 2, 0, 2, 0, 2, 2, 1, 0, 1, 2, 1, 2, 1, 0, 2, 1,
        2, 1, 1, 0, 2, 2, 1, 2, 1, 2, 0, 2, 2, 0, 2, 2, 2, 0, 1, 1, 2, 2, 1, 1, 0, 1, 2, 2, 2, 2,
        1, 2, 0, 0, 2, 2, 1, 1, 2, 1, 2, 2, 1, 0, 2, 2, 2, 2, 2, 0, 2, 1, 2, 2, 2, 1, 1, 1, 2, 1,
        1, 2, 0, 1, 2, 2, 1, 2, 2, 0, 1, 2, 1, 1, 1, 1, 2, 2, 2, 0, 0, 2, 1, 1, 2, 2,
    ];

    let a2 = vec![
        0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 2, 0, 1, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0,
        1, 1, 0, 1, 0, 0, 2, 0, 0, 0, 1, 2, 0, 1, 0, 1, 2, 1, 0, 0, 0, 2, 1, 1, 1, 0, 1, 1, 1, 0,
        2, 1, 0, 0, 1, 2, 1, 2, 1, 0, 1, 0, 2, 0, 1, 1, 0, 2, 0, 0, 1, 0, 2, 1, 2, 0, 0, 0, 2, 2,
        0, 0, 1, 1, 2, 0, 2, 0, 0, 2, 0, 2, 0, 1, 2, 0, 0, 2, 2, 1, 1, 0, 0, 2, 1, 0, 1, 1, 2, 1,
        0, 2, 0, 2, 2, 1, 0, 0, 2, 2, 2, 1, 0, 1, 2, 2, 0, 0, 2, 1, 2, 2, 1, 1, 1, 1, 1, 2, 0, 0,
        1, 2, 2, 1, 2, 0, 1, 1, 1, 2, 1, 1, 2, 0, 1, 2, 1, 1, 1, 2, 2, 0, 2, 2, 0, 1, 1, 2, 2, 2,
        2, 1, 2, 1, 2, 2, 0, 1, 2, 2, 2, 0, 2, 0, 2, 1, 1, 2, 2, 1, 0, 2, 2, 0, 2, 1, 0, 2, 1, 1,
        0, 2, 2, 2, 2, 0, 1, 0, 2, 2, 1, 2, 2, 2, 1, 1, 2, 1, 2, 0, 2, 2, 2,
    ];

    let a3 = vec![
        0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 0, 1,
    ];

    let a4_alt = vec![
        0, 0, 0, 0, 2, 2, 2, 2, 0, 2, 2, 2, 1, 0, 2, 2, 2, 0, 0, 2, 2, 1, 2, 0, 2, 2, 1, 1, 0, 2,
        2, 1, 0, 0, 2, 2, 0, 0, 0, 2, 1, 2, 2, 0, 2, 1, 2, 1, 0, 2, 1, 2, 0, 0, 2, 1, 1, 2, 0, 2,
        1, 1, 1, 0, 2, 1, 1, 0, 0, 2, 1, 0, 0, 0, 2, 0, 2, 2, 0, 2, 0, 2, 1, 0, 2, 0, 2, 0, 0, 2,
        0, 1, 0, 0, 2, 0, 0, 0, 0, 1, 2, 2, 2, 0, 1, 2, 2, 1, 0, 1, 2, 2, 0, 0, 1, 2, 1, 2, 0, 1,
        2, 1, 1, 0, 1, 2, 1, 0, 0, 1, 2, 0, 0, 0, 1, 1, 2, 2, 0, 1, 1, 2, 1, 0, 1, 1, 2, 0, 0, 1,
        1, 1, 2, 0, 1, 1, 1, 1, 2, 2, 2, 2, 1, 2, 2, 2, 1, 1, 2, 2, 1, 1, 1, 2, 1, 2, 2, 1, 2, 1,
        2, 1, 1, 2, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 1, 0, 2, 2, 0, 1, 0, 2, 1, 0,
        1, 0, 2, 0, 0, 1, 0, 1, 2, 0, 2, 0, 1, 2, 0, 1, 0, 1, 1, 0, 2, 0, 1, 1, 0, 1, 0, 1, 0, 0,
        1,
    ];

    let sns = [a1.clone(), a2.clone(), a3.clone(), a4_alt.clone()];
    let sns_lengths: Vec<usize> = sns.iter().map(|s| s.len()).collect();
    let sns_order = 5;
    let sns_cyclic: Vec<Vec<i8>> = sns.iter().map(|s| make_cyclic(s, sns_order)).collect();
    let pfactors = vec![3, 3, 2, 3];
    let mut bases = vec![1i32];
    for &p in &pfactors {
        bases.push(bases.last().unwrap() * p);
    }
    bases.pop(); // remove the last
    let delta_range = (5, 58);

    AnotoCodec::new(
        mns,
        63,
        vec![a1, a2, a3, a4_alt],
        vec![3i32, 3i32, 2i32, 3i32],
        (5, 58),
        sns_lengths,
        sns_cyclic,
        bases,
        pfactors,
        delta_range,
    )
}

fn make_cyclic(seq: &[i8], order: usize) -> Vec<i8> {
    let mut cyclic = seq.to_vec();
    cyclic.extend_from_slice(&seq[0..order - 1]);
    cyclic
}

pub fn gen_matrix(
    height: usize,
    width: usize,
    sect_u: i32,
    sect_v: i32,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let bitmatrix = generate_matrix_only(height, width, sect_u, sect_v)?;
    save_generated_matrix(&bitmatrix, height, width, sect_u, sect_v)?;
    Ok(())
}

pub fn generate_matrix_only(
    height: usize,
    width: usize,
    sect_u: i32,
    sect_v: i32,
) -> std::result::Result<Array3<i32>, Box<dyn std::error::Error>> {
    let codec = anoto_6x6_a4_fixed();
    let bitmatrix = codec.encode_bitmatrix((height, width), (sect_u, sect_v));
    Ok(bitmatrix)
}

pub fn save_generated_matrix(
    bitmatrix: &Array3<i32>,
    height: usize,
    width: usize,
    sect_u: i32,
    sect_v: i32,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let base_filename = format!("G__{}__{}__{}__{}", height, width, sect_u, sect_v);

    // Create output directory
    std::fs::create_dir_all("output")?;

    // Save as JSON
    crate::persist_json::save_as_json(bitmatrix, &base_filename)?;

    // Save as TXT
    crate::persist_json::save_as_txt(bitmatrix, &base_filename)?;

    // Generate PNG
    crate::make_plots::draw_dots(&bitmatrix.mapv(|x| x as i8), 1.0, &base_filename)?;

    // Generate PDF
    crate::pdf_dotpaper::gen_pdf::gen_pdf_from_matrix_data(
        bitmatrix,
        &format!("{}.pdf", base_filename),
        &crate::pdf_dotpaper::gen_pdf::PdfConfig::default(),
    )?;

    Ok(())
}

pub fn load_matrix_from_json(
    json_path: &str,
) -> std::result::Result<Array3<i32>, Box<dyn std::error::Error>> {
    let bitmatrix = crate::persist_json::load_array3_from_json(json_path)?;
    Ok(bitmatrix)
}

pub fn load_matrix_from_txt(
    txt_path: &str,
) -> std::result::Result<Array3<i32>, Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(txt_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let mut rows = Vec::new();
    for line in content.lines() {
        let mut row = Vec::new();
        let mut i = 0;
        let chars: Vec<char> = line.chars().collect();
        while i < chars.len() {
            if chars[i] == '[' {
                // Find the closing ]
                let mut j = i + 1;
                while j < chars.len() && chars[j] != ']' {
                    j += 1;
                }
                if j < chars.len() {
                    let inner: String = chars[i + 1..j].iter().collect();
                    let nums: Vec<i32> = inner
                        .split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    if nums.len() == 2 {
                        row.push([nums[0], nums[1]]);
                    }
                }
                i = j + 1;
            } else {
                i += 1;
            }
        }
        if !row.is_empty() {
            rows.push(row);
        }
    }

    if rows.is_empty() {
        return Err("No data found in txt file".into());
    }

    let height = rows.len();
    let width = rows[0].len();
    let mut bitmatrix = Array3::<i32>::zeros((height, width, 2));
    for (i, row) in rows.into_iter().enumerate() {
        for (j, pair) in row.into_iter().enumerate() {
            bitmatrix[[i, j, 0]] = pair[0];
            bitmatrix[[i, j, 1]] = pair[1];
        }
    }

    Ok(bitmatrix)
}

pub fn save_matrix_from_json(
    bitmatrix: &Array3<i32>,
    json_path: &str,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let (height, width, _) = bitmatrix.dim();

    // Try to extract sect_u and sect_v from the filename if it follows G__ pattern
    let stem = std::path::Path::new(json_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let (sect_u, sect_v) = if let Some(parts) = stem.strip_prefix("G__") {
        let nums: Vec<&str> = parts.split("__").collect();
        if nums.len() >= 4 {
            (nums[2].parse().unwrap_or(10), nums[3].parse().unwrap_or(2))
        } else {
            (10, 2)
        }
    } else {
        (10, 2)
    };

    let base_filename = format!("J__{}__{}__{}__{}", height, width, sect_u, sect_v);

    // Save as TXT
    crate::persist_json::save_as_txt(bitmatrix, &base_filename)?;

    // Generate PNG
    crate::make_plots::draw_dots(&bitmatrix.mapv(|x| x as i8), 1.0, &base_filename)?;

    // Generate PDF
    crate::pdf_dotpaper::gen_pdf::gen_pdf_from_matrix_data(
        bitmatrix,
        &format!("{}.pdf", base_filename),
        &crate::pdf_dotpaper::gen_pdf::PdfConfig::default(),
    )?;

    Ok(())
}

pub fn extract_6x6_section(
    bitmatrix: &Array3<i32>,
    pos: (i32, i32),
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let (rows, cols, _) = bitmatrix.dim();
    let (row, col) = (pos.0 as usize, pos.1 as usize);

    println!("Matrix size [{}, {}]", rows, cols);
    println!("Requested position ({}, {})", row, col);

    let max_row = rows.saturating_sub(6);
    let max_col = cols.saturating_sub(6);
    println!(
        "Maximum 6x6 position for this matrix is ({}, {})",
        max_row, max_col
    );

    // Create output directory
    std::fs::create_dir_all("output")?;

    if row > max_row || col > max_col {
        println!("Position out of bounds, returning zeroed 6x6 section");
        // Create a zeroed 6x6 section
        let zeroed_section = Array3::<i32>::zeros((6, 6, 2));

        // Save the zeroed section
        let filename = format!("section_{}_{}", row, col);
        crate::persist_json::save_as_json(&zeroed_section, &filename)?;
        crate::persist_json::save_as_txt(&zeroed_section, &filename)?;

        return Ok(());
    }

    // Extract the 6x6 section
    let section = bitmatrix.slice(s![row..row + 6, col..col + 6, ..]);
    let section = section.to_owned();

    // Save the section
    let filename = format!("section_{}_{}", row, col);
    crate::persist_json::save_as_json(&section, &filename)?;
    crate::persist_json::save_as_txt(&section, &filename)?;

    Ok(())
}

pub fn gen_matrix_from_json(
    json_path: &str,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let bitmatrix = crate::persist_json::load_array3_from_json(json_path)?;
    let (height, width, _) = bitmatrix.dim();

    // Try to extract sect_u and sect_v from the filename if it follows G__ pattern
    let stem = std::path::Path::new(json_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let (sect_u, sect_v) = if let Some(parts) = stem.strip_prefix("G__") {
        let nums: Vec<&str> = parts.split("__").collect();
        if nums.len() >= 4 {
            (nums[2].parse().unwrap_or(10), nums[3].parse().unwrap_or(2))
        } else {
            (10, 2)
        }
    } else {
        (10, 2)
    };

    let base_filename = format!("J__{}__{}__{}__{}", height, width, sect_u, sect_v);

    // Save as TXT
    crate::persist_json::save_as_txt(&bitmatrix, &base_filename)?;

    // Generate PNG
    crate::make_plots::draw_dots(&bitmatrix.mapv(|x| x as i8), 1.0, &base_filename)?;

    // Generate PDF
    crate::pdf_dotpaper::gen_pdf::gen_pdf_from_matrix_data(
        &bitmatrix,
        &format!("{}.pdf", base_filename),
        &crate::pdf_dotpaper::gen_pdf::PdfConfig::default(),
    )?;

    Ok(())
}
