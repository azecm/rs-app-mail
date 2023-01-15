use dominator::{Dom, events, html};
use futures_signals::signal::{Mutable, SignalExt};
use once_cell::sync::Lazy;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Element, FormData, HtmlInputElement};

use shared::types::{MessageRequest, NotesChannel};
use shared::utils::box_type_index;

use crate::connect_files::connect_files;
use crate::constants::{PROP_EDITABLE, PROP_HTML, PROP_NAME, PROP_PLACEHOLDER, PROP_TITLE, TAG_BUTTON, TAG_DIV, TAG_INPUT, TAG_SPAN};
use crate::dialog::dialogs::Dialog;
use crate::editor::app_editor::{editor_close, open_email_editor};
use crate::editor::icons::{icon_attach, icon_close, icon_envelope, icon_eraser, icon_font_bold, icon_font_italic, icon_font_underline, icon_forward, icon_heading, icon_link, icon_list_ol, icon_list_ul, icon_paragraph, icon_reply, icon_save, icon_send, icon_unlink};
use crate::editor::state::EDITOR;
use crate::loader::{message_update, notes_update};
use crate::state::{CURRENT_BOX, NOTES_SELECTED};
use crate::utils::{drop_element, exec_command, exec_command_full, get_element_from_node, get_html_element, get_input_value, get_selection, node_parent, obj_to_string, query_selector};

static LINK: Lazy<Mutable<String>> = Lazy::new(|| {
    Mutable::new("".to_string())
});

thread_local! {
    static LINK_ELEMENT: Lazy<Mutable<Option<Element>>> = Lazy::new(|| {
        Mutable::new(None)
    });
}

fn css_class(label: &str) -> String {
    format!("editor-tools__{label}")
}

pub fn editor_tools(is_note: bool) -> Dom {
    let btn_save = match is_note {
        true => button("сохранить", handle_save, icon_save),
        false => button("отправить", handle_send, icon_send)
    };
    let mut buttons = vec![
        button("закрыть", handle_close, icon_close),
        btn_save,
        html!(TAG_SPAN, {.class(css_class("space"))}),
        button("удалить форматирование", handle_eraser, icon_eraser),
        button("заголовок", handle_heading, icon_heading),
        button("параграф", handle_paragraph, icon_paragraph),
        button("жирный", handle_bold, icon_font_bold),
        button("курсив", handle_italic, icon_font_italic),
        button("подчеркнутый", handle_underline, icon_font_underline),
        button("маркированный список", handle_unordered, icon_list_ul),
        button("нумерованный список", handle_ordered, icon_list_ol),
        button("ссылка", handle_link, icon_link),
        button("удалить ссылку", handle_unlink, icon_unlink),
    ];

    if !is_note {
        buttons.push(button_attach());
    }

    html!(TAG_DIV, {
        .class(css_class("container"))
        .children(buttons)
    })
}

fn handle_eraser(_: events::Click) {
    return_focus();
    exec_command("removeFormat");
}

fn handle_bold(_: events::Click) {
    return_focus();
    exec_command("bold");
}

fn handle_italic(_: events::Click) {
    return_focus();
    exec_command("italic");
}

fn handle_underline(_: events::Click) {
    return_focus();
    exec_command("underline");
}

fn handle_paragraph(_: events::Click) {
    return_focus();
    exec_command_full("formatBlock", false, "p");
}

fn handle_heading(_: events::Click) {
    return_focus();
    exec_command_full("formatBlock", false, "h1");
}

fn handle_unordered(_: events::Click) {
    return_focus();
    exec_command("insertUnorderedList");
}

fn handle_ordered(_: events::Click) {
    return_focus();
    exec_command("insertOrderedList");
}

fn handle_unlink(_: events::Click) {
    return_focus();
    exec_command("unlink");
}

