use ndarray::{Array2, Array3, Axis, s};
use std::error::Error;
use std::fmt;

pub fn anoto_6x6_a4_fixed() -> AnotoCodec {
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

    AnotoCodec::new(
        mns.into_iter().map(|x| x as i8).collect(),
        6,
        vec![a1, a2, a3, a4_alt]
            .into_iter()
            .map(|a| a.into_iter().map(|x| x as i8).collect())
            .collect(),
        vec![3, 3, 2, 3],
        (5, 58),
    )
}

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

impl DecodingError {
    pub fn new(message: &str) -> Self {
        DecodingError {
            message: message.to_string(),
        }
    }
}

// Number basis for Chinese Remainder Theorem calculations
pub struct NumberBasis {
    factors: Vec<i64>,
}

impl NumberBasis {
    fn new(factors: Vec<i64>) -> Self {
        NumberBasis { factors }
    }

    fn reconstruct(&self, coeffs: &[i8]) -> i64 {
        let mut result = 0i64;
        let mut base = 1i64;
        for (i, &coeff) in coeffs.iter().enumerate() {
            result += coeff as i64 * base;
            base *= self.factors[i];
        }
        result
    }

    fn project(&self, values: &[i64]) -> Vec<Vec<i8>> {
        let mut result = vec![vec![0i8; self.factors.len()]; values.len()];
        for (i, &val) in values.iter().enumerate() {
            let mut remaining = val;
            for (j, &factor) in self.factors.iter().enumerate() {
                result[i][j] = (remaining % factor) as i8;
                remaining /= factor;
            }
        }
        result
    }
}

// Chinese Remainder Theorem solver
pub struct CRT {
    moduli: Vec<i64>,
}

impl CRT {
    fn new(moduli: Vec<i64>) -> Self {
        CRT { moduli }
    }

    fn solve(&self, remainders: &[i64]) -> Result<i64, DecodingError> {
        if remainders.len() != self.moduli.len() {
            return Err(DecodingError::new("Remainders and moduli length mismatch"));
        }

        let mut result = 0i64;
        let product: i64 = self.moduli.iter().product();

        for (&remainder, &modulus) in remainders.iter().zip(self.moduli.iter()) {
            let partial_product = product / modulus;
            let inverse = self.mod_inverse(partial_product, modulus)?;
            result = (result + remainder * partial_product * inverse) % product;
        }

        Ok(result)
    }

    fn mod_inverse(&self, a: i64, m: i64) -> Result<i64, DecodingError> {
        let (gcd, x, _) = self.extended_gcd(a, m);
        if gcd != 1 {
            return Err(DecodingError::new("Modular inverse does not exist"));
        }
        Ok((x % m + m) % m)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn extended_gcd(&self, a: i64, b: i64) -> (i64, i64, i64) {
        if a == 0 {
            return (b, 0, 1);
        }
        let (gcd, x1, y1) = self.extended_gcd(b % a, a);
        let x = y1 - (b / a) * x1;
        let y = x1;
        (gcd, x, y)
    }
}

// Main Anoto codec implementation
pub struct AnotoCodec {
    mns: Vec<i8>,
    pub mns_length: usize,
    mns_cyclic: Vec<i8>,
    mns_order: usize,
    _sns_order: usize,
    _sns: Vec<Vec<i8>>,
    sns_lengths: Vec<usize>,
    sns_cyclic: Vec<Vec<i8>>,
    num_basis: NumberBasis,
    crt: CRT,
    delta_range: (i32, i32),
}

impl AnotoCodec {
    pub fn new(
        mns: Vec<i8>,
        mns_order: usize,
        sns: Vec<Vec<i8>>,
        pfactors: Vec<i32>,
        delta_range: (i32, i32),
    ) -> Self {
        let mns_length = mns.len();
        let mns_cyclic = make_cyclic(&mns, mns_order);
        let sns_order = mns_order - 1;
        let sns_lengths: Vec<usize> = sns.iter().map(|s| s.len()).collect();
        let sns_cyclic: Vec<Vec<i8>> = sns.iter().map(|s| make_cyclic(s, sns_order)).collect();
        let num_basis = NumberBasis::new(pfactors.into_iter().map(|x| x as i64).collect());
        let crt = CRT::new(sns_lengths.iter().map(|&l| l as i64).collect());

        AnotoCodec {
            mns,
            mns_length,
            mns_cyclic,
            mns_order,
            _sns_order: sns_order,
            _sns: sns,
            sns_lengths,
            sns_cyclic,
            num_basis,
            crt,
            delta_range,
        }
    }

