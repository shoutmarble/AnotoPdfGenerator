pub mod anoto_matrix;
pub mod codec;
pub mod controls;
pub mod decode_utils;
pub mod fonts;
pub mod make_plots;
pub mod pdf_dotpaper;
pub mod persist_json;

pub use anoto_matrix::{
    extract_6x6_section, gen_matrix, gen_matrix_from_json, generate_matrix_only,
    load_matrix_from_json, load_matrix_from_txt, save_generated_matrix, save_matrix_from_json,
};
pub use codec::anoto_6x6_a4_fixed;
pub use decode_utils::decode_position;
