use dominator::{Dom, events, html, svg};
use futures_signals::signal::{Signal, SignalExt};
use wasm_bindgen_futures::spawn_local;

use shared::types::MailBoxes;
use shared::utils::box_type_index;

use crate::constants::{TAG_BUTTON, TAG_DIV};
use crate::editor::app_editor::open_email_editor;
use crate::elements::app_login::get_user_box;
use crate::notes::notes_events::handle_events;
use crate::state::{BOX_STATE, CURRENT_BOX, EVENTS, LOADING_NEXT, USER_KEY};
use crate::utils::{location_reload, query_selector};

fn css_class(label: &str) -> String {
    format!("app-header__{label}")
}

pub fn app_header() -> Dom {
    html!(TAG_DIV, {
        .class(css_class("container"))
        .child_signal(events())
        .children([
            button("написать", new_mail),
            button_typed("входящие", MailBoxes::Inbox),
            button_typed("прочтенные", MailBoxes::Ready),
            button_typed("отправленные", MailBoxes::Sent),
            button_typed("корзина", MailBoxes::Trash),
            button_typed("заметки", MailBoxes::Notes),
            button(&get_user_box(), location_reload),
            button_icon(icon_exit(), handle_exit)
        ])
    })
}

fn events() -> impl Signal<Item=Option<Dom>> {
    EVENTS.signal_cloned().map(|items| if !items.is_empty() { Some(button(&format!("[{}]", items.len()), handle_events)) } else { None })
}

fn new_mail() {
    open_email_editor(0, "".to_string(), "".to_string(), "".to_string());
}

fn handle_exit() {
    USER_KEY.set("".to_string());
    location_reload();
}

fn button_typed(label: &str, mb: MailBoxes) -> Dom {
    let ind_next = box_type_index(&mb);
    html!(TAG_BUTTON, {
        .class(css_class("button"))
        .class(css_class("text"))
        .class_signal("active", CURRENT_BOX.signal().map(move|x|x==mb))
        .text(label)
        .event(move|_: events::Click|{
            let ind_prev = box_type_index(&CURRENT_BOX.get());
            if let Some(elem) = query_selector("#col-1") {
                BOX_STATE[ind_prev].scroll_text.set(elem.scroll_top());
            }
            if let Some(elem) = query_selector("#col-2") {
                BOX_STATE[ind_prev].scroll_list.set(elem.scroll_top());
            }
            CURRENT_BOX.set(mb);

            spawn_local(async move {
                if let Some(elem) = query_selector("#col-1") {
                    elem.set_scroll_top(BOX_STATE[ind_next].scroll_text.get());
                }
            });

            if let Some(elem) = query_selector("#col-2") {
                elem.set_scroll_top(BOX_STATE[ind_next].scroll_list.get());
            }

            LOADING_NEXT.set(false);
        })
    })
}

fn button(label: &str, click: fn()) -> Dom {
    html!(TAG_BUTTON, {
        .class(css_class("button"))
        .class(css_class("text"))
        .text(label)
        .event(move|_: events::Click|click())
    })
}

fn button_icon(icon: Dom, click: fn()) -> Dom {
    html!(TAG_BUTTON, {
        .class(css_class("button"))
        .children([icon])
        .event(move|_: events::Click|click())
    })
}

fn icon_exit() -> Dom {
    svg!("svg", {
        .attr("viewBox", "0 0 512 512")
        .class(css_class("icon"))
        .children([
            svg!("path", {
                .attr("fill", "currentColor")
                .attr("d", "M497 273L329 441c-15 15-41 4.5-41-17v-96H152c-13.3 0-24-10.7-24-24v-96c0-13.3 10.7-24 24-24h136V88c0-21.4 25.9-32 41-17l168 168c9.3 9.4 9.3 24.6 0 34zM192 436v-40c0-6.6-5.4-12-12-12H96c-17.7 0-32-14.3-32-32V160c0-17.7 14.3-32 32-32h84c6.6 0 12-5.4 12-12V76c0-6.6-5.4-12-12-12H96c-53 0-96 43-96 96v192c0 53 43 96 96 96h84c6.6 0 12-5.4 12-12z")
            })
        ])
    })
}
