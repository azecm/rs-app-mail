
pub const MAIL_ATTACH_EXT: &str = ".tmp";
pub const BY_PAGE: i64 = 30;

pub static ROOT_API: &'static str = "api";
pub static API_NOTES: &'static str = "notes";
pub static API_FILE: &'static str = "file";
pub static API_FILES: &'static str = "files";
pub static API_LOGIN: &'static str = "login";
pub static API_EVENT: &'static str = "event";

pub static CHANNEL_NOTES: &'static str = "notes";
pub static CHANNEL_BOXES: &'static str = "boxes";
pub static CHANNEL_MESSAGES: &'static str = "msg-list";
pub static CHANNEL_MESSAGE: &'static str = "msg-update";
pub static CHANNEL_INIT: &'static str = "init";
pub static CHANNEL_USER_KEY: &'static str = "user";

pub static HEADER_USER_KEY: &'static str = "User-Key";

#[cfg(target_os = "macos")]
pub const TEST_USER_ID: i32 = 1;
#[cfg(not(target_os = "macos"))]
pub const TEST_USER_ID: i32 = 0;
