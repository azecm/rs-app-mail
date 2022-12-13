use dominator::{Dom, html};
use futures_signals::signal::SignalExt;
use futures_signals::signal_vec::SignalVecExt;

use crate::constants::{EMAIL_DATALIST, PROP_VALUE, TAG_DIV, TAG_OPTION};
use crate::dialog::dialogs::dialogs;
use crate::editor::app_editor::app_editor;
use crate::elements::app_body::app_body;
use crate::elements::app_header::app_header;
use crate::state::NOTES;
use crate::types::NoteStruct;
use crate::utils::view_email;

pub fn app_root() -> Dom {
    html!(TAG_DIV, {
        .class("app-root")
        .children([app_header(), app_body()])
        .child_signal(app_editor())
        .child_signal(dialogs())
        .child_signal(NOTES.signal_vec_cloned().to_signal_cloned().map(data_list))
    })
}

fn data_list(notes: Vec<NoteStruct>) -> Option<Dom> {
    let options = notes.iter()
        .map(|row| view_email(&row.label.get_cloned(), &row.email.get_cloned()))
        .filter(|text| !text.is_empty())
        .map(|text| html!(TAG_OPTION, {.attr(PROP_VALUE, &text)}))
        .collect::<Vec<_>>();

    Some(html!("datalist", {
        .attr("id", EMAIL_DATALIST)
        .children(options)
    }))
}
