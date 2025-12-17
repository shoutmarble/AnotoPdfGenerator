use ndarray::s;
//   POS: (np.int64(7), np.int64(3))
//   SEC: (np.int64(10), np.int64(2))
//   [1 0] [1 0] [0 0] [1 0] [0 1] [0 0] [1 0] [1 1] [1 1] [1 1] [0 1] [0 1] [1 0] [1 1] [1 0] [1 0] [0 1] [1 0] [1 0] [0 0]  0
//   [1 0] [0 0] [0 1] [0 1] [0 1] [1 1] [0 1] [0 0] [0 1] [0 0] [0 0] [1 1] [0 0] [1 0] [1 0] [1 0] [0 1] [0 1] [1 1] [0 0]  1
//   [1 1] [0 1] [0 0] [1 1] [1 0] [1 0] [0 1] [1 0] [0 0] [1 0] [0 0] [0 1] [0 1] [1 1] [0 0] [1 1] [0 1] [1 1] [0 0] [0 0]  2
//   [1 0] [1 1] [1 0] [1 0] [0 0] [1 0] [0 1] [1 1] [0 1] [0 0] [0 1] [0 1] [1 1] [1 0] [1 0] [0 1] [0 0] [1 1] [0 0] [1 1]  3
//   [0 0] [0 1] [1 1] [1 1] [1 0] [1 1] [0 1] [0 1] [0 0] [1 0] [1 1] [1 0] [1 1] [0 0] [1 1] [1 0] [1 0] [1 0] [0 1] [0 0]  4
//   [1 0] [0 0] [1 0] [0 0] [0 0] [1 0] [0 1] [1 0] [1 0] [0 1] [1 1] [1 1] [0 1] [1 1] [1 0] [0 1] [0 0] [1 0] [0 1] [0 0]  5
//   [0 1] [0 1] [0 1] [0 1] [1 0] [0 0] [0 0] [1 1] [1 1] [0 0] [1 0] [1 0] [1 0] [0 0] [0 0] [0 1] [0 0] [0 0] [0 1] [1 1]  6
//   [0 1] [0 0] [1 1] [1 0] [0 1] [1 0] [1 0] [0 0] [1 1] [0 0] [0 1] [1 1] [0 0] [0 1] [0 1] [1 0] [1 0] [0 1] [0 1] [1 0]  7
//   [1 1] [1 1] [1 1] [0 1] [0 0] [0 1] [0 0] [0 0] [0 1] [1 0] [1 0] [1 0] [1 0] [1 1] [1 1] [0 1] [1 0] [0 1] [1 1] [1 1]  8
//   [0 1] [1 1] [1 0] [1 1] [0 1] [0 1] [0 0] [1 0] [1 1] [0 0] [1 1] [0 0] [1 1] [0 0] [0 0] [0 0] [1 1] [1 0] [0 1] [1 1]  9
//   [0 1] [1 1] [0 1] [0 0] [1 1] [1 0] [1 0] [1 1] [1 0] [1 0] [1 0] [1 0] [1 1] [0 1] [1 1] [0 0] [1 1] [1 1] [0 1] [1 0]  10
//   [0 0] [0 1] [0 1] [1 0] [0 0] [0 0] [1 0] [0 0] [1 0] [1 1] [0 0] [0 0] [1 1] [0 1] [0 1] [0 1] [1 1] [0 0] [1 1] [0 0]  11
//   [0 0] [1 0] [1 1] [0 0] [1 1] [0 1] [1 0] [1 1] [0 1] [0 0] [0 0] [0 1] [0 1] [0 0] [1 1] [1 0] [0 1] [0 1] [1 1] [1 1]
//   [1 0] [1 0] [0 0] [0 0] [1 1] [0 0] [1 0] [0 1] [0 1] [1 1] [1 1] [1 1] [0 0] [1 1] [0 0] [1 0] [1 1] [0 0] [1 0] [0 0]
//   [1 0] [1 0] [1 1] [0 0] [0 0] [0 0] [1 0] [1 1] [1 1] [1 1] [0 0] [0 1] [0 1] [1 1] [0 0] [1 0] [0 1] [0 0] [1 1] [0 0]
//   [1 0] [0 0] [0 0] [1 1] [1 0] [1 0] [0 1] [1 1] [0 1] [0 1] [1 1] [0 0] [1 1] [1 0] [0 0] [0 1] [0 0] [0 0] [1 0] [1 0]
//   [0 1] [0 0] [1 0] [0 0] [1 1] [1 1] [1 0] [1 0] [1 0] [0 0] [0 0] [0 0] [1 1] [0 0] [1 0] [1 1] [1 1] [0 1] [0 1] [0 1]
//   [1 0] [1 1] [0 1] [1 0] [0 0] [1 1] [0 1] [1 0] [0 1] [1 0] [1 1] [0 1] [0 1] [1 1] [0 0] [1 0] [0 0] [1 1] [1 1] [0 0]
//   [1 0] [0 1] [0 1] [1 1] [0 1] [0 0] [0 0] [0 0] [1 1] [1 1] [0 0] [1 0] [0 0] [1 0] [1 0] [1 0] [0 1] [0 0] [0 0] [0 1]
//   [1 0] [1 0] [0 0] [0 1] [1 1] [1 0] [1 0] [0 0] [0 0] [0 0] [0 0] [1 1] [0 0] [1 0] [1 1] [0 1] [0 1] [0 1] [0 1] [0 0]

