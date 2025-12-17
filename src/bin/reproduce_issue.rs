use anoto_pdf::codec::anoto_6x6_a4_fixed;
use ndarray::Array3;
use serde_json::Value;

fn main() {
    let input_json = r#"
[
  [[1,1],[1,1],[0,1],[1,1],[0,0],[0,1],[1,0],[1,0]],
  [[1,1],[0,0],[0,0],[0,1],[0,0],[1,0],[0,0],[0,0]],
  [[1,0],[0,0],[0,0],[1,1],[1,1],[1,1],[0,0],[1,1]],
  [[1,1],[1,0],[1,1],[1,1],[0,1],[1,0],[0,0],[1,1]],
  [[0,0],[0,0],[1,1],[1,0],[1,1],[1,0],[0,1],[0,0]],
  [[1,0],[0,1],[1,1],[0,1],[0,1],[1,1],[0,0],[1,1]],
  [[0,1],[0,0],[0,0],[0,0],[1,0],[0,0],[0,0],[1,1]],
  [[0,1],[0,0],[1,1],[1,1],[0,0],[1,1],[1,1],[0,0]],
  [[1,1],[1,0],[1,0],[0,0],[0,0],[0,1],[0,1],[0,1]]
]
"#;

    let result = decode_json_input(input_json);
    println!("Result:\n{}", result);
}

fn decode_json_input(input: &str) -> String {
    let parsed: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(e) => return format!("JSON Parse Error: {}", e),
    };

    // Helper to map direction to bits
    let map_direction = |dir: &str| -> Option<(i8, i8)> {
        match dir {
            "↑" | "Up" | "up" => Some((0, 0)),
            "←" | "Left" | "left" => Some((1, 0)),
            "→" | "Right" | "right" => Some((0, 1)),
            "↓" | "Down" | "down" => Some((1, 1)),
            _ => None,
        }
    };

    let map_coords = |x: i64, y: i64| -> Option<(i8, i8)> {
        match (x, y) {
            (0, 0) => Some((0, 0)), // Up
            (1, 0) => Some((1, 0)), // Left
            (0, 1) => Some((0, 1)), // Right
            (1, 1) => Some((1, 1)), // Down
            _ => None,
        }
    };

    let map_cell = |cell: &Value| -> Option<(i8, i8)> {
        if let Some(cell_arr) = cell.as_array() {
            // Case: [0,0]
            if cell_arr.len() == 2 {
                let x = cell_arr[0].as_i64();
                let y = cell_arr[1].as_i64();
                if let (Some(x), Some(y)) = (x, y) {
                    return map_coords(x, y);
                } else {
                    // Maybe it's ["↑"]
                    if let Some(s) = cell_arr[0].as_str() {
                        return map_direction(s);
                    }
                }
            } else if cell_arr.len() == 1 {
                // Case: ["↑"]
                if let Some(s) = cell_arr[0].as_str() {
                    return map_direction(s);
                }
            }
        } else if let Some(s) = cell.as_str() {
            // Case: "↑"
            return map_direction(s);
        }
        None
    };

    // Find the matrix
    let mut matrix_val = &parsed;
    while let Some(arr) = matrix_val.as_array() {
        if arr.is_empty() {
            return "Empty array".to_string();
        }

        if let Some(first_row) = arr[0].as_array() {
            if first_row.is_empty() {
                return "Empty row".to_string();
            }

            let is_cell = |v: &Value| {
                if v.is_string() {
                    true
                } else if let Some(a) = v.as_array() {
                    // It's a cell if it's [num, num] or ["str"]
                    // It's NOT a cell if it's [[...]] (array of arrays)
                    !a.is_empty() && !a[0].is_array()
                } else {
                    false
                }
            };

            if is_cell(&first_row[0]) {
                // Found the matrix!
                break;
            } else {
                // Go deeper
                matrix_val = &arr[0];
            }
        } else {
            return "Invalid structure".to_string();
        }
    }

    let rows = match matrix_val.as_array() {
        Some(a) => a,
        None => return "Could not find matrix array".to_string(),
    };

    let mut grid = Vec::new();
    for (r_idx, row_val) in rows.iter().enumerate() {
        let row_arr = match row_val.as_array() {
            Some(a) => a,
            None => return format!("Row {} is not an array", r_idx),
        };
        let mut row_bits = Vec::new();
        for (c_idx, cell_val) in row_arr.iter().enumerate() {
            match map_cell(cell_val) {
                Some(bits) => row_bits.push(bits),
                None => return format!("Invalid cell at [{}, {}]", r_idx, c_idx),
            }
        }
        grid.push(row_bits);
    }

    let height = grid.len();
    if height < 6 {
        return "Matrix too small (height < 6)".to_string();
    }
    let width = grid[0].len();
    if width < 6 {
        return "Matrix too small (width < 6)".to_string();
    }

    // Check all rows have same width
    if grid.iter().any(|r| r.len() != width) {
        return "Matrix rows have inconsistent lengths".to_string();
    }

    let mut results = String::new();
    let codec = anoto_6x6_a4_fixed();

    println!("Grid size: {}x{}", height, width);

    for r in 0..=(height - 6) {
        for c in 0..=(width - 6) {
            // Extract 6x6
            let mut bits = Array3::<i8>::zeros((6, 6, 2));
            for i in 0..6 {
                for j in 0..6 {
                    let (b0, b1) = grid[r + i][c + j];
                    bits[[i, j, 0]] = b0;
                    bits[[i, j, 1]] = b1;
                }
            }

            match codec.decode_position(&bits) {
                Ok((x, y)) => {
                    if !results.is_empty() {
                        results.push('\n');
                    }
                    results.push_str(&format!("Position: ({}, {})", x, y));
                }
                Err(e) => {
                    println!("Failed at ({}, {}): {}", r, c, e);
                }
            }
        }
    }

    if results.is_empty() {
        "No valid positions found".to_string()
    } else {
        results
    }
}
