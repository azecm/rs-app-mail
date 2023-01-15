use dominator::{Dom, events, html};
use futures_signals::map_ref;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use futures_signals::signal_vec::{MutableVec, SignalVecExt};
use once_cell::sync::Lazy;

use shared::types::{MailBoxes, MessageRequest, MessagesRequest};
use shared::utils::box_type_index;

use crate::constants::{PROP_HTML, PROP_ROLE, PROP_ROLE_BUTTON, PROP_TITLE, TAG_DIV, TAG_SPAN};
use crate::dialog::dialogs::Dialog;
use crate::editor::app_editor::{editor_close, open_message_preview, set_editor_attachments};
use crate::elements::attachment::attachments_preview;
use crate::elements::icons::{icon_envelope, icon_envelope_open, icon_inbox, icon_note, icon_read, icon_trash};
use crate::loader::{message_update, messages_load};
use crate::state::{BOX_STATE, CURRENT_BOX, LOADING_NEXT, NOTES};
use crate::types::{BoxMailAddress, BoxMessage, MessagesResponse};
use crate::utils::{attr_data, from_dataset, view_email};

static BOXES: Lazy<Vec<MutableVec<BoxMessage>>> = Lazy::new(|| {
    vec![
        MutableVec::new(),
        MutableVec::new(),
        MutableVec::new(),
        MutableVec::new(),
    ]
});

fn css_class(label: &str) -> String {
    format!("app_message__{label}")
}

pub fn messages_channel(data: MessagesResponse) {
    if !data.data.is_empty() {
        if data.news {
            // подгружаем новые
            BOXES[data.email_box].lock_mut().insert_cloned(0, BoxMessage::from(data.data[0].clone()));
        } else {
            // подгружаем страницу
            log::info!("{} initialized", data.email_box);
            BOX_STATE[data.email_box].initialized.set(true);
            BOX_STATE[data.email_box].page.set(data.page);
            let keys = BOXES[data.email_box].lock_ref().iter().map(|row| row.idb).collect::<Vec<_>>();
            BOXES[data.email_box].lock_mut().extend(data.data.iter().filter(|row| !keys.contains(&row.idb)).map(|row| BoxMessage::from(row.clone())).collect::<Vec<_>>());
        }
    } else {
        BOX_STATE[data.email_box].fully_loaded.set(true);
    }
    LOADING_NEXT.set(false);
}

pub fn message_channel(data: MessageRequest) {
    if let Some(send) = data.send {
        if send {
            editor_close();
        } else {
            Dialog::alert("Ошибка при отправке...");
        }
    } else if let Some(box_current) = data.box_current {
        let box_current = box_current as usize;
        if let Some(unread) = data.unread {
            if let Some(message) = BOXES[box_current].lock_ref().iter().find(|row| row.idb == data.idb) {
                message.unread.set(unread);
            }
        }
        if let Some(box_target) = data.box_target {
            let box_target = box_target as usize;
            let pos = BOXES[box_current].lock_ref().iter().position(|row| row.idb == data.idb);
            if let Some(pos) = pos {
                let message = BOXES[box_current].lock_mut().remove(pos);
                BOXES[box_target].lock_mut().insert_cloned(0, message);
            }
        }
    } else if let Some(attachments) = data.attachments {
        set_editor_attachments(attachments);
    }
}

pub fn box_view(mbox: MailBoxes) -> Dom {
    let mbox_2 = mbox;
    html!(TAG_DIV, {
        .visible_signal(CURRENT_BOX.signal().map(move|b| box_visible(b==mbox, &mbox)))
        .children_signal_vec(BOXES[box_type_index(&mbox)].signal_vec_cloned().map(move |row| message(&mbox_2, row)))
    })
}

const ATTR_DATA_KEY: &str = "key";
const DATA_KEY_NOTE: &str = "note";
const DATA_KEY_BOX: &str = "box";

