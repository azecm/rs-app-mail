use std::fmt::Debug;

use futures_signals::signal::Mutable;
use serde::Serialize;

use shared::constants::{API_NOTES, CHANNEL_MESSAGE, CHANNEL_MESSAGES};
use shared::types::{MessageRequest, MessagesRequest, NotesChannel};

use crate::connect_fetch::connect_json_send;
use crate::editor::app_editor::editor_version;
use crate::elements::app_login::get_user_box;
use crate::notes::notes_events::events_reload;
use crate::state::{NOTES, USER, USER_KEY};
use crate::types::{InitialStruct, NoteStruct, UserKey};

pub fn init_channel(data: InitialStruct) {
    if let Ok(mut user) = USER.lock() {
        user.signature = data.user.signature.clone();
        user.label = data.user.prefix.clone();
        user.email = get_user_box();
    }
    if NOTES.lock_mut().len() == 0 {
        NOTES.lock_mut().extend(data.notes.iter().map(|item| NoteStruct::from(item.clone())));
        events_reload();
    }
}

// ===

pub fn user_channel(data: UserKey) {
    USER_KEY.set_neq(data);
}

// ===

pub fn messages_load(data: MessagesRequest) {
    connect_json_send(CHANNEL_MESSAGES, data);
}

pub fn message_update(data: MessageRequest) {
    connect_json_send(CHANNEL_MESSAGE, data);
}

// ===

pub fn notes_update<T: Serialize + Debug>(data: T) {
    connect_json_send(API_NOTES, data);
}

pub fn notes_channel(data: NotesChannel) {
    if data.remove.unwrap_or(false) {
        let ind_current = NOTES.lock_ref().iter().position(|row| row.idn == data.idn);
        if let Some(ind_current) = ind_current {
            NOTES.lock_mut().remove(ind_current);
        }
    } else if data.insert.unwrap_or(false) {
        let position = data.position.unwrap_or_default();
        let idp = data.idp.unwrap_or_default();
        let item = NoteStruct {
            idn: data.idn,
            idp: Mutable::new(idp.clone()),
            position: Mutable::new(position.clone()),
            label: Mutable::new(data.label.unwrap_or_default()),
            email: Mutable::new(data.email.unwrap_or_default()),
            content: Mutable::new(data.content.unwrap_or_default()),
            ..NoteStruct::default()
        };
        let ind_first = NOTES.lock_ref().iter().position(|row| row.idp.get() == idp).unwrap_or_default();
        NOTES.lock_mut().insert_cloned(ind_first + position as usize, item);
    } else {
        if let Some(item) = NOTES.lock_ref().iter().find(|row| row.idn == data.idn) {
            if let Some(position) = data.position {
                item.position.set_neq(position);
            }
            if let Some(label) = data.label {
                item.label.set_neq(label);
            }
            if let Some(email) = data.email {
                item.email.set_neq(email);
            }
            if let Some(content) = data.content {
                item.content.set_neq(content);
                editor_version();
            }
            if let Some(event) = data.event {
                item.event.set(if event.date.is_empty() { None } else { Some(event) });
            }
        }

        if let Some(idp) = data.idp {
            let ind_current = NOTES.lock_ref().iter().position(|row| row.idn == data.idn);
            if let Some(ind_current) = ind_current {
                let item = NOTES.lock_mut().remove(ind_current);
                item.idp.set_neq(idp);

                let ind_next = NOTES.lock_ref().iter().rposition(|row| row.idp.get() == idp);
                match ind_next {
                    Some(next) => {
                        NOTES.lock_mut().insert_cloned(next + 1, item.clone());
                    }
                    None => {
                        NOTES.lock_mut().push_cloned(item.clone());
                    }
                };
            }
        }

        if let Some(to) = data.to {
            let position = match NOTES.lock_ref().iter().find(|row| row.idn == data.idn) {
                Some(item) => item.position.get_cloned(),
                None => 0
            };
            if position > 0 {
                let ind = NOTES.lock_ref().iter().position(|row| row.idn == data.idn).unwrap_or(usize::MAX);
                if ind != usize::MAX {
                    let item = NOTES.lock_mut().remove(ind);
                    item.position.set(to);
                    let next = (ind as i32 + (to - position)) as usize;

                    NOTES.lock_mut().insert_cloned(next, item.clone());
                }
            }
        }
    }
    events_reload();
}
