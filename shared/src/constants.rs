
pub const MAIL_ATTACH_EXT: &str = ".tmp";
pub const BY_PAGE: i64 = 30;

pub const ROOT_API: &str = "api";
pub const API_NOTES: &str = "notes";
pub const API_FILE: &str = "file";
pub const API_FILES: &str = "files";
pub const API_LOGIN: &str = "login";
pub const API_EVENT: &str = "event";

pub const CHANNEL_NOTES: &str = "notes";
pub const CHANNEL_BOXES: &str = "boxes";
pub const CHANNEL_MESSAGES: &str = "msg-list";
pub const CHANNEL_MESSAGE: &str = "msg-update";
pub const CHANNEL_INIT: &str = "init";
pub const CHANNEL_USER_KEY: &str = "user";

pub const HEADER_USER_KEY: &str = "User-Key";

#[cfg(target_os = "macos")]
pub const TEST_USER_ID: i32 = 1;
#[cfg(not(target_os = "macos"))]
pub const TEST_USER_ID: i32 = 0;