fn message(mbox: &MailBoxes, row: BoxMessage) -> Dom {
    let count = match &row.attachments {
        Some(att) => format!("+[{}]", att.list.len()),
        None => "".to_string()
    };

    let is_inbox = mbox == &MailBoxes::Inbox;
    let idb = row.idb;
    let mbox_over = *mbox;

    let idb_selected = row.idb;

    html!(TAG_DIV, {
        .class(css_class("container"))
        .class_signal("unread", row.unread.signal().map(move|flag|flag && is_inbox))
        .class_signal("selected", BOX_STATE[box_type_index(mbox)].selected.signal().map(move|val|val==idb_selected))
        .event(move|_:events::MouseEnter|handle_over(&mbox_over, &idb))
        .event(move|e:events::Click|handle_click(from_dataset(e.target(), ATTR_DATA_KEY), &mbox_over, &idb))
        .children([
            html!(TAG_DIV, {
                .class(css_class("col-1"))
                .child(email_icon(mbox, &row))
            }),
            html!(TAG_DIV, {
                .class(css_class("col-2"))
                .children([
                    html!(TAG_DIV, {
                        .class(css_class("note-block"))
                        .attr(PROP_TITLE, "сохранить в записках")
                        .attr(PROP_ROLE, PROP_ROLE_BUTTON)
                        .attr(&attr_data(ATTR_DATA_KEY), DATA_KEY_NOTE)
                        .child(icon_note())
                    }),
                    html!(TAG_DIV, {
                        .class(css_class("date-block"))
                        .child(html!(TAG_DIV, {
                            .class(css_class("date-elem"))
                            .text(&row.date)
                        }))
                    }),
                    email_view(mbox, &row),
                    html!(TAG_DIV, {
                        .text(&row.subject)
                    }),
                    html!(TAG_DIV, {
                        .text(&count)
                    }),
                ])
            })
        ])
    })
}

fn email_view(mbox: &MailBoxes, row: &BoxMessage) -> Dom {
    if mbox == &MailBoxes::Sent {
        email_elem(&row.recipient)
    } else {
        email_elem(&row.sender)
    }
}

fn email_elem(data: &BoxMailAddress) -> Dom {
    if let Some(name) = &data.name {
        return html!(TAG_DIV, {
            .children([
                html!(TAG_SPAN, {
                    .class(css_class("email-name"))
                    .text(name)
                }),
                html!(TAG_SPAN, {
                    .class(css_class("email-address"))
                    .text(&data.address)
                }),
            ])
        });
    }

    html!(TAG_DIV, {
        .child(html!(TAG_SPAN, {
            .class(css_class("email-address"))
            .text(&data.address)
        }))
    })
}

fn box_visible(selected: bool, mb_type: &MailBoxes) -> bool {
    if selected {
        let box_index = box_type_index(mb_type);
        if !BOX_STATE[box_index].initialized.get() {
            messages_load(MessagesRequest { page: 0, email_box: box_type_index(mb_type) as i32 });
        }
    }
    selected
}

enum IconState {
    InboxDefault,
    InboxOpen,
    AnyDefault,
    AnyOver,
    TrashOver,
}

fn email_icon(mbox: &MailBoxes, row: &BoxMessage) -> Dom {
    let over_state = Mutable::new(false);
    let over_state_enter = over_state.clone();
    let over_state_leave = over_state.clone();

    let hover_class = if mbox == &MailBoxes::Trash { "to-inbox" } else { "to-trash" };
    let title = if mbox == &MailBoxes::Trash { "во входящие" } else { "в корзину" };

    let is_trash = mbox == &MailBoxes::Trash;
    let is_inbox = mbox == &MailBoxes::Inbox;

    let inbox_signal = Mutable::new(is_inbox);
    let trash_signal = Mutable::new(is_trash);

    let icon_signal = || {
        map_ref! {
            let is_over = over_state.signal(),
            let is_inbox =  inbox_signal.signal(),
            let is_trash = trash_signal.signal(),
            let unread = row.unread.signal() =>
            if *is_inbox {
                if *is_over {IconState::AnyOver} else if *unread { IconState::InboxDefault } else {  IconState::InboxOpen }
            }
            else if *is_over {
                if *is_trash { IconState::TrashOver } else { IconState::AnyOver }
            }
            else { IconState::AnyDefault }
        }
    };

    html!(TAG_DIV, {
        .class(css_class("email-icon"))
        .class(hover_class)
        .attr(PROP_TITLE, title)
        .attr(PROP_ROLE, PROP_ROLE_BUTTON)
        .attr(&attr_data(ATTR_DATA_KEY), DATA_KEY_BOX)
        .child_signal(icon_signal().map(get_icon))
        .event(move|_:events::MouseEnter|{over_state_enter.set(true)})
        .event(move|_:events::MouseLeave|{over_state_leave.set(false)})
    })
}

