use dominator::{Dom, events, html};
use futures_signals::map_ref;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use js_sys::JsString;

use shared::types::{MailBoxes, NotesChannel, NotesEvent};

use crate::constants::{PROP_HTML, PROP_NAME, PROP_PLACEHOLDER, PROP_SELECTED, PROP_TITLE, PROP_TYPE, PROP_VALUE, TAG_BUTTON, TAG_DIV, TAG_INPUT, TAG_OPTION, TAG_SPAN};
use crate::dialog::dialogs::{Dialog, DialogButton, DialogType};
use crate::editor::app_editor::{open_email_editor, open_note_editor};
use crate::loader::notes_update;
use crate::state::{CURRENT_BOX, NOTES, NOTES_SELECTED};
use crate::utils::{create_element, get_element_from_node, get_input_value, view_email};

pub const PERIOD_DAY: i32 = 1;
pub const PERIOD_MONTH: i32 = 2;
pub const PERIOD_YEAR: i32 = 3;

fn css_class(label: &str) -> String {
    format!("notes-content__{label}")
}

pub fn notes_content() -> Dom {
    html!(TAG_DIV,{
        .visible_signal(notes_content_visible(true))
        .children([
            html!("p", {
                .class(css_class("header"))
                .child_signal(NOTES_SELECTED.signal().map(header))
            }),
            html!(TAG_DIV, {
                .class(css_class("tools"))
                .child(html!(TAG_BUTTON, {
                    .text("редактировать")
                    .event(handle_edit)
                }))
                .child_signal(NOTES_SELECTED.signal().map(button_create_email))
                .child_signal(button_event_signal())
                .child_signal(button_next_signal())
            }),
            html!(TAG_DIV, {
                .class(css_class("content"))
                .prop_signal(PROP_HTML, notes_content_html())
            })
        ])
    })
}

// ===

fn view_event(d: &str) -> String {
    let l: Vec<&str> = d.split("-").collect();
    format!("{}.{}.{}", l[2], l[1], l[0])
}

fn button_event_signal() -> impl Signal<Item=Option<Dom>> {
    NOTES_SELECTED.signal().map(button_event_sub_signal).flatten()
}

fn button_next_signal() -> impl Signal<Item=Option<Dom>> {
    NOTES_SELECTED.signal().map(button_next_sub_signal).flatten()
}


fn event_signal(idn: i32) -> impl Signal<Item=Option<NotesEvent>> {
    match NOTES.lock_ref().iter().find(|row| row.idn == idn) {
        Some(item) => item.event.signal_cloned(),
        None => Mutable::new(None).signal_cloned()
    }
}

fn button_event_sub_signal(idn: i32) -> impl Signal<Item=Option<Dom>> {
    event_signal(idn).map(|event| {
        let (label, title) = match event {
            Some(event) => (view_event(&event.date), "редактировать событие".to_string()),
            None => ("добавить событие".to_string(), "добавить событие".to_string())
        };
        Some(html!(TAG_BUTTON, {
            .attr(PROP_TITLE, &title)
            .text(&label)
            .event(dlg_event_open)
        }))
    })
}

fn period_to_text(ind: &i32) -> String {
    match ind {
        &PERIOD_DAY => "день".to_string(),
        &PERIOD_MONTH => "месяц".to_string(),
        &PERIOD_YEAR => "год".to_string(),
        _ => "".to_string()
    }
}

fn button_next_sub_signal(idn: i32) -> impl Signal<Item=Option<Dom>> {
    event_signal(idn).map(|event| {
        match event {
            Some(event) => Some(html!(TAG_BUTTON, {
                .attr(PROP_TITLE, "выполнено")
                .text(&format!("+{} {}", event.delta, period_to_text(&event.period)))
                .event(handle_next)
            })),
            None => None
        }
    })
}