    pub fn encode_bitmatrix(&self, shape: (usize, usize), section: (i32, i32)) -> Array3<i8> {
        // Find nearest multiples of MNS length for ease of generation
        let mshape = (
            (self.mns_length as f64 * (shape.0 as f64 / self.mns_length as f64).ceil()) as usize,
            (self.mns_length as f64 * (shape.1 as f64 / self.mns_length as f64).ceil()) as usize,
        );

        let mut m = Array3::<i8>::zeros((mshape.0, mshape.1, 2));

        // x-direction
        let mut roll = section.0 % self.mns_length as i32;
        let _ytiles = mshape.0 / self.mns_length;

        for x in 0..mshape.1 {
            roll = self.next_roll(x as i32, roll);
            let rolled_mns = rotate_vec(&self.mns, -(roll as isize));
            let tiled: Vec<i8> = rolled_mns.iter().cycle().take(mshape.0).cloned().collect();

            for y in 0..mshape.0 {
                m[[y, x, 0]] = tiled[y];
            }
        }

        // y-direction
        let mut roll = section.1 % self.mns_length as i32;
        let _xtiles = mshape.1 / self.mns_length;

        for y in 0..mshape.0 {
            roll = self.next_roll(y as i32, roll);
            let rolled_mns = rotate_vec(&self.mns, -(roll as isize));
            let tiled: Vec<i8> = rolled_mns.iter().cycle().take(mshape.1).cloned().collect();

            for x in 0..mshape.1 {
                m[[y, x, 1]] = tiled[x];
            }
        }

        // Return the requested shape
        m.slice(s![0..shape.0, 0..shape.1, ..]).to_owned()
    }

    fn next_roll(&self, pos: i32, prev_roll: i32) -> i32 {
        if pos == 0 {
            return prev_roll;
        }
        (prev_roll + self.delta(pos - 1)) % self.mns_length as i32
    }

    fn delta(&self, pos: i32) -> i32 {
        let rs: Vec<usize> = self
            .sns_lengths
            .iter()
            .map(|&len| (pos as usize) % len)
            .collect();

        let coeffs: Vec<i8> = rs
            .iter()
            .zip(self.sns_cyclic.iter())
            .map(|(&r, s)| s[r])
            .collect();

        (self.num_basis.reconstruct(&coeffs) + self.delta_range.0 as i64) as i32
    }

    pub fn decode_position(&self, bits: &Array3<i8>) -> Result<(i32, i32), DecodingError> {
        if bits.dim().2 != 2 {
            return Err(DecodingError::new("Expected (M,N,2) matrix"));
        }

        let sub_bits = bits.slice(s![0..self.mns_order, 0..self.mns_order, ..]);

        // Decode x (transpose for x-direction)
        let x_bits = sub_bits.slice(s![.., .., 0]).t().to_owned();
        let x = self.decode_position_along_direction(&x_bits)?;

        // Decode y
        let y_bits = sub_bits.slice(s![.., .., 1]).to_owned();
        let y = self.decode_position_along_direction(&y_bits)?;

        Ok((x, y))
    }

    fn decode_position_along_direction(&self, bits: &Array2<i8>) -> Result<i32, DecodingError> {
        let mut locs = Vec::new();

        for row in bits.axis_iter(Axis(0)) {
            let row_vec: Vec<i8> = row.to_vec();
            match find_subsequence(&self.mns_cyclic, &row_vec) {
                Some(pos) => locs.push(pos as i32),
                None => return Err(DecodingError::new("Failed to find partial sequence in MNS")),
            }
        }

        // Compute differences
        let mut deltae = Vec::new();
        for i in 1..locs.len() {
            let diff = (locs[i] - locs[i - 1] + self.mns_length as i32) % self.mns_length as i32;
            if diff < self.delta_range.0 || diff > self.delta_range.1 {
                return Err(DecodingError::new("Delta value out of range"));
            }
            deltae.push(diff - self.delta_range.0);
        }

        // Project to coefficients
        let deltae_i64: Vec<i64> = deltae.iter().map(|&x| x as i64).collect();
        let coeffs = self.num_basis.project(&deltae_i64);

        // Find positions in secondary sequences
        let mut ps = Vec::new();
        for (i, sns_seq) in self.sns_cyclic.iter().enumerate() {
            let coeff_seq: Vec<i8> = coeffs.iter().map(|c| c[i]).collect();
            match find_subsequence(sns_seq, &coeff_seq) {
                Some(pos) => ps.push(pos as i64),
                None => return Err(DecodingError::new("Failed to find coefficients in SNS")),
            }
        }

        self.crt.solve(&ps).map(|x| x as i32)
    }

