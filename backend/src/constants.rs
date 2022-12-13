use std::fs;

use shared::constants::MAIL_ATTACH_EXT;

#[cfg(target_os = "macos")]
const MAIL_ROOT_PATH: &str = "/Users/mac-user/Documents/_cache/mail";
#[cfg(not(target_os = "macos"))]
const MAIL_ROOT_PATH: &str = "/usr/local/www/cache/mail/";

#[cfg(target_os = "macos")]
pub const MAIL_SOURCE_PATH: &str = "/Users/mac-user/Documents/_cache/mail/virtual";
#[cfg(not(target_os = "macos"))]
pub const MAIL_SOURCE_PATH: &str = "/var/mail/virtual";

const DIR_TEMP: &'static str = "temp";
const DIR_SOURCE: &'static str = "source";
const DIR_ATTACHMENT: &'static str = "attachment";


pub fn path_to_attachment(email: &str, key: &str, ind: &usize) -> String {
    format!("{MAIL_ROOT_PATH}/{DIR_ATTACHMENT}/{email}/{key}-{ind}{MAIL_ATTACH_EXT}")
}

pub fn path_to_attachment_with_email_and_key(email: &str, key: &str) -> String {
    format!("{MAIL_ROOT_PATH}/{DIR_ATTACHMENT}/{email}/{key}{MAIL_ATTACH_EXT}")
}

pub fn path_to_temp_upload(key: &str) -> String {
    format!("{MAIL_ROOT_PATH}/{DIR_TEMP}/{key}")
}

pub fn path_to_temp_with_ind(key: &str, ind: &usize) -> String {
    format!("{MAIL_ROOT_PATH}/{DIR_TEMP}/{key}-{ind}{MAIL_ATTACH_EXT}")
}

pub fn path_to_temp(key: &str) -> String {
    format!("{MAIL_ROOT_PATH}/{DIR_TEMP}/{key}{MAIL_ATTACH_EXT}")
}

pub fn path_to_saved(email: &str, file_name: &str) -> String {
    format!("{MAIL_ROOT_PATH}/{DIR_SOURCE}/{email}/{file_name}")
}

pub fn test_dirs() {
    let path_to_dir = &format!("{MAIL_ROOT_PATH}/{DIR_TEMP}");
    if let Err(err) = fs::create_dir_all(path_to_dir) {
        tracing::error!("test_dirs: {:?}", err);
    }
}