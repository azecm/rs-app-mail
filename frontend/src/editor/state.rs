use futures_signals::signal::Mutable;
use once_cell::sync::Lazy;

use shared::types::BoxMailAttachments;

pub static EDITOR: Lazy<Mutable<Option<EditorState>>> = Lazy::new(|| {
    Mutable::new(None)
});

#[derive(Clone, Default, Debug)]
pub struct EditorState {
    pub content: String,
    pub editable: bool,
    pub subject: Option<String>,
    pub sender: Option<String>,
    pub recipient: Option<String>,
    pub is_note: bool,
    pub idb: u64,
    pub with_unread: bool,
    pub version: Mutable<usize>,
    pub attachments: Mutable<Option<BoxMailAttachments>>,
}

