use dominator::{Dom, events, html};
use futures_signals::signal::{Mutable, SignalExt};
use once_cell::sync::Lazy;
use serde::Deserialize;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::Window;

use shared::constants::{API_LOGIN, TEST_USER_ID};

use crate::connect_fetch::connect_json_data;
use crate::connect_sse::start_sse;
use crate::constants::{PROP_NAME, PROP_PLACEHOLDER, PROP_TITLE, TAG_DIV, TAG_INPUT};
use crate::elements::app_root::app_root;
use crate::state::USER_KEY;
use crate::utils::{get_html_element, get_input_value, get_location, query_selector, set_title};

const KEY_ENTER: &str = "Enter";
const FIELD_NAME: &str = "a";
const FIELD_PASS: &str = "b";

const AUTH_STATE_DEFAULT: usize = 1;
const AUTH_STATE_LOGIN: usize = 2;
const AUTH_STATE_AUTHORIZED: usize = 3;

static AUTH_ERR: Lazy<Mutable<bool>> = Lazy::new(|| Mutable::new(false));
static AUTH_STATE: Lazy<Mutable<usize>> = Lazy::new(|| Mutable::new(0));

fn css_class(label: &str) -> String {
    format!("app-login__{label}")
}

fn init_storage() {
    if let Some(w) = web_sys::window() {
        let mail_box = get_user_box();
        if let Ok(Some(local_storage)) = w.local_storage() {
            if let Ok(Some(value)) = local_storage.get_item(&mail_box) {
                connect_json_data(API_LOGIN, vec![value], login_connect_result);
                local_storage.remove_item(&mail_box).ok();
            } else {
                if TEST_USER_ID > 0 {
                    connect_json_data(API_LOGIN, vec![""], login_connect_result);
                }
                AUTH_STATE.set(AUTH_STATE_LOGIN);
            }
        }

        let w1: Window = w.clone();
        let listener_callback = Closure::<dyn FnMut()>::new(move || {
            if let Ok(Some(local_storage)) = w1.local_storage() {
                let key = USER_KEY.get_cloned();
                if !key.is_empty() {
                    local_storage.set_item(&mail_box, &key).ok();
                }
            }
        });

        let is_iphone:bool = match w.navigator().user_agent() {
            Ok(ua)=>ua.contains("iPhone"),
            Err(_)=>false
        };

        if is_iphone {
            w.set_onpagehide(Some(listener_callback.as_ref().unchecked_ref()));
        }
        else{
            w.set_onbeforeunload(Some(listener_callback.as_ref().unchecked_ref()));
        }

        listener_callback.forget();
    }
}

pub fn app_state() -> Dom {
    if get_user_box().is_empty() {
        AUTH_STATE.set(AUTH_STATE_DEFAULT);
    } else {
        init_storage();
    }
    html!(TAG_DIV, {
        .child_signal(AUTH_STATE.signal().map(|state|{
            match state {
                AUTH_STATE_DEFAULT => Some(default_page()),
                AUTH_STATE_LOGIN => Some(login_page()),
                AUTH_STATE_AUTHORIZED => Some(login_after()),
                _ => None
            }
        }))
    })
}

fn default_page() -> Dom {
    set_title("Здравствуйте!");
    html!(TAG_DIV, {
        .class(css_class("container"))
        .child(html!("p", {
            .text("Сайт Почты России")
        }))
    })
}

fn login_page() -> Dom {
    set_title("Авторизация");
    html!(TAG_DIV, {
        .class(css_class("container"))
        .child(html!(TAG_DIV, {
            .class(css_class("form"))
            .children([
                html!(TAG_INPUT, {
                    .class(css_class("input"))
                    .attr(PROP_TITLE, "Имя")
                    .attr(PROP_PLACEHOLDER, "Имя")
                    .attr(PROP_NAME, FIELD_NAME)
                    .event(handle_key_name)
                }),
                html!(TAG_INPUT, {
                    .class(css_class("input"))
                    .attr(PROP_TITLE, "Пароль")
                    .attr(PROP_PLACEHOLDER, "Пароль")
                    .attr(PROP_NAME, FIELD_PASS)
                    .event(handle_key_pass)
                }),
                html!(TAG_DIV, {
                    .class(css_class("message"))
                    .text_signal(AUTH_ERR.signal().map(|flag| if flag {"...ошибка авторизации..."} else {""}))
                }),
            ])
        }))
    })
}

fn handle_key_name(ev: events::KeyDown) {
    if ev.key() == KEY_ENTER {
        if let Some(elem) = get_html_element(query_selector(&format!("[name={FIELD_PASS}]"))) {
            if elem.focus().is_ok() {}
        }
    }
}

fn handle_key_pass(ev: events::KeyDown) {
    if ev.key() == KEY_ENTER {
        login_connect();
    }
}

fn login_connect() {
    let user_name = get_input_value(FIELD_NAME).trim().to_string();
    let user_pass = get_input_value(FIELD_PASS).trim().to_string();
    let source = get_user_box().split('@').map(|row| row.split('.')
        .collect::<Vec<_>>().join("+")).collect::<Vec<_>>().join("!");
    connect_json_data(API_LOGIN, vec![source, user_name, user_pass], login_connect_result);
}

#[derive(Deserialize)]
struct LoginResult {
    result: bool,
}

pub fn login_after_error() {
    log::info!("login_after_error");
    let key = USER_KEY.get_cloned();
    connect_json_data(API_LOGIN, vec![key], login_connect_result);
}

fn login_connect_result(data: LoginResult) {
    log::info!("login_connect_result: {}", data.result);
    if data.result {
        AUTH_STATE.set(AUTH_STATE_AUTHORIZED);
    } else {
        AUTH_ERR.set_neq(true);
        AUTH_STATE.set(AUTH_STATE_LOGIN);
    }
}

fn login_after() -> Dom {
    let title = format!("Почта для {}", get_user_box().split('@').collect::<Vec<_>>().join(" на "));
    set_title(&title);
    if let Err(err) = start_sse() {
        log::error!("sse connection error {:?}", err);
    }
    app_root()
}

pub fn get_user_box() -> String {
    get_location().map(|l|l.pathname().map_or(String::new(), |pathname|{
        let pathname: String = pathname;
        let mail_box = pathname.split('/').collect::<Vec<_>>()[1];
        if test_email(mail_box) {
            mail_box.to_string()
        }
        else {
            String::new()
        }
    })).unwrap_or_default()
}

fn test_email(email: &str) -> bool {
    let re: js_sys::RegExp = js_sys::RegExp::new("[a-z]+@[a-z-]+\\.[a-z]+", "");
    re.test(email)
}