fn get_icon(state: IconState) -> Option<Dom> {
    Some(match state {
        IconState::InboxDefault => icon_envelope(),
        IconState::InboxOpen => icon_envelope_open(),
        IconState::AnyDefault => icon_read(),
        IconState::AnyOver => icon_trash(),
        IconState::TrashOver => icon_inbox(),
    })
}

fn handle_over(mbox: &MailBoxes, idb: &u64) {
    BOX_STATE[box_type_index(mbox)].selected.set(*idb);
}

fn handle_click(data_key: String, mbox: &MailBoxes, idb: &u64) {
    match data_key.as_str() {
        DATA_KEY_BOX => {
            let box_target = if mbox == &MailBoxes::Trash {
                box_type_index(&MailBoxes::Inbox) as i32
            } else {
                box_type_index(&MailBoxes::Trash) as i32
            };
            message_update(MessageRequest {
                idb: *idb,
                box_current: Some(box_type_index(mbox) as i32),
                box_target: Some(box_target),
                ..MessageRequest::default()
            });
            return;
        }
        DATA_KEY_NOTE => {
            if NOTES.lock_ref().len() > 0 {
                let idp = NOTES.lock_ref()[0].idn;
                let label = NOTES.lock_ref()[0].label.get_cloned();
                message_update(MessageRequest {
                    idb: *idb,
                    notes_idp: Some(idp),
                    ..MessageRequest::default()
                });
                Dialog::alert(&format!("Добавлено в группу «{label}»."));
            } else {
                Dialog::alert("Создайте группу в заметках.");
            }
            return;
        }
        _ => {}
    }

    let idb = *idb;
    if let Some(message) = BOXES[box_type_index(mbox)].lock_ref().iter().find(|row| row.idb == idb) {
        let sender = if let Some(label) = &message.sender.name { label.clone() } else { "".to_string() };
        let sender = view_email(&sender, &message.sender.address);
        let recipient = if let Some(label) = &message.recipient.name { label.clone() } else { "".to_string() };
        let recipient = view_email(&recipient, &message.recipient.address);
        open_message_preview(&idb, &message.attachments, sender, recipient, message.subject.clone(), message.content.clone());
    }
}

pub fn message_content() -> Dom {
    html!(TAG_DIV, {
        .visible_signal(CURRENT_BOX.signal().map(|b|b!=MailBoxes::Notes))
        .child_signal(attachments_signal())
        .child(
            html!(TAG_DIV, {
                .class(css_class("content"))
                .prop_signal(PROP_HTML,
                    common_signal().map(|item:BoxMessage|item.content)
                )
            })
        )
    })
}

fn attachments_signal() -> impl Signal<Item=Option<Dom>> {
    common_signal().map(|item: BoxMessage| item.attachments.map(|attachments| attachments_preview(&attachments)))
}


fn common_signal() -> impl Signal<Item=BoxMessage> {
    CURRENT_BOX.signal().map(|mb| {
        BOX_STATE[box_type_index(&mb)].selected.signal()
            .map(move |idb| if mb != MailBoxes::Notes && idb > 0 {
                match BOXES[box_type_index(&mb)].lock_ref().iter().find(|row| row.idb == idb) {
                    Some(item) => item.clone(),
                    None => BoxMessage::default()
                }
            } else {
                BoxMessage::default()
            })
    }).flatten()
}