fn handle_next(_: events::Click) {
    let idn = NOTES_SELECTED.get();
    if let Some(item) = NOTES.lock_ref().iter().find(|row| row.idn == idn) {
        if let Some(event) = item.event.get_cloned() {
            let js_date: js_sys::Date = js_sys::Date::new(&event.date.clone().into());
            let period = event.period.clone();
            let delta = event.delta.clone() as u32;
            match period {
                PERIOD_DAY => {
                    js_date.set_date(js_date.get_date() + delta);
                }
                PERIOD_MONTH => {
                    js_date.set_month(js_date.get_month() + delta);
                }
                PERIOD_YEAR => {
                    js_date.set_full_year(js_date.get_full_year() + delta);
                }
                _ => {}
            }
            let date: String = js_date.to_json().substring(0, 10).into();
            let delta = delta as i32;
            notes_update(NotesChannel {
                idn: NOTES_SELECTED.get(),
                event: Some(NotesEvent { date, delta, period }),
                ..NotesChannel::default()
            });
        }
    }
}


// ===

fn dlg_event_open(_: events::Click) {
    Dialog::custom(Dialog {
        type_: DialogType::Form,
        title: "Событие".to_string(),
        form: dlg_event_init,
        confirm: dlg_event_result,
        before: vec![DialogButton { label: "удалить событие".to_string(), click: remove_event }],
        ..Dialog::default()
    });
}

fn remove_event() -> bool {
    notes_update(NotesChannel {
        idn: NOTES_SELECTED.get(),
        event: Some(NotesEvent::default()),
        ..NotesChannel::default()
    });
    true
}

const INPUT_NAME_DATE: &'static str = "date";
const INPUT_NAME_DELTA: &'static str = "delta";
const INPUT_NAME_PERIOD: &'static str = "period";

fn dlg_event_init() -> Dom {
    let current_date: String = String::from(js_sys::Date::new_0().to_json().substring(0, 10));
    let idn = NOTES_SELECTED.get();
    let (date, period, delta) = match NOTES.lock_ref().iter().find(|row| row.idn == idn) {
        Some(item) => {
            match item.event.get_cloned() {
                Some(event) => (event.date, event.period, event.delta),
                None => (current_date, PERIOD_MONTH, 1)
            }
        }
        None => (current_date, PERIOD_MONTH, 1)
    };

    html!(TAG_DIV, {
        .class(css_class("event-dlg"))
        .children([
            html!(TAG_DIV, {
                .child(html!(TAG_INPUT, {
                    .attr(PROP_TITLE, "дата")
                    .attr(PROP_PLACEHOLDER, "дата")
                    .attr(PROP_TYPE, "date")
                    .attr(PROP_NAME, INPUT_NAME_DATE)
                    .attr(PROP_VALUE, &date)
                }))
            }),
            html!(TAG_DIV, {
                .child(html!(TAG_INPUT, {
                    .attr(PROP_TITLE, "период (интервал)")
                    .attr(PROP_PLACEHOLDER, "период (интервал)")
                    .attr(PROP_TYPE, "number")
                    .attr(PROP_NAME, INPUT_NAME_DELTA)
                    .attr(PROP_VALUE, &delta.to_string())
                    .attr("min", "1")
                    .attr("step", "1")
                }))
            }),
            html!(TAG_DIV, {
                .child(html!("select", {
                    .attr(PROP_TITLE, "период (вид)")
                    .attr(PROP_NAME, INPUT_NAME_PERIOD)
                    .children([
                        html!(TAG_OPTION, {
                            .attr(PROP_VALUE, &PERIOD_DAY.to_string())
                            .prop_signal(PROP_SELECTED, Mutable::new(period==PERIOD_DAY).signal())
                            .text(&period_to_text(&PERIOD_DAY))
                        }),
                        html!(TAG_OPTION, {
                            .attr(PROP_VALUE, &PERIOD_MONTH.to_string())
                            .prop_signal(PROP_SELECTED, Mutable::new(period==PERIOD_MONTH).signal())
                            .text(&period_to_text(&PERIOD_MONTH))
                        }),
                        html!(TAG_OPTION, {
                            .attr(PROP_VALUE, &PERIOD_YEAR.to_string())
                            .prop_signal(PROP_SELECTED, Mutable::new(period==PERIOD_YEAR).signal())
                            .text(&period_to_text(&PERIOD_YEAR))
                        })
                    ])
                }))
            }),
        ])
    })
}

