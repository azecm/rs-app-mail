use std::ops::Deref;

use dominator::{Dom, events, html};
use futures_signals::signal::{Mutable, Signal, SignalExt};
use futures_signals::signal_vec::{MutableVec, SignalVecExt};
use once_cell::sync::Lazy;
use web_sys::Range;

use crate::constants::{TAG_BUTTON, TAG_DIV};
use crate::utils::get_selection;

pub static DIALOGS: Lazy<MutableVec<Dialog>> = Lazy::new(|| {
    MutableVec::new()
});

thread_local! {
    static RANGE_SAVED: Lazy<Mutable<Option<Range>>> = Lazy::new(||Mutable::new(None));
}

#[derive(Clone, Debug)]
pub struct Dialog {
    pub type_: DialogType,
    pub title: String,
    pub message: String,
    pub form: fn() -> Dom,
    pub confirm: fn(),
    pub cancel: fn(),
    pub before: Vec<DialogButton>,
}

impl Default for Dialog {
    fn default() -> Self {
        Self {
            type_: DialogType::Alert,
            title: "".to_string(),
            message: "".to_string(),
            form: || html!(TAG_DIV),
            confirm: || {},
            cancel: || {},
            before: vec![],
        }
    }
}

#[derive(Clone, Debug)]
pub struct DialogButton {
    pub label: String,
    pub click: fn() -> bool,
}

#[derive(Clone, Debug)]
pub enum DialogType {
    Alert,
    Confirm,
    Form,
}

impl Dialog {
    pub fn alert(message: &str) {
        common_open();
        DIALOGS.lock_mut().push_cloned(Self {
            type_: DialogType::Alert,
            title: "".to_string(),
            message: message.to_string(),
            form: || html!(TAG_DIV),
            confirm: || {},
            cancel: || {},
            ..Dialog::default()
        });
    }
    pub fn confirm(message: &str, confirm: fn(), cancel: fn()) {
        common_open();
        DIALOGS.lock_mut().push_cloned(Self {
            type_: DialogType::Confirm,
            title: "".to_string(),
            message: message.to_string(),
            form: || html!(TAG_DIV),
            confirm,
            cancel,
            ..Dialog::default()
        });
    }
    pub fn form(title: &str, form: fn() -> Dom, confirm: fn(), cancel: fn()) {
        common_open();
        DIALOGS.lock_mut().push_cloned(Self {
            type_: DialogType::Form,
            title: title.to_string(),
            message: "".to_string(),
            form,
            confirm,
            cancel,
            ..Dialog::default()
        });
    }
    pub fn custom(dialog: Dialog) {
        common_open();
        DIALOGS.lock_mut().push_cloned(dialog);
    }
}

// ===

fn css_class(label: &str) -> String {
    format!("dialogs__{label}")
}

pub fn dialogs() -> impl Signal<Item=Option<Dom>> {
    DIALOGS.signal_vec_cloned().to_signal_cloned().map(current_element)
}

fn current_element(list: Vec<Dialog>) -> Option<Dom> {
    match list.last() {
        Some(dialog) => {
            match dialog.type_ {
                DialogType::Alert => Some(dialog_alert(dialog)),
                DialogType::Confirm => Some(dialog_confirm(dialog)),
                DialogType::Form => Some(dialog_form(dialog))
            }
        }
        None => None
    }
}

fn dialog_common(rows: Vec<Dom>) -> Dom {
    html!(TAG_DIV, {
        .class(css_class("back"))
        .child(html!(TAG_DIV, {
            .class(css_class("container"))
            .children(rows)
        }))
    })
}

fn dialog_alert(data: &Dialog) -> Dom {
    dialog_common(vec![
        html!(TAG_DIV,{
            .class(css_class("header"))
        }),
        html!(TAG_DIV,{
            .class(css_class("body"))
            .text(&data.message)
        }),
        html!(TAG_DIV,{
            .class(css_class("footer"))
            .children([
                html!(TAG_BUTTON, {
                    .text("Ok")
                    .attr("aria-label", "confirm")
                    .event(|_: events::Click|{
                        dialog_close();
                    })
                }),
            ])
        }),
    ])
}

fn dialog_confirm(data: &Dialog) -> Dom {
    dialog_common(vec![
        html!(TAG_DIV,{
            .class(css_class("header"))
            .text(&data.title)
        }),
        html!(TAG_DIV,{
            .class(css_class("body"))
            .text(&data.message)
        }),
        dialog_footer(data),
    ])
}

fn dialog_form(data: &Dialog) -> Dom {
    dialog_common(vec![
        html!(TAG_DIV,{
            .class(css_class("header"))
            .text(&data.title)
        }),
        html!(TAG_DIV,{
            .class(css_class("body"))
            .child((data.form)())
        }),
        dialog_footer(data),
    ])
}

fn dialog_footer(data: &Dialog) -> Dom {
    let confirm = data.confirm;
    let cancel = data.cancel;

    let mut buttons: Vec<Dom> = data.before.iter()
        .map(|row| {
            let click = row.click;
            html!(TAG_BUTTON, {
            .text(&row.label)
            .event(move|_: events::Click|{
                    if click() {
                        dialog_close();
                    }
            })
        })
        }).collect();

    buttons.push(html!(TAG_BUTTON, {
        .text("Да")
        .attr("aria-label", "confirm")
        .event(move|_: events::Click|{
            confirm();
            dialog_close();
        })
    }));
    buttons.push(html!(TAG_BUTTON, {
        .text("Нет")
        .attr("aria-label", "cancel")
        .event(move|_: events::Click|{
            cancel();
            dialog_close();
        })
    }));

    html!(TAG_DIV,{
        .class(css_class("footer"))
        .children(buttons)
    })
}

fn dialog_close() {
    let count = DIALOGS.lock_mut().len();
    if count > 0 {
        DIALOGS.lock_mut().remove(count - 1);
    }
    common_close();
}

fn common_open() {
    let count = DIALOGS.lock_mut().len();

    if count == 0 {
        if let Some(selection) = get_selection() {
            if selection.range_count() > 0 {
                if let Ok(current) = selection.get_range_at(0) {
                    RANGE_SAVED.with(|range| range.set(Some(current)));
                }
            }
        }
    }
}

fn common_close() {
    let count = DIALOGS.lock_mut().len();
    if count == 0 {
        if let Some(selection) = get_selection() {
            if selection.remove_all_ranges().is_ok() {
                RANGE_SAVED.with(|saved| {
                    match saved.lock_ref().deref() {
                        Some(saved) => {
                            if selection.add_range(saved).is_ok() {}
                        }
                        None => {}
                    }
                });
            }
        }
    }
}

/*
#[link(wasm_import_module = "./index.js")]
extern {
    fn save_selection();
}

#[link(wasm_import_module = "./index.js")]
extern {
    fn load_selection();
}
*/
