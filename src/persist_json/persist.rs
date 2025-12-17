use ndarray::{Array2, Array3};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub fn save_bitmatrix_text(bitmatrix: &Array3<i8>, filename: &str) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;
    for row in 0..bitmatrix.dim().0 {
        for col in 0..bitmatrix.dim().1 {
            let x_bit = bitmatrix[[row, col, 0]];
            let y_bit = bitmatrix[[row, col, 1]];
            write!(file, "[{} {}]", x_bit, y_bit)?;
            if col < bitmatrix.dim().1 - 1 {
                write!(file, " ")?;
            }
        }
        writeln!(file)?;
    }
    Ok(())
}

pub fn save_bitmatrix_json(bitmatrix: &Array3<i8>, filename: &str) -> Result<(), Box<dyn Error>> {
    let mut data = Vec::new();
    for row in 0..bitmatrix.dim().0 {
        let mut row_data = Vec::new();
        for col in 0..bitmatrix.dim().1 {
            let pair = vec![bitmatrix[[row, col, 0]], bitmatrix[[row, col, 1]]];
            row_data.push(pair);
        }
        data.push(row_data);
    }
    let file = File::create(filename)?;
    serde_json::to_writer(file, &data)?;
    Ok(())
}

pub fn save_as_json(bitmatrix: &Array3<i32>, base_filename: &str) -> Result<(), Box<dyn Error>> {
    let filename = format!("output/{}.json", base_filename);
    let data: Vec<Vec<Vec<i32>>> = bitmatrix
        .outer_iter()
        .map(|row| row.outer_iter().map(|col| col.to_vec()).collect())
        .collect();
    let file = File::create(filename)?;
    serde_json::to_writer(file, &data)?;
    Ok(())
}

pub fn save_as_txt(bitmatrix: &Array3<i32>, base_filename: &str) -> Result<(), Box<dyn Error>> {
    let filename = format!("output/{}.txt", base_filename);
    let mut file = File::create(filename)?;
    for row in bitmatrix.outer_iter() {
        for col in row.outer_iter() {
            write!(file, "[{} {}] ", col[0], col[1])?;
        }
        writeln!(file)?;
    }
    Ok(())
}

pub fn load_from_json(json_path: &str) -> Result<Array2<i32>, Box<dyn Error>> {
    let file = File::open(json_path)?;
    let data: Vec<Vec<i32>> = serde_json::from_reader(file)?;
    let height = data.len();
    let width = data[0].len();
    let mut bitmatrix = Array2::zeros((height, width));
    for (i, row) in data.into_iter().enumerate() {
        for (j, val) in row.into_iter().enumerate() {
            bitmatrix[[i, j]] = val;
        }
    }
    Ok(bitmatrix)
}

pub fn load_array3_from_json(json_path: &str) -> Result<Array3<i32>, Box<dyn Error>> {
    let file = File::open(json_path)?;
    let data: Vec<Vec<Vec<i32>>> = serde_json::from_reader(file)?;
    let height = data.len();
    let width = data[0].len();
    let depth = data[0][0].len();
    let mut bitmatrix = Array3::zeros((height, width, depth));
    for (i, row) in data.into_iter().enumerate() {
        for (j, col) in row.into_iter().enumerate() {
            for (k, val) in col.into_iter().enumerate() {
                bitmatrix[[i, j, k]] = val;
            }
        }
    }
    Ok(bitmatrix)
}

pub fn load_6x6_section(file_path: &str) -> Result<Array3<i8>, Box<dyn Error>> {
    // Try to determine file format by extension or content
    if file_path.ends_with(".json") {
        let array3_i32 = load_array3_from_json(file_path)?;
        if array3_i32.dim() != (6, 6, 2) {
            return Err("Section file must contain a 6x6x2 array".into());
        }
        let array3_i8 = array3_i32.mapv(|x| x as i8);
        Ok(array3_i8)
    } else if file_path.ends_with(".txt") {
        load_6x6_from_text(file_path)
    } else {
        // Try JSON first, then TXT
        match load_array3_from_json(file_path) {
            Ok(array3_i32) => {
                if array3_i32.dim() != (6, 6, 2) {
                    return Err("Section file must contain a 6x6x2 array".into());
                }
                let array3_i8 = array3_i32.mapv(|x| x as i8);
                Ok(array3_i8)
            }
            Err(_) => load_6x6_from_text(file_path),
        }
    }
}

pub fn load_6x6_from_text(file_path: &str) -> Result<Array3<i8>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut data = Vec::new();

    for line_result in reader.lines() {
        let line = line_result?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut row_data = Vec::new();
        // Parse pairs like "[1 1] [1 0] ..."
        let pairs: Vec<&str> = line.split("] [").collect();
        for (i, pair) in pairs.iter().enumerate() {
            let clean_pair = if i == 0 {
                pair.trim_start_matches('[')
            } else if i == pairs.len() - 1 {
                pair.trim_end_matches(']')
            } else {
                *pair
            };

            let nums: Vec<i8> = clean_pair
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();

            if nums.len() != 2 {
                return Err(format!("Invalid pair format in line: {}", line).into());
            }

            row_data.push(nums);
        }

        if row_data.len() != 6 {
            return Err(format!(
                "Expected 6 pairs per row, got {} in line: {}",
                row_data.len(),
                line
            )
            .into());
        }

        data.push(row_data);
    }

    if data.len() != 6 {
        return Err(format!("Expected 6 rows, got {}", data.len()).into());
    }

    // Convert to Array3<i8>
    let mut array3 = Array3::zeros((6, 6, 2));
    for (i, row) in data.into_iter().enumerate() {
        for (j, pair) in row.into_iter().enumerate() {
            array3[[i, j, 0]] = pair[0];
            array3[[i, j, 1]] = pair[1];
        }
    }

    Ok(array3)
}
