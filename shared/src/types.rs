use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MailBoxes {
    Inbox,
    Ready,
    Sent,
    Trash,
    Notes,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct NotesChannel {
    pub idn: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
    //#[serde(skip_serializing_if = "Option::is_none")]
    //pub from: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idp: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<NotesEvent>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct NotesEvent {
    pub date: String,
    pub delta: i32,
    pub period: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MessagesRequest {
    pub email_box: i32,
    pub page: usize
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MessageRequest {
    pub idb: u64,
    pub send: Option<bool>,
    pub unread: Option<bool>,
    pub box_current: Option<i32>,
    pub box_target: Option<i32>,
    pub notes_idp: Option<i32>,
    pub attachments: Option<BoxMailAttachments>,
    pub remove_id: Option<usize>,
    pub content: Option<String>,
    pub subject: Option<String>,
    pub recipient: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct BoxMailAttachments {
    pub key: String,
    pub list: Vec<BoxMailAttachmentItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct BoxMailAttachmentItem {
    pub id: usize,
    #[serde(rename = "fileName")]
    pub file_name: String,
    pub size: u64,
}