use dominator::{Dom, events, html};
use futures_signals::signal::{Mutable, Signal, SignalExt};

use shared::types::{BoxMailAttachments, MailBoxes, MessageRequest};
use shared::utils::box_type_index;

use crate::constants::{EMAIL_DATALIST, PROP_EDITABLE, PROP_HTML, PROP_NAME, PROP_PLACEHOLDER, PROP_TITLE, PROP_TYPE, PROP_VALUE, TAG_DIV, TAG_INPUT};
use crate::editor::editor_tools::{editor_preview_tools, editor_tools};
use crate::editor::state::{EDITOR, EditorState};
use crate::elements::attachment::{attachments_active, attachments_preview};
use crate::loader::message_update;
use crate::state::{CURRENT_BOX, USER};

fn css_class(label: &str) -> String {
    format!("app-editor__{label}")
}

pub fn editor_close() {
    EDITOR.set(None);
}

pub fn app_editor() -> impl Signal<Item=Option<Dom>> {
    EDITOR.signal_cloned().map(|state| match state {
        Some(state) => Some(editor(state)),
        None => None
    })
}

pub fn get_editor() -> Option<EditorState> {
    EDITOR.get_cloned()
}

pub fn open_note_editor(is_note: bool, content: String) {
    let content = if content.trim().is_empty() {
        "<p><br></p>".to_string()
    } else {
        content.clone()
    };
    EDITOR.set(Some(EditorState {
        is_note,
        content,
        editable: true,
        ..EditorState::default()
    }));
}

pub fn set_editor_attachments(attachments: BoxMailAttachments) {
    if let Some(editor) = EDITOR.get_cloned() {
        editor.attachments.set(Some(attachments.clone()));
    }
}

pub fn open_email_editor(idb: u64, mail_to: String, subject: String, content: String) {
    let signature = match USER.lock() {
        Ok(user) => user.signature.clone(),
        Err(_) => "".to_string()
    };
    message_update(MessageRequest {
        idb: idb.clone(),
        attachments: Some(BoxMailAttachments { key: "".to_string(), list: vec![] }),
        ..MessageRequest::default()
    });
    EDITOR.set(Some(EditorState {
        recipient: Some(mail_to),
        subject: Some(subject),
        content: format!("{content}<p><br></p>{signature}"),
        editable: true,
        ..EditorState::default()
    }));
}

pub fn open_message_preview(idb: &u64, attachments: &Option<BoxMailAttachments>, mail_from: String, mail_to: String, subject: String, content: String) {
    let with_unread = CURRENT_BOX.get() == MailBoxes::Inbox;
    EDITOR.set(Some(EditorState {
        idb: idb.clone(),
        with_unread,
        sender: Some(mail_from),
        recipient: Some(mail_to),
        subject: Some(subject),
        attachments: Mutable::new(attachments.clone()),
        content,
        ..EditorState::default()
    }));
    message_update(MessageRequest {
        idb: idb.clone(),
        unread: Some(false),
        box_current: Some(box_type_index(&CURRENT_BOX.get()) as i32),
        ..MessageRequest::default()
    });
}

pub fn editor_version() {
    if let Some(editor) = EDITOR.get_cloned() {
        editor.version.set_neq(editor.version.get() + 1);
    }
}

fn editor(state: EditorState) -> Dom {
    let mut top: Vec<Dom> = vec![];

    if state.editable {
        let is_note = state.is_note.clone();
        if !is_note {
            top.push(header_active(&state));
        }
        top.push(editor_tools(is_note));
        if !is_note {
            top.push(attachments_active(state.attachments.signal_cloned()));
        }
    } else {
        top.push(editor_preview_tools());
        top.push(header_preview(&state));
        if let Some(attachments) = state.attachments.get_cloned() {
            top.push(attachments_preview(&attachments));
        }
    }

    html!(TAG_DIV, {
        .class(css_class("back"))
        .child(html!(TAG_DIV, {
            .class(css_class("container"))
            .class_signal("saved", state.version.signal_cloned().map(|version|version%2==1))
            .event(animation_end)
            .children([
                html!(TAG_DIV, {
                    .children(top)
                }),
                html!(TAG_DIV, {
                    .class(css_class("content"))
                    .attr(PROP_EDITABLE, &state.editable.to_string())
                    .prop(PROP_HTML, state.content)
                })
            ])
        }))
    })
}

fn header_active(state: &EditorState) -> Dom {
    let recipient = state.recipient.clone().unwrap_or_default();
    let subject = state.subject.clone().unwrap_or_default();
    html!(TAG_DIV, {
        .children([
            html!(TAG_DIV, {
                .child(html!(TAG_INPUT, {
                    .class(css_class("input"))
                    .attr(PROP_TITLE, "получатель")
                    .attr(PROP_PLACEHOLDER, "получатель")
                    .attr(PROP_TYPE, "string")
                    .attr(PROP_NAME, "recipient")
                    .attr(PROP_VALUE, &recipient)
                    .attr("list", EMAIL_DATALIST)
                }))
            }),
            html!(TAG_DIV, {
                .child(html!(TAG_INPUT, {
                    .class(css_class("input"))
                    .attr(PROP_TITLE, "тема")
                    .attr(PROP_PLACEHOLDER, "тема")
                    .attr(PROP_TYPE, "string")
                    .attr(PROP_NAME, "subject")
                    .attr(PROP_VALUE, &subject)
                }))
            })
        ])
    })
}

fn animation_end(_: events::AnimationEnd) {
    editor_version();
}

fn header_preview(state: &EditorState) -> Dom {
    let sender = state.sender.clone().unwrap_or_default();
    let recipient = state.recipient.clone().unwrap_or_default();
    let subject = state.subject.clone().unwrap_or_default();
    html!(TAG_DIV, {
        .children([
            html!(TAG_DIV, {
                .child(html!("b", {.text("Кому: ")}))
                .text(&recipient)
            }),
            html!(TAG_DIV, {
                .child(html!("b", {.text("От кого: ")}))
                .text(&sender)
            }),
            html!(TAG_DIV, {
                .child(html!("b", {.text("Тема: ")}))
                .text(&subject)
            }),
        ])
    })
}