    pub fn decode_section(
        &self,
        bits: &Array3<i8>,
        pos: (i32, i32),
    ) -> Result<(i32, i32), DecodingError> {
        let px_seq = bits.slice(s![0..self.mns_order, 0, 0]).to_vec();
        let py_seq = bits.slice(s![0, 0..self.mns_order, 1]).to_vec();

        let px_mns = find_subsequence(&self.mns_cyclic, &px_seq)
            .ok_or_else(|| DecodingError::new("Failed to find x sequence in MNS"))?;
        let py_mns = find_subsequence(&self.mns_cyclic, &py_seq)
            .ok_or_else(|| DecodingError::new("Failed to find y sequence in MNS"))?;

        let sx = self.integrate_roll(pos.0, 0);
        let sy = self.integrate_roll(pos.1, 0);

        let section_x = (px_mns as i32 - pos.1 - sx) % self.mns_length as i32;
        let section_y = (py_mns as i32 - pos.0 - sy) % self.mns_length as i32;

        Ok((section_x, section_y))
    }

    pub fn integrate_roll(&self, pos: i32, first_roll: i32) -> i32 {
        let mut r = 0i64;
        for i in 0..pos {
            r += self.delta(i) as i64;
        }
        ((first_roll as i64 + r) % self.mns_length as i64) as i32
    }

    pub fn encode_patch(
        &self,
        pos: (i32, i32),
        size: (usize, usize),
        section_start_rolls: (i32, i32),
    ) -> Array3<i8> {
        let (x_start, y_start) = pos;
        let (w, h) = size;
        let mut m = Array3::<i8>::zeros((h, w, 2));

        // x-direction
        for c in 0..w {
            let abs_x = x_start + c as i32;
            let roll = self.integrate_roll(abs_x, section_start_rolls.0);
            let rolled_mns = rotate_vec(&self.mns, -(roll as isize));

            for r in 0..h {
                let abs_y = y_start + r as i32;
                m[[r, c, 0]] = rolled_mns[(abs_y as usize) % self.mns_length];
            }
        }

        // y-direction
        for r in 0..h {
            let abs_y = y_start + r as i32;
            let roll = self.integrate_roll(abs_y, section_start_rolls.1);
            let rolled_mns = rotate_vec(&self.mns, -(roll as isize));

            for c in 0..w {
                let abs_x = x_start + c as i32;
                m[[r, c, 1]] = rolled_mns[(abs_x as usize) % self.mns_length];
            }
        }

        m
    }
}

// Default codec configurations
fn make_cyclic(seq: &[i8], order: usize) -> Vec<i8> {
    let mut result = seq.to_vec();
    result.extend_from_slice(&seq[0..(order - 1)]);
    result
}

fn rotate_vec(vec: &[i8], shift: isize) -> Vec<i8> {
    let len = vec.len() as isize;
    let shift = ((shift % len) + len) % len;
    let mut result = Vec::with_capacity(vec.len());

    // For positive shift, rotate right; for negative shift, rotate left
    if shift >= 0 {
        // Rotate right by shift
        result.extend_from_slice(&vec[(len - shift) as usize..]);
        result.extend_from_slice(&vec[0..(len - shift) as usize]);
    } else {
        // Rotate left by -shift
        let left_shift = (-shift) as usize;
        result.extend_from_slice(&vec[left_shift..]);
        result.extend_from_slice(&vec[0..left_shift]);
    }

    result
}

fn find_subsequence(haystack: &[i8], needle: &[i8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
