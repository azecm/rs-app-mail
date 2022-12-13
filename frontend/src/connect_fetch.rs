use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{Request, RequestInit, Response};

use shared::constants::{HEADER_USER_KEY, ROOT_API};

use crate::state::USER_KEY;

pub fn connect_json_send<T: serde::Serialize>(url: &str, data: T) {
    if USER_KEY.get_cloned().is_empty() {
        return;
    }
    match serde_wasm_bindgen::to_value(&data) {
        Ok(data) => {
            let url = url.to_string();
            spawn_local(async move {
                if let Err(_) = send(&url, Some(data)).await {};
            });
        }
        Err(err) => {
            log::error!("connect_json_send: {:?}", err);
        }
    }
}

pub fn connect_json_data<R, T>(url: &str, data: T, result: fn(R))
    where
        T: serde::Serialize,
        R: serde::de::DeserializeOwned + 'static
{
    match serde_wasm_bindgen::to_value(&data) {
        Ok(data) => {
            connect_post_json(url, Some(data), Some(result));
        }
        Err(err) => {
            log::error!("connect_json_data: {:?}", err);
        }
    }
}

fn connect_post_json<R>(url: &str, data: Option<JsValue>, result: Option<fn(R)>)
    where
        R: serde::de::DeserializeOwned + 'static
{
    let url = url.to_string();
    spawn_local(async move {
        match send(&url, data).await {
            Ok(data) => {
                if let Ok(data) = serde_wasm_bindgen::from_value::<R>(data) {
                    if let Some(result) = result {
                        result(data);
                    }
                }
            }
            Err(_) => {}
        };
    });
}

async fn send(url: &str, data: Option<JsValue>) -> Result<JsValue, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("POST");
    //opts.mode(RequestMode::Cors);
    opts.credentials(web_sys::RequestCredentials::Include);
    if let Some(data) = data {
        if let Ok(data) = js_sys::JSON::stringify(&data) {
            opts.body(Some(&data));
        }
    }

    let request = Request::new_with_str_and_init(&format!("/{ROOT_API}/{url}"), &opts)?;
    request.headers().set("Content-Type", "application/json")?;
    let user_key = USER_KEY.get_cloned();
    if !user_key.is_empty() {
        request.headers().set(&HEADER_USER_KEY, &user_key)?;
    }

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().unwrap();

    let json = JsFuture::from(resp.json()?).await?;

    Ok(json)
}