//   (0, 0)
//   [1 0] [1 0] [0 0] [1 0] [0 1] [0 0]
//   [1 0] [0 0] [0 1] [0 1] [0 1] [1 1]
//   [1 1] [0 1] [0 0] [1 1] [1 0] [1 0]
//   [1 0] [1 1] [1 0] [1 0] [0 0] [1 0]
//   [0 0] [0 1] [1 1] [1 1] [1 0] [1 1]
//   [1 0] [0 0] [1 0] [0 0] [0 0] [1 0]

//   (11, 11)
//   [0 0] [1 1] [0 1] [0 1] [0 1] [1 1]
//   [0 1] [0 1] [0 0] [1 1] [1 0] [0 1]
//   [1 1] [0 0] [1 1] [0 0] [1 0] [1 1]
//   [0 1] [0 1] [1 1] [0 0] [1 0] [0 1]
//   [0 0] [1 1] [1 0] [0 0] [0 1] [0 0]
//   [0 0] [1 1] [0 0] [1 0] [1 1] [1 1]

/**
 * Get a 6x6 section from the bitmatrix at the specified position.
 * If the position is out of bounds, return a zeroed 6x6 section.
 * Save the 6x6 to the output folder as section_{row}_{col}.json
 * and section_{row}_{col}.txt
 */
pub fn get_6x6_section(bitmatrix: &ndarray::Array3<i8>, pos: (i64, i64)) -> ndarray::Array3<i8> {
    let (start_row, start_col) = (pos.0 as usize, pos.1 as usize);
    let section = bitmatrix.slice(s![start_row..start_row + 6, start_col..start_col + 6, ..]);
    section.to_owned()
}

/**
* return a 6x6 from a generated anoto matrix
* saved to a file named section_{row}_{col}.json and section_{row}_{col}.txt
* out of bounds returns a zeroed 6x6
* out of bounds return a message of the matrix size and
* matrix size in rows and cols [rows, cols] and the requested position
* print the maximum 6x6 position for that matrix [max_row, max_col]
*
* Matrix size [20,20]
* Requested position (15, 15)
* Maximum 6x6 position for this matrix is (14, 14)
*
* Example usage:
* anoto.exe -pos 7 3 -g 20 20 10 10
* anoto.exe -pos 7 3 -g 20 20 (default section 10 10)
* anoto.exe -pos 7 3 (default area 20 20 default section 10 10)

* anoto.exe -pos 7 3 -j PY__20_20__10_10.json
* anoto.exe -pos 7 3 -g 20 20 (default section 10 10)
* anoto.exe -pos 7 3 (default area 20 20 default section 10 10)

* either from json or generated
*/
pub fn find_position(
    bitmatrix: &ndarray::Array3<i8>,
    section: &ndarray::Array3<i8>,
) -> Option<(i64, i64)> {
    let (rows, cols) = (bitmatrix.dim().0, bitmatrix.dim().1);
    for r in 0..=rows - 6 {
        for c in 0..=cols - 6 {
            let candidate = get_6x6_section(bitmatrix, (r as i64, c as i64));
            if candidate == *section {
                return Some((r as i64, c as i64));
            }
        }
    }
    None
}

/**
 * Given a 6x6 section, decode the position using the anoto codec
 * return (row, col) as (i64, i64)
 *
 * Reference: https://github.com/cheind/py-microdots/blob/develop/examples/hello_anoto.py
 *
 * ```text
 *     pos = codec.decode_position(S)
 *     sec = codec.decode_section(S, pos=pos)
 *     rot = codec.decode_rotation(R)
 * ```
 *
 * If position cannot be decoded, return None
 * If position is decoded, return Some((row, col))
 *  
 * Example usage:
 *      anoto.exe --decode file_6x6.json | -decode file_6x6.txt
 *      anoto.exe -d file_6x6.json | -d file_6x6.txt
 *      POS (7, 3)
 */
pub fn decode_position(section: &ndarray::Array3<i8>) -> Option<(i64, i64)> {
    let codec = crate::codec::anoto_6x6_a4_fixed();
    match codec.decode_position(section) {
        Ok((x, y)) => Some((x as i64, y as i64)),
        Err(_) => None,
    }
}
