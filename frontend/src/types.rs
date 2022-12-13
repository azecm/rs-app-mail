use futures_signals::signal::Mutable;
use serde::{Deserialize, Serialize};

use shared::types::{BoxMailAttachments, NotesEvent};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct NotesSource {
    pub idn: i32,
    pub idp: i32,
    pub position: i32,
    pub label: String,
    pub email: String,
    pub content: String,
    pub event: Option<NotesEvent>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct UserSource {
    pub prefix: String,
    pub signature: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct InitialStruct {
    pub notes: Vec<NotesSource>,
    pub user: UserSource,
}

pub type UserKey = String;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct UserStruct {
    pub label: String,
    pub email: String,
    pub signature: String,
}

#[derive(Debug, Clone, Default)]
pub struct EventItemStruct {
    pub idn: i32,
    pub idp: i32,
    pub group: String,
    pub label: String,
    pub order: i64,
    pub date: String,
}

#[derive(Debug, Clone, Default)]
pub struct NoteStruct {
    pub idn: i32,
    pub idp: Mutable<i32>,
    pub position: Mutable<i32>,
    pub label: Mutable<String>,
    pub email: Mutable<String>,
    pub content: Mutable<String>,
    pub event: Mutable<Option<NotesEvent>>,
}

impl AsRef<NoteStruct> for NoteStruct {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl From<NotesSource> for NoteStruct {
    fn from(src: NotesSource) -> Self {
        Self {
            idn: src.idn,
            idp: Mutable::new(src.idp),
            position: Mutable::new(src.position),
            label: Mutable::new(src.label),
            email: Mutable::new(src.email),
            event: Mutable::new(src.event),
            content: Mutable::new(src.content),
        }
    }
}

impl PartialEq<NoteStruct> for NoteStruct {
    fn eq(&self, other: &Self) -> bool {
        self.idn == other.idn
    }
}

// ===

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MessagesResponse {
    pub email_box: usize,
    pub page: usize,
    pub news: bool,
    pub data: Vec<BoxMessageSource>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BoxMessageSource {
    pub idb: u64,
    pub date: String,
    pub order: u64,
    pub unread: bool,
    pub sender: BoxMailAddress,
    pub recipient: BoxMailAddress,
    pub subject: String,
    pub content: String,
    pub attachments: Option<BoxMailAttachments>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct BoxMessage {
    pub idb: u64,
    pub date: String,
    pub order: u64,
    pub unread: Mutable<bool>,
    pub sender: BoxMailAddress,
    pub recipient: BoxMailAddress,
    pub subject: String,
    pub content: String,
    pub attachments: Option<BoxMailAttachments>,
}

impl From<BoxMessageSource> for BoxMessage {
    fn from(src: BoxMessageSource) -> Self {
        Self {
            idb: src.idb,
            date: src.date,
            order: src.order,
            unread: Mutable::new(src.unread),
            sender: src.sender,
            recipient: src.recipient,
            subject: src.subject,
            content: src.content,
            attachments: src.attachments,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct BoxMailAddress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub address: String,
}


// ===

#[derive(Debug, Clone, Default)]
pub struct BoxState {
    pub scroll_text: Mutable<i32>,
    pub scroll_list: Mutable<i32>,
    pub selected: Mutable<u64>,
    pub page: Mutable<usize>,
    pub initialized: Mutable<bool>,
    pub fully_loaded: Mutable<bool>,
}