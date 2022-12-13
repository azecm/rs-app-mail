use std::string::ToString;
use std::sync::{Arc, Mutex};

use futures_signals::signal::Mutable;
use futures_signals::signal_vec::MutableVec;
use once_cell::sync::Lazy;

use shared::types::MailBoxes;

use crate::types::{BoxState, EventItemStruct, NoteStruct, UserStruct};

pub static CURRENT_BOX: Lazy<Mutable<MailBoxes>> = Lazy::new(|| {
    Mutable::new(MailBoxes::Inbox)
});

pub static BOX_STATE: Lazy<Vec<BoxState>> = Lazy::new(|| {
    vec![BoxState::default(), BoxState::default(), BoxState::default(), BoxState::default(), BoxState::default()]
});

pub static NOTES: Lazy<MutableVec<NoteStruct>> = Lazy::new(|| {
    MutableVec::new()
});

pub static EVENTS: Lazy<Mutable<Vec<EventItemStruct>>> = Lazy::new(|| {
    Mutable::new(vec![])
});

pub static LOADING_NEXT: Lazy<Mutable<bool>> = Lazy::new(|| Mutable::new(false));

pub static NOTES_SELECTED: Lazy<Mutable<i32>> = Lazy::new(|| Mutable::new(0));

pub static USER_KEY: Lazy<Mutable<String>> = Lazy::new(|| Mutable::new("".to_string()));

pub static USER: Lazy<Arc<Mutex<UserStruct>>> = Lazy::new(|| Arc::new(Mutex::new(UserStruct::default())));