fn handle_link(_: events::Click) {
    return_focus();
    let mut flag = false;
    LINK.set_neq("https://".to_string());
    LINK_ELEMENT.with(|m| m.set_neq(None));

    if let Some(selection) = get_selection() {
        if selection.range_count() > 0 {
            if let Some(node) = selection.anchor_node() {
                if let Some(element) = get_element_from_node(node_parent(node, "a")) {
                    if let Some(text) = element.get_attribute("href") {
                        LINK.set_neq(text);
                        LINK_ELEMENT.with(|m| m.set_neq(Some(element)));
                        flag = true;
                    }
                }
            }
        }
    }

    let title = if flag { "Свойства ссылки" } else { "Создать ссылку" };
    Dialog::form(title, dlg_link_init, dlg_link_result, || {});
}

fn dlg_link_init() -> Dom {
    html!("textarea", {
        .class(css_class("link"))
        .attr(PROP_TITLE, "адрес ссылки")
        .attr(PROP_PLACEHOLDER, "адрес ссылки")
        .attr("rows", "3")
        .attr(PROP_NAME, "link")
        .prop(PROP_HTML, &LINK.get_cloned())
    })
}

fn dlg_link_result() {
    spawn_local(async {
        let link = get_input_value("link").trim().to_string();
        LINK_ELEMENT.with(|element| {
            match element.get_cloned() {
                Some(element) => {
                    if link.is_empty() {
                        drop_element(element);
                    } else if element.set_attribute("href", &link).is_ok() {}
                }
                None => {
                    if !link.is_empty() {
                        exec_command_full("createLink", false, &link);
                        after_link_created();
                    }
                }
            }
        });
    });
}

fn after_link_created() {
    spawn_local(async {
        while let Some(elem) = query_selector("[_moz_dirty]") {
            if elem.remove_attribute("_moz_dirty").is_ok() {}
        }
    });
}

fn handle_save(_: events::Click) {
    return_focus();
    if let Some(elem) = query_selector(&format!("[{PROP_EDITABLE}]")) {
        let content: String = elem.inner_html();
        let idn = NOTES_SELECTED.get();
        notes_update(NotesChannel {
            idn,
            content: Some(content),
            ..NotesChannel::default()
        });
    };
}

fn handle_close(_: events::Click) {
    return_focus();
    if let Some(editor) = EDITOR.get_cloned() {
        if editor.editable {
            if let Some(attachments) = editor.attachments.get_cloned() {
                if !attachments.list.is_empty() {
                    message_update(MessageRequest {
                        idb: 0,
                        attachments: Some(attachments),
                        ..MessageRequest::default()
                    });
                }
            }
        }
    }
    editor_close();
}

fn return_focus() {
    if let Some(element) = get_html_element(query_selector(&format!("[{PROP_EDITABLE}]"))) { if element.focus().is_ok() {} };
}

fn button(title: &str, click: fn(events::Click), icon: fn()->Dom) -> Dom {
    html!(TAG_BUTTON, {
        .attr(PROP_TITLE, title)
        .event(click)
        .child(icon())
    })
}

// ===

pub static UPLOAD_STARTED: Lazy<Mutable<bool>> = Lazy::new(|| {
    Mutable::new(false)
});

pub static UPLOAD_PROGRESS: Lazy<Mutable<u32>> = Lazy::new(|| {
    Mutable::new(0)
});

fn handle_progress(val: u32) {
    UPLOAD_PROGRESS.set(val);
}

fn handle_progress_final(success: bool) {
    if !success {
        Dialog::alert("Ошибка при загрузке файла.");
    }
    UPLOAD_STARTED.set(false);
    UPLOAD_PROGRESS.set(0);
}

fn handle_change(e: events::Change) {
    if let Some(target) = e.target() {
        if let Some(input) = JsValue::from(target).dyn_ref::<HtmlInputElement>() {
            if let Some(files) = input.files() {
                if let Some(editor) = EDITOR.get_cloned() {
                    if let Some(attachments) = editor.attachments.get_cloned() {
                        let current = obj_to_string(&attachments);
                        if let Ok(form) = FormData::new() {
                            form.append_with_str("current", &current).ok();
                            for ind in 0..files.length() {
                                if let Some(file) = files.item(ind) {
                                    form.append_with_blob("files", &file).ok();
                                }
                            }
                            connect_files(&form, handle_progress, handle_progress_final);
                            UPLOAD_STARTED.set(true);
                        }
                    }
                }
            }
        }
    }
}

