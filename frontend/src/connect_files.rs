use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, FormData, ProgressEvent, XmlHttpRequest};

use shared::constants::{API_FILES, HEADER_USER_KEY, ROOT_API};

use crate::state::USER_KEY;

pub fn connect_files(form: &FormData, progress: fn(u32), full: fn(bool)) {
    let user_key = USER_KEY.get_cloned();
    if user_key.is_empty() {
        return;
    }
    if let Ok(xhr) = XmlHttpRequest::new() {
        if let Err(_) = xhr.open_with_async("POST", &format!("/{ROOT_API}/{API_FILES}/"), true) {
            return;
        }
        if let Err(_) = xhr.set_request_header(&HEADER_USER_KEY, &user_key) {}

        if let Ok(upload) = xhr.upload() {
            let onprogress_callback = Closure::<dyn FnMut(_)>::new(move |e: ProgressEvent| {
                let value = (e.loaded() / e.total() * 1000.0).round() as u32;
                progress(value);
            });
            upload.set_onprogress(Some(onprogress_callback.as_ref().unchecked_ref()));
            onprogress_callback.forget();
        }

        if let Err(_) = xhr.send_with_opt_form_data(Some(&form)) {}

        let onerror_callback = Closure::<dyn FnMut(_)>::new(move |_e: ErrorEvent| {
            log::error!("[connect_files]");
            full(false);
        });
        xhr.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        let xhr_load = xhr.clone();
        let onload_callback = Closure::<dyn FnMut()>::new(move || {
            if let Ok(status) = xhr_load.status() {
                if status > 199 && status < 300 {
                    full(true);
                } else {
                    full(false);
                }
            }
        });
        xhr.set_onload(Some(onload_callback.as_ref().unchecked_ref()));
        onload_callback.forget();
    }
}