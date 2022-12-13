use dominator::{Dom, events, html};
use futures_signals::signal::{Mutable, SignalExt};
use futures_signals::signal_vec::SignalVecExt;
use gloo_timers::callback::Timeout;
use once_cell::sync::Lazy;
use wasm_bindgen_futures::spawn_local;

use shared::types::{MailBoxes, NotesChannel};

use crate::constants::{PROP_NAME, PROP_PLACEHOLDER, PROP_SELECTED, PROP_TITLE, PROP_TYPE, PROP_VALUE, TAG_BUTTON, TAG_DIV, TAG_INPUT, TAG_OPTION, TAG_SPAN};
use crate::dialog::dialogs::Dialog;
use crate::loader::notes_update;
use crate::state::{CURRENT_BOX, NOTES, NOTES_SELECTED};
use crate::types::NoteStruct;
use crate::utils::{attr_data, from_dataset, get_input_value};

const INPUT_NAME_LABEL: &'static str = "label";
const INPUT_NAME_EMAIL: &'static str = "email";
const INPUT_NAME_POSITION: &'static str = "position";
const INPUT_NAME_IDP: &'static str = "idp";
const ATTR_ID: &'static str = "id";

static CURRENT_ID: Lazy<Mutable<i32>> = Lazy::new(|| Mutable::new(0));
thread_local! {
    static LONG_CLICK_TIMER: Lazy<Mutable<Option<Timeout>>> = Lazy::new(||Mutable::new(None));
}

fn css_class(label: &str) -> String {
    format!("app-notes__{label}")
}

fn get_group_len(idp: i32) -> usize {
    NOTES.lock_ref().iter().filter(|row| row.idp.get() == idp).collect::<Vec<_>>().len()
}

pub fn app_notes() -> Dom {
    html!(TAG_DIV,{
        .class(css_class("container"))
        .visible_signal(CURRENT_BOX.signal().map(|b|b==MailBoxes::Notes))
        .children([
            html!(TAG_DIV, {
                .class(css_class("top"))
                .children([
                    html!(TAG_BUTTON, {
                        .class(css_class("button"))
                        .text("создать группу")
                        .event(create_group)
                    }),
                    html!(TAG_BUTTON, {
                        .class(css_class("button"))
                        .text("создать заметку")
                        .event(create_item)
                    }),
                    html!("small", {
                        .text("всего заметок: ")
                        .text_signal(NOTES.signal_vec_cloned().len().map(|v|v.to_string()))
                    })
                ])
            }),
            notes_list()
        ])
    })
}

// ===

fn create_group(_: events::Click) {
    Dialog::form("Новая группа", create_group_init, create_group_result, || {});
}

fn create_group_init() -> Dom {
    let max = get_group_len(0) + 1;
    dlg_group_view("", &max.clone(), &max.clone())
}

fn create_group_result() {
    let label = get_input_value(INPUT_NAME_LABEL).trim().to_string();
    let position = get_input_value(INPUT_NAME_POSITION).parse::<i32>().unwrap_or_default();
    if position > 0 && !label.is_empty() {
        notes_update(NotesChannel {
            idn: 0,
            insert: Some(true),
            position: Some(position),
            label: Some(label),
            ..NotesChannel::default()
        });
    }
}

// ===

fn create_item(_: events::Click) {
    Dialog::form("Новый элемент", create_item_init, create_item_result, || {});
}

fn create_item_init() -> Dom {
    dlg_item_view("", "", &0, &0, &0)
}

fn create_item_result() {
    let label = get_input_value(INPUT_NAME_LABEL).trim().to_string();
    let email = get_input_value(INPUT_NAME_EMAIL).trim().to_string();
    let idp = get_input_value(INPUT_NAME_IDP).parse::<i32>().unwrap_or_default();

    if idp > 0 && !label.is_empty() {
        notes_update(NotesChannel {
            idn: 0,
            insert: Some(true),
            label: Some(label),
            email: if email.is_empty() { None } else { Some(email) },
            idp: Some(idp),
            ..NotesChannel::default()
        });
    }
}

// ===

fn edit_group(event: events::MouseDown) {
    let id = from_dataset(event.target(), ATTR_ID).parse::<i32>().unwrap_or_default();
    if id > 0 {
        CURRENT_ID.set_neq(id);
        Dialog::form("Свойства группы", edit_group_init, edit_group_result, || {});
    }
}

fn edit_group_init() -> Dom {
    let idn = CURRENT_ID.get();
    let item = match NOTES.lock_ref().iter().find(|row| row.idn == idn) {
        Some(item) => item.clone(),
        None => NoteStruct::default()
    };
    let max = get_group_len(0);
    dlg_group_view(&item.label.get_cloned(), &(item.position.get() as usize), &max)
}

