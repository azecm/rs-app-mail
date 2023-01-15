use std::error::Error as StdError;
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use tokio_postgres::types::{FromSql, Type};

use shared::types::{BoxMailAttachmentItem, BoxMailAttachments};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DBNotes {
    pub idn: i32,
    pub idp: i32,
    pub position: i32,
    pub label: String,
    pub email: String,
    pub content: String,
    pub event: Option<DBNotesEvent>,
}

impl From<Row> for DBNotes {
    fn from(row: Row) -> Self {
        Self {
            idn: row.get("idn"),
            idp: row.get("idp"),
            position: row.get("position"),
            label: row.get("label"),
            email: row.get("email"),
            content: row.get("content"),
            event: row.get("event"),
        }
    }
}

// ===

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DBNotesEvent {
    pub date: String,
    pub delta: i32,
    pub period: i32,
}


impl<'a> FromSql<'a> for DBNotesEvent {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<DBNotesEvent, Box<(dyn StdError + Send + Sync + 'static)>> {
        match serde_json::from_slice::<DBNotesEvent>(&raw[1..]) {
            Ok(data) => Ok(data),
            Err(err) => {
                tracing::error!("from_sql DBNotesEvent {:?}", err);
                Ok(DBNotesEvent::default())
            }
        }
    }
    fn accepts(ty: &Type) -> bool {
        ty == &Type::JSONB
    }
}

// ===

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DBCount {
    pub count: i64,
}

impl From<Row> for DBCount {
    fn from(row: Row) -> Self {
        Self {
            count: row.get("count"),
        }
    }
}

// ===

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DBIdn {
    pub idn: i32,
}

impl From<Row> for DBIdn {
    fn from(row: Row) -> Self {
        Self {
            idn: row.get("idn"),
        }
    }
}

// ===

#[derive(Debug, Clone)]
pub struct DBNotesMini {
    pub idn: i32,
    pub idp: i32,
    pub position: i32,
}

impl From<Row> for DBNotesMini {
    fn from(row: Row) -> Self {
        Self {
            idn: row.get("idn"),
            idp: row.get("idp"),
            position: row.get("position"),
        }
    }
}

// ===

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct DBPageResponse {
    pub email_box: i32,
    pub page: usize,
    pub news: bool,
    pub data: Vec<DBBox>,
}

// ===

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DBBox {
    pub idb: i64,
    pub date: String,
    pub order: u64,
    pub unread: bool,
    pub sender: DBMailAddress,
    pub recipient: DBMailAddress,
    pub subject: String,
    pub content: String,
    pub attachments: Option<DBMailAttachments>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DBMailAddress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub address: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DBMailAttachments {
    pub key: String,
    pub list: Vec<DBMailAttachmentItem>,
}

impl From<&BoxMailAttachments> for DBMailAttachments {
    fn from(row: &BoxMailAttachments) -> Self {
        Self {
            key: row.key.clone(),
            list: row.list.iter().map(DBMailAttachmentItem::from).collect::<Vec<_>>(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DBMailAttachmentItem {
    pub id: usize,
    #[serde(rename = "fileName")]
    pub file_name: String,
    pub size: u64,
}

impl From<&BoxMailAttachmentItem> for DBMailAttachmentItem {
    fn from(row: &BoxMailAttachmentItem) -> Self {
        Self {
            id: row.id,
            size: row.size,
            file_name: row.file_name.clone(),
        }
    }
}

// ===

impl From<Row> for DBBox {
    fn from(row: Row) -> Self {
        let date: SystemTime = row.get("date");
        let datetime: DateTime<Utc> = date.into();
        let datetime = datetime.format("%d.%m.%Y %T").to_string();
        Self {
            idb: row.get("idb"),
            date: datetime,
            order: date.elapsed().unwrap_or_default().as_secs(),
            unread: row.get("unread"),
            sender: row.get("sender"),
            recipient: row.get("recipient"),
            subject: row.get("subject"),
            content: row.get("content"),
            attachments: row.get("attachments"),
        }
    }
}

impl<'a> FromSql<'a> for DBMailAddress {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<DBMailAddress, Box<(dyn StdError + Send + Sync + 'static)>> {
        match serde_json::from_slice::<DBMailAddress>(&raw[1..]) {
            Ok(data) => Ok(data),
            Err(err) => {
                tracing::error!("from_sql DBMailAddress {:?}", err);
                Ok(DBMailAddress::default())
            }
        }
    }
    fn accepts(ty: &Type) -> bool {
        ty == &Type::JSONB
    }
}

impl<'a> FromSql<'a> for DBMailAttachments {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<DBMailAttachments, Box<(dyn StdError + Send + Sync + 'static)>> {
        match serde_json::from_slice::<DBMailAttachments>(&raw[1..]) {
            Ok(data) => Ok(data),
            Err(err) => {
                tracing::error!("from_sql DBMailAttachments {:?}", err);
                Ok(DBMailAttachments::default())
            }
        }
    }
    fn accepts(ty: &Type) -> bool {
        ty == &Type::JSONB
    }
}

impl<'a> FromSql<'a> for DBMailAttachmentItem {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<DBMailAttachmentItem, Box<(dyn StdError + Send + Sync + 'static)>> {
        match serde_json::from_slice::<DBMailAttachmentItem>(&raw[1..]) {
            Ok(data) => Ok(data),
            Err(err) => {
                tracing::error!("from_sql DBMailAttachmentItem {:?}", err);
                Ok(DBMailAttachmentItem::default())
            }
        }
    }
    fn accepts(ty: &Type) -> bool {
        ty == &Type::JSONB
    }
}