fn dlg_event_result() {
    let date = get_input_value(INPUT_NAME_DATE).trim().to_string();
    let delta = get_input_value(INPUT_NAME_DELTA).parse::<i32>().unwrap_or(1);
    let period = get_input_value(INPUT_NAME_PERIOD).parse::<i32>().unwrap_or(PERIOD_MONTH);
    let js_date: js_sys::Date = js_sys::Date::new(&date.clone().into());

    // js_sys::Date::new(&date.clone().into()).to_json().substring(0, 10)==date
    if js_date.to_string() != JsString::from("Invalid Date") && delta > 0 {
        notes_update(NotesChannel {
            idn: NOTES_SELECTED.get(),
            event: Some(NotesEvent { date, delta, period }),
            ..NotesChannel::default()
        });
    }
}

// ===

fn button_create_email(idn: i32) -> Option<Dom> {
    match NOTES.lock_ref().iter().find(|row| row.idn == idn) {
        Some(item) => {
            match item.email.get_cloned().is_empty() {
                true => None,
                false => Some(html!(TAG_BUTTON, {
                    .text("создать письмо")
                    .event(handle_create_email)
                }))
            }
        }
        None => None
    }
}

fn handle_create_email(_: events::Click) {
    let marker_subject = "[subject]";
    let marker_content_start = "[content]";
    let marker_content_end = "[//content]";

    let mut subject = "".to_string();
    let mut content: Vec<String> = vec![];

    let idn = NOTES_SELECTED.get();
    let (source, label, email) = match NOTES.lock_ref().iter().find(|row| row.idn == idn) {
        Some(item) => (item.content.get_cloned(), item.label.get_cloned(), item.email.get_cloned()),
        None => ("".to_string(), "".to_string(), "".to_string())
    };

    if !source.is_empty() {
        let mut flag_content = false;
        let div = create_element("div");
        div.set_inner_html(&source);
        let list = div.child_nodes();
        for ind in 0..list.length() {
            if let Some(node) = list.item(ind) {
                let text: String = node.text_content().unwrap_or_default().trim().to_string();
                if text.starts_with(marker_subject) {
                    subject = text[marker_subject.len()..].trim().to_string();
                }
                if text.contains(marker_content_end) {
                    flag_content = false;
                }
                if flag_content && node.node_type() == 1 {
                    if let Some(elem) = get_element_from_node(Some(node)) {
                        content.push(elem.outer_html());
                    }
                }
                if text.contains(marker_content_start) {
                    flag_content = true;
                }
            }
        }
    }

    open_email_editor(0, view_email(&label, &email), subject, content.join(""));
}

// ===

fn handle_edit(_: events::Click) {
    let idn = NOTES_SELECTED.get();
    let content = match NOTES.lock_ref().iter().find(|row| row.idn == idn) {
        Some(item) => item.content.get_cloned(),
        None => "".to_string()
    };
    open_note_editor(true, content);
}

fn notes_content_html() -> impl Signal<Item=String> {
    NOTES_SELECTED.signal().map(|idn| match NOTES.lock_ref().iter().find(|row| row.idn == idn) {
        Some(item) => {
            item.content.signal_cloned()
        }
        None => Mutable::new("".to_string()).signal_cloned()
    }).flatten()
}

pub fn notes_content_visible(flag: bool) -> impl Signal<Item=bool> {
    map_ref! {
        let flag1 = CURRENT_BOX.signal().map(|b|b==MailBoxes::Notes),
        let flag2 = NOTES_SELECTED.signal().map(move|idn|(flag&&idn>0)||(!flag&&idn==0)) =>
        *flag1 && *flag2
    }
}

fn header(idn: i32) -> Option<Dom> {
    let (label_item, email, idp) = match NOTES.lock_ref().iter().find(|row| row.idn == idn) {
        Some(item) => (item.label.get_cloned(), item.email.get_cloned(), item.idp.get()),
        None => ("".to_string(), "".to_string(), 0)
    };
    let label_group = match NOTES.lock_ref().iter().find(|row| row.idn == idp) {
        Some(item) => item.label.get_cloned(),
        None => "".to_string()
    };
    Some(html!(TAG_SPAN, {
        .child(html!("b", {
                .text(&label_group)
            }))
        .child(html!("br"))
        .text(&label_item)
        .text(" ")
        .child(html!("small", {
            .child(html!("i", {
                .text(&email)
            }))
        }))
    }))
}