fn edit_group_result() {
    let idn = CURRENT_ID.get() as i32;
    let label = get_input_value(INPUT_NAME_LABEL).trim().to_string();
    let position = get_input_value(INPUT_NAME_POSITION).parse::<i32>().unwrap_or_default();

    if label.is_empty() {
        let mut flag_alert = false;
        match NOTES.lock_ref().iter().find(|row| row.idp.get() == idn) {
            Some(item) => {
                let idp = item.idp.get();
                flag_alert = get_group_len(idp) > 0;
            }
            None => {}
        };
        if flag_alert {
            spawn_local(async {
                Dialog::alert("Удалить можно только пустую группу.");
            });
        } else {
            before_delete("Удалить группу?");
        }
    } else {
        if let Some(item) = NOTES.lock_ref().iter().find(|row| row.idn == idn) {
            let item = item.clone();
            let label = match item.label.get_cloned() == label {
                false => Some(label),
                true => None
            };
            let position = match item.position.get() == position {
                false => Some(position),
                true => None
            };

            if label.is_some() || position.is_some() {
                notes_update(NotesChannel {
                    idn,
                    label,
                    position,
                    ..NotesChannel::default()
                });
            }
        }
    }
}

// ===

fn edit_item(event: events::MouseDown) {
    let id = from_dataset(event.target(), ATTR_ID).parse::<i32>().unwrap_or_default();
    if id > 0 {
        CURRENT_ID.set_neq(id);
        Dialog::form("Свойства элементы", edit_item_init, edit_item_result, || {});
    }
}

fn edit_item_init() -> Dom {
    let idn = CURRENT_ID.get();
    let item = match NOTES.lock_ref().iter().find(|row| row.idn == idn) {
        Some(item) => item.clone(),
        None => NoteStruct::default()
    };
    let max = &(get_group_len(item.idp.get()));
    let label = &item.label.get_cloned();
    let email = &item.email.get_cloned();
    let position = &(item.position.get() as usize);
    let idp = &(item.idp.get() as usize);
    dlg_item_view(label, email, position, max, idp)
}

fn edit_item_result() {
    let idn = CURRENT_ID.get();
    let label_src = get_input_value(INPUT_NAME_LABEL).trim().to_string();
    let email_src = get_input_value(INPUT_NAME_EMAIL).trim().to_string();
    let position = get_input_value(INPUT_NAME_POSITION).parse::<i32>().unwrap_or_default();
    let idp = get_input_value(INPUT_NAME_IDP).parse::<i32>().unwrap_or_default();

    if label_src.is_empty() && email_src.is_empty() {
        before_delete("Удалить элемент?");
        return;
    }

    let item = match NOTES.lock_ref().iter().find(|row| row.idn == idn) {
        Some(item) => item.clone(),
        None => NoteStruct::default()
    };
    if item.idn == 0 { return; }

    let label = if label_src == item.label.get_cloned() { None } else { Some(label_src) };
    let email = if email_src == item.email.get_cloned() { None } else { Some(email_src) };
    let idp = if idp == item.idp.get() { None } else { Some(idp) };
    let position = if position == item.position.get() || idp.is_some() { None } else { Some(position) };

    if label.is_some() || email.is_some() || idp.is_some() || position.is_some() {
        notes_update(NotesChannel {
            idn,
            label,
            email,
            idp,
            position,
            ..NotesChannel::default()
        });
    }
}

fn before_delete(title: &str) {
    let confirm = || {
        notes_update(NotesChannel {
            idn: CURRENT_ID.get(),
            remove: Some(true),
            ..NotesChannel::default()
        });
    };
    let title = title.to_string().clone();
    spawn_local(async move {
        Dialog::confirm(&title, confirm, || {});
    });
}

// ===

fn notes_list() -> Dom {
    html!(TAG_DIV, {
        .class(css_class("list"))
        .children_signal_vec(NOTES.signal_vec_cloned().filter_map(group))
    })
}

fn group(item: NoteStruct) -> Option<Dom> {
    if item.idp.get() > 0 {
        return None;
    }
    Some(html!("details", {
        .children([
            html!("summary", {
                .child(html!(TAG_SPAN, {
                    .class(css_class("item"))
                    .text_signal(item.label.signal_cloned())
                    .attr(&attr_data(ATTR_ID), &item.idn.to_string())
                    .event(long_click_init_group)
                    .event(long_click_up)
                    .event(long_click_move)
                }))
            }),
            html!("ul", {
                .class(css_class("sub-list"))
                .children_signal_vec(NOTES.signal_vec_cloned().filter_map(move|sub_item|group_items(sub_item, &item.idn)))
            })
        ])
    }))
}

