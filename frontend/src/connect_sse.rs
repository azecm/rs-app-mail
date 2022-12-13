use std::fmt::Debug;

use gloo_timers::callback::Timeout;
use serde::de::DeserializeOwned;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{ErrorEvent, EventSource, MessageEvent};

use shared::constants::{API_EVENT, CHANNEL_INIT, CHANNEL_MESSAGE, CHANNEL_MESSAGES, CHANNEL_NOTES, CHANNEL_USER_KEY, ROOT_API};

use crate::elements::app_login::login_after_error;
use crate::elements::app_message::{message_channel, messages_channel};
use crate::loader::{init_channel, notes_channel, user_channel};

#[wasm_bindgen]
pub fn start_sse() -> Result<(), JsValue> {
    let sse: EventSource = EventSource::new(&format!("/{ROOT_API}/{API_EVENT}"))?;

    let onopen_callback = Closure::<dyn FnMut()>::new(move || {
        log::info!("[SSE] opened");
    });
    sse.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();


    let onerror_callback = Closure::<dyn FnMut(_)>::new(move |_e: ErrorEvent| {
        handle_sse_error();
    });
    sse.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    sse_data_event_channel(&sse, CHANNEL_MESSAGES, messages_channel);
    sse_data_event_channel(&sse, CHANNEL_MESSAGE, message_channel);
    sse_data_event_channel(&sse, CHANNEL_NOTES, notes_channel);
    sse_data_event_channel(&sse, CHANNEL_INIT, init_channel);
    sse_text_event_channel(&sse, CHANNEL_USER_KEY, user_channel);

    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        if let Ok(_txt) = e.data().dyn_into::<js_sys::JsString>() {
            log::info!("[SSE] received Text: {:?}", _txt);
        } else {
            log::info!("[SSE] received Unknown: {:?}", e.data());
        }
    });
    sse.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();

    Ok(())
}
// let timeout = Timeout::new(300, || edit_group(event));

fn handle_sse_error() {
    log::info!("[SSE] error event");
    let timer = Timeout::new(1000, login_after_error);
    timer.forget();
}


fn sse_data_event_channel<T>(sse: &EventSource, event_name: &str, call: fn(T))
    where
        T: DeserializeOwned + Debug + 'static
{
    let notes_listener_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            match from_json::<T>(txt.clone()) {
                Some(data) => {
                    call(data);
                }
                None => {
                    log::error!("sse_data_event_channel[1] {txt} ");
                }
            };
        } else {
            log::error!("++");
        }
    });
    if let Err(err) = sse.add_event_listener_with_event_listener(event_name, notes_listener_callback.as_ref().unchecked_ref()) {
        log::error!("add_event_listener: {:?}", err);
    }
    notes_listener_callback.forget();
}

fn sse_text_event_channel(sse: &EventSource, event_name: &str, call: fn(String)) {
    let notes_listener_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            let text: String = txt.into();
            call(text);
        }
    });
    if let Err(err) = sse.add_event_listener_with_event_listener(event_name, notes_listener_callback.as_ref().unchecked_ref()) {
        log::error!("add_event_listener: {:?}", err);
    }
    notes_listener_callback.forget();
}

fn from_json<T>(text: js_sys::JsString) -> Option<T>
    where
        T: DeserializeOwned, {
    let txt: String = text.into();
    match js_sys::JSON::parse(&txt) {
        Ok(data) => {
            match serde_wasm_bindgen::from_value::<T>(data) {
                Ok(data) => {
                    Some(data)
                }
                Err(_) => None
            }
        }
        Err(_) => None
    }
}