fn button_attach() -> Dom {
    html!(TAG_BUTTON, {
        .attr(PROP_TITLE, "прикрепить файлы")
        .child_signal(UPLOAD_STARTED.signal().map(|flag|
            if flag {Some(html!(TAG_DIV, {
                .class(css_class("progress"))
                .child(html!(TAG_SPAN, {
                    .class(css_class("progress-text"))
                    .text_signal(UPLOAD_PROGRESS.signal_cloned().map(|val| format!("{val}%")))
                }))
            }))}
            else {None}
        ))
        .child(html!(TAG_DIV, {
            .class(css_class("icon-block"))
            .child_signal(UPLOAD_STARTED.signal().map(move|flag|if flag {None} else {Some(icon_attach())}))
        }))
        .child(html!(TAG_INPUT, {
            .class(css_class("input-file"))
            .attr("type", "file")
            .attr("multiple", "")
            .prop_signal("disabled", UPLOAD_STARTED.signal())
            .event(handle_change)
        }))
    })
}

// ===

pub fn editor_preview_tools() -> Dom {
    let with_unread = EDITOR.get_cloned().unwrap_or_default().with_unread;
    let mut buttons = vec![
        button("закрыть", handle_close, icon_close),
        button("ответить", handle_reply, icon_reply),
        button("переслать", handle_forward, icon_forward),
        html!(TAG_SPAN, {.class(css_class("space"))}),
    ];
    if with_unread {
        buttons.push(button("отменить прочтение", handle_unread, icon_envelope));
    }

    html!(TAG_DIV, {
        .class(css_class("container"))
        .children(buttons)
    })
}

fn handle_reply(_: events::Click) {
    if let Some(editor) = EDITOR.get_cloned() {
        let recipient = editor.sender.unwrap_or_default();
        let subject = format!("RE: {}", editor.subject.unwrap_or_default());
        let content = format!("<p><br></p><blockquote>{}</blockquote>", editor.content);
        open_email_editor(0, recipient, subject, content);
    }
}

fn handle_forward(_: events::Click) {
    if let Some(editor) = EDITOR.get_cloned() {
        let sender = to_html(editor.sender.unwrap_or_default());
        let recipient = to_html(editor.recipient.unwrap_or_default());
        let subject = format!("FW: {}", editor.subject.unwrap_or_default());
        let content = format!("<p><b>Отправитель:</b> {sender}<br><b>Переадресовано с:</b> {recipient}</p><p><br></p><hr>{}", editor.content);
        open_email_editor(editor.idb, "".to_string(), subject, content);
    }
}

fn to_html(text: String) -> String {
    text.replace('<', "&lt;").replace('>', "&gt;")
}

fn handle_unread(_: events::Click) {
    if let Some(editor) = EDITOR.get_cloned() {
        if editor.idb > 0 {
            message_update(MessageRequest {
                idb: editor.idb,
                unread: Some(true),
                box_current: Some(box_type_index(&CURRENT_BOX.get()) as i32),
                ..MessageRequest::default()
            });
            editor_close();
        }
    }
}

fn handle_send(_: events::Click) {
    return_focus();
    log::info!("handle_send");
    if let Some(editor) = EDITOR.get_cloned() {
        let recipient = get_input_value("recipient");
        if recipient.is_empty() {
            Dialog::alert("Укажите получателя");
            return;
        }
        let subject = get_input_value("subject");
        if subject.is_empty() {
            Dialog::alert("Укажите тему");
            return;
        }
        let content: String = match query_selector("[contenteditable=true]") {
            Some(elem) => {
                elem.inner_html()
            }
            None => {
                Dialog::alert("Блок содержания не найден");
                return;
            }
        };
        message_update(MessageRequest {
            send: Some(true),
            attachments: editor.attachments.get_cloned(),
            subject: Some(subject),
            content: Some(content),
            recipient: Some(recipient),
            ..MessageRequest::default()
        });
    }
}