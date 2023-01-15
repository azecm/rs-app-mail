use std::collections::HashMap;

use dominator::{Dom, events, html};
use futures_signals::signal::SignalExt;

use shared::types::MailBoxes;

use crate::constants::{TAG_DIV, TAG_SPAN};
use crate::notes::notes_content::notes_content_visible;
use crate::state::{CURRENT_BOX, EVENTS, NOTES, NOTES_SELECTED};
use crate::types::EventItemStruct;
use crate::utils::{attr_data, from_dataset, query_selector_all};

fn css_class(label: &str) -> String {
    format!("notes-event__{label}")
}

pub fn events_reload() {
    let delta = 1000.0 * 3600.0 * 24.0 * 3.0;
    let groups = NOTES.lock_ref()
        .iter()
        .filter(|row| row.idp.get() == 0).map(|item| (item.idn, item.label.get_cloned())).collect::<HashMap<_, _>>();
    let mut items = NOTES.lock_ref().iter().filter_map(|note| {
        match note.event.get_cloned() {
            Some(event) => {
                let js_date: js_sys::Date = js_sys::Date::new(&event.date.clone().into());
                if js_date.get_time() - js_sys::Date::now() < delta {
                    let group = match groups.get(&note.idp.get()) {
                        Some(v) => v.clone(),
                        None => "".to_string()
                    };
                    Some(EventItemStruct {
                        idn: note.idn,
                        idp: note.idp.get_cloned(),
                        label: note.label.get_cloned(),
                        order: js_date.get_time() as i64,
                        date: event.date,
                        group,
                    })
                } else {
                    None
                }
            }
            None => None
        }
    }).collect::<Vec<_>>();

    items.sort_by(|a, b| a.order.cmp(&b.order));

    EVENTS.set(items);
}

pub fn handle_events() {
    NOTES_SELECTED.set(0);
    CURRENT_BOX.set(MailBoxes::Notes);
}

pub fn events_content() -> Dom {
    html!(TAG_DIV,{
        .visible_signal(notes_content_visible(false).map(|flag|flag && EVENTS.lock_ref().len()>0))
        .class(css_class("container"))
        .child(html!("h1", {
            .text("Напоминания")
        }))
        .child_signal(EVENTS.signal_cloned().map(events_view))
    })
}

fn events_view(notes: Vec<EventItemStruct>) -> Option<Dom> {
    if notes.is_empty() {
        None
    } else {
        Some(html!("ul", {
            .children(notes.iter().map(|event|{
                html!("li", {
                    .child(html!(TAG_SPAN,{
                        .class(css_class("item"))
                        .attr(attr_data("idn"), &event.idn.to_string())
                        .attr(attr_data("idp"), &event.idp.to_string())
                        .event(handle_select)
                        .child(html!(TAG_SPAN, {
                            .class(css_class("date"))
                            .text(&event.date)
                        }))
                        .text(" ")
                        .child(html!("small", {
                            .text(&event.group)
                            .text(": ")
                            .text(&event.label)
                        }))
                    }))
                })
            }))
        }))
    }
}

fn handle_select(e: events::Click) {
    let idn = from_dataset(e.target(), "idn").parse::<i32>().unwrap_or_default();
    let idp = from_dataset(e.target(), "idp").parse::<i32>().unwrap_or_default();
    if idn > 0 && idp > 0 {
        NOTES_SELECTED.set(idn);

        let groups = NOTES.lock_ref()
            .iter()
            .filter_map(|row| if row.idp.get() == 0 { Some(row.idn) } else { None }).collect::<Vec<_>>();
        let position = groups.iter().position(|val| val == &idp).unwrap_or_default();
        for (ind, elem) in query_selector_all("#col-2 details").into_iter().enumerate() {
            if position == ind {
                elem.set_attribute("open", "").ok();
            } else {
                elem.remove_attribute("open").ok();
            }
        }
    }
}