fn group_items(item: NoteStruct, idp: &i32) -> Option<Dom> {
    if &item.idp.get() != idp {
        return None;
    }
    let idn_1 = item.idn.clone();
    let idn_2 = item.idn.clone();
    Some(html!("li", {
        .child(html!(TAG_SPAN, {
            .class(css_class("item"))
            .class_signal(css_class("item-selected"), NOTES_SELECTED.signal().map(move|current|current==idn_1))
            .text_signal(item.label.signal_cloned())
            .child(html!("i",{
                .text_signal(item.email.signal_cloned())
                .attr(&attr_data(ATTR_ID), &item.idn.to_string())
            }))
            .attr(&attr_data(ATTR_ID), &item.idn.to_string())
            .event(long_click_init_item)
            .event(long_click_up)
            .event(long_click_move)
            .event(move|_: events::Click|{
                NOTES_SELECTED.set_neq(idn_2);
            })
        }))
    }))
}

// ===

fn long_click_up(_: events::MouseUp) {
    long_click_clear();
}

fn long_click_move(_: events::MouseMove) {
    long_click_clear();
}

fn long_click_clear() {
    LONG_CLICK_TIMER.with(|timer| {
        match timer.lock_mut().take() {
            Some(timeout) => {
                timeout.cancel();
            }
            None => {}
        }
    });
}

fn long_click_init_group(event: events::MouseDown) {
    long_click_clear();
    let timeout = Timeout::new(300, || edit_group(event));
    LONG_CLICK_TIMER.with(|timer| {
        timer.set(Some(timeout));
    });
}

fn long_click_init_item(event: events::MouseDown) {
    long_click_clear();
    let timeout = Timeout::new(300, || edit_item(event));
    LONG_CLICK_TIMER.with(|timer| {
        timer.set(Some(timeout));
    });
}

// ===

fn dlg_group_view(label: &str, position: &usize, max: &usize) -> Dom {
    html!(TAG_DIV, {
        .class(css_class("dlg-body"))
        .children([
            element_label(label),
            element_position(position, max),
        ])
    })
}

fn dlg_item_view(label: &str, email: &str, position: &usize, max: &usize, idp: &usize) -> Dom {
    html!(TAG_DIV, {
        .class(css_class("dlg-body"))
        .children([
            element_label(label),
            element_email(email),
            if max>&0 {element_position(position, max)} else {html!(TAG_DIV)},
            element_select(idp)
        ])
    })
}

// ===

fn element_label(value: &str) -> Dom {
    element_input_string("Наименование", value, INPUT_NAME_LABEL)
}

fn element_email(value: &str) -> Dom {
    element_input_string("Email", value, INPUT_NAME_EMAIL)
}

fn element_select(idp: &usize) -> Dom {
    let idp = idp.clone() as i32;
    let children = NOTES.lock_ref().iter().filter(|row| row.idp.get() == 0)
        .map(|row| element_option(row.label.get_cloned(), row.idn, row.idn == idp))
        .collect::<Vec<_>>();

    html!(TAG_DIV, {
        .child(html!("select", {
            .attr(PROP_NAME, INPUT_NAME_IDP)
            .children(children)
        }))
    })
}

fn element_option(label: String, value: i32, selected: bool) -> Dom {
    let selected = Mutable::new(selected);
    html!(TAG_OPTION, {
        .attr(PROP_VALUE, &value.to_string())
        .prop_signal(PROP_SELECTED, selected.signal())
        .text(&label)
    })
}

fn element_input_string(title: &str, value: &str, name: &str) -> Dom {
    html!(TAG_DIV, {
        .child(html!(TAG_INPUT, {
            .attr(PROP_TITLE, title)
            .attr(PROP_PLACEHOLDER, title)
            .attr(PROP_TYPE, "string")
            .attr(PROP_NAME, name)
            .attr(PROP_VALUE, value)
            .attr("maxlength", "50")
        }))
    })
}

fn element_position(position: &usize, max: &usize) -> Dom {
    html!(TAG_DIV, {
        .child(html!(TAG_INPUT, {
            .attr(PROP_TITLE, "Порядковый номер")
            .attr(PROP_PLACEHOLDER, "Порядковый номер")
            .attr(PROP_TYPE, "number")
            .attr(PROP_NAME, INPUT_NAME_POSITION)
            .attr(PROP_VALUE, &position.to_string())
            .attr("step", "1")
            .attr("min", "1")
            .attr("max", &max.to_string())
        }))
    })
}