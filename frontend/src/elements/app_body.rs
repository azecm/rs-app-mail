use dominator::{Dom, events, html};

use shared::types::{MailBoxes, MessagesRequest};
use shared::utils::box_type_index;

use crate::constants::TAG_DIV;
use crate::elements::app_message::{box_view, message_content};
use crate::loader::messages_load;
use crate::notes::app_notes::app_notes;
use crate::notes::notes_content::notes_content;
use crate::notes::notes_events::events_content;
use crate::state::{BOX_STATE, CURRENT_BOX, LOADING_NEXT};
use crate::utils::get_element_from_target;

fn css_class(label: &str) -> String {
    format!("app-body__{label}")
}

pub fn app_body() -> Dom {
    html!(TAG_DIV,{
        .class(css_class("container"))
        .children([
            html!(TAG_DIV, {
                .attr("id","col-1")
                .class(css_class("column-1"))
                .children([
                    message_content(),
                    notes_content(),
                    events_content()
                ])
            }),
            html!(TAG_DIV, {
                .attr("id","col-2")
                .class(css_class("column-2"))
                .event(handle_scroll)
                .children([
                    app_notes(),
                    box_view(MailBoxes::Inbox),
                    box_view(MailBoxes::Ready),
                    box_view(MailBoxes::Sent),
                    box_view(MailBoxes::Trash),
                ])
            })
        ])
    })
}

fn handle_scroll(e: events::Scroll) {
    if let Some(element) = get_element_from_target(e.target()) {
        let scroll_max = element.scroll_height() - element.offset_height();
        if scroll_max - element.scroll_top() < 10 && !LOADING_NEXT.get() {
            LOADING_NEXT.set(true);
            let mb = CURRENT_BOX.get();
            let box_index = box_type_index(&mb);
            if !BOX_STATE[box_index].fully_loaded.get() {
                let page = BOX_STATE[box_index].page.get() + 1;
                messages_load(MessagesRequest { page, email_box: box_index as i32 });
            }
        }
    }
}
