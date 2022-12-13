use dominator::{Dom, events, html};
use futures_signals::signal::{MutableSignalCloned, SignalExt};
use web_sys::UrlSearchParams;

use shared::constants::{API_FILE, ROOT_API};
use shared::types::{BoxMailAttachmentItem, BoxMailAttachments, MessageRequest};

use crate::constants::{PROP_HTML, PROP_TITLE, TAG_DIV, TAG_SPAN};
use crate::editor::app_editor::get_editor;
use crate::elements::app_login::get_user_box;
use crate::loader::message_update;
use crate::state::USER_KEY;
use crate::utils::{attr_data, from_dataset};

const ATTR_ID: &'static str = "id";

fn css_class(label: &str) -> String {
    format!("attachments__{label}")
}

pub fn attachments_preview(attachments: &BoxMailAttachments) -> Dom {
    html!(TAG_DIV, {
        .class(css_class("container-preview"))
        .children(attachments.list.iter().map(|item|{
            html!(TAG_DIV, {
                .child(item_link(item, &attachments.key, false))
            })
        }))
    })
}

fn item_link(row: &BoxMailAttachmentItem, key: &str, is_temp: bool) -> Dom {
    let params: String = if let Ok(params) = UrlSearchParams::new() {
        //params.append("user", user_key);
        params.append("filename", &row.file_name);
        params.append("email", &get_user_box());
        if is_temp {
            params.append("temp", "1");
        }
        params.to_string().into()
    } else { "".to_string() };

    let id = &row.id;
    let filename = format!("{key}-{id}");
    let href = format!("/{ROOT_API}/{API_FILE}/{filename}?{params}");

    html!("a", {
        .attr_signal("href", USER_KEY.signal_cloned().map(move|key|format!("{href}&user={key}")))
        .attr("download", &row.file_name)
        .attr(PROP_TITLE, &row.file_name)
        .children([
            html!(TAG_SPAN, {
                .class(css_class("filename"))
                .text(&row.file_name)
            }),
            html!(TAG_SPAN, {
                .text(&get_size(&row.size))
            })
        ])
    })
}

// ===

pub fn attachments_active(attachments_signal: MutableSignalCloned<Option<BoxMailAttachments>>) -> Dom {
    html!(TAG_DIV, {
        .child_signal(
            attachments_signal.map(|attachments|{
                match attachments {
                    Some(attachments)=>{
                        Some(html!(TAG_DIV, {
                            .class(css_class("container-active"))
                            .children(attachments.list.iter().map(|item|item_active(item, &attachments.key)))
                        }))
                    }
                    None=>None
                }
            })
        )
    })
}

fn item_active(row: &BoxMailAttachmentItem, key: &str) -> Dom {
    let icon_remove = include_str!("../icons/editor/remove.svg");
    html!(TAG_DIV, {
        .class(css_class("item"))
        .children([
            item_link(row, key, true),
            html!(TAG_SPAN, {
                .class(css_class("item-icon"))
                .attr(attr_data(ATTR_ID), &row.id.to_string())
                .prop(PROP_HTML, icon_remove)
                .event(handle_remove)
            })
        ])
    })
}

fn handle_remove(e: events::Click) {
    let id = from_dataset(e.target(), ATTR_ID).parse::<usize>().unwrap_or_default();
    if let Some(editor) = get_editor() {
        message_update(MessageRequest {
            idb: 0,
            attachments: editor.attachments.get_cloned(),
            remove_id: Some(id),
            ..MessageRequest::default()
        });
    }
}

// ===

fn get_size(size: &u64) -> String {
    let mut ext = "б";
    let mut size = size.clone() as f32;
    if size > 1024.0 {
        size = (size / 1024.0 * 10.0).round() / 10.0;
        ext = "кб";
    }
    if size > 1024.0 {
        size = (size / 1024.0 * 10.0).round() / 10.0;
        ext = "мб";
    }
    if size > 1024.0 {
        size = (size / 1024.0 * 10.0).round() / 10.0;
        ext = "Гб";
    }
    format!(" /{size} {ext}/")
}
