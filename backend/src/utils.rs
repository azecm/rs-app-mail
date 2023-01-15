use std::fmt::LowerHex;

use sha2::{Digest, Sha512};

pub fn get_hash(text: String) -> String {
    format!("{:x}", hash_prepare(text))
}

fn hash_prepare(text: String) -> impl LowerHex {
    let mut hasher = Sha512::new();
    hasher.update(text);
    hasher.finalize()
}

pub fn get_dir_path(file_path: &str) -> String {
    if let Some(pos) = file_path.rfind('/') {
        file_path[..pos].to_string()
    } else {
        "".to_string()
    }
}

pub fn get_file_name(file_path: &str) -> String {
    if let Some(pos) = file_path.rfind('/') {
        file_path[pos + 1..].to_string()
    } else {
        "".to_string()
    }
}