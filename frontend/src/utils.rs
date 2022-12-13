use dominator::events;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use web_sys::{Document, Element, EventTarget, HtmlDocument, HtmlElement, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement, Location, Node, Selection, Window};

thread_local! {
    static WINDOW: Window = web_sys::window().unwrap_throw();
    static DOCUMENT: Document = WINDOW.with(|w| w.document().unwrap_throw());
    static HTML_DOCUMENT: HtmlDocument = WINDOW.with(|w| w.document().unwrap_throw().dyn_into::<HtmlDocument>().unwrap_throw());
}

pub fn _get_value_from_event(event: &events::Input) -> String {
    match event.target() {
        Some(target) => get_value_from_input(JsValue::from(target)),
        None => "".to_string()
    }
}

fn get_value_from_input(element: JsValue) -> String {
    if let Some(element) = element.dyn_ref::<HtmlInputElement>() {
        element.value()
    } else if let Some(element) = element.dyn_ref::<HtmlTextAreaElement>() {
        element.value()
    } else if let Some(element) = element.dyn_ref::<HtmlSelectElement>() {
        element.value()
    } else {
        "".to_string()
    }
}


pub fn get_location() -> Location {
    WINDOW.with(|w| w.location())
}

pub fn set_title(text: &str) {
    DOCUMENT.with(|d| d.set_title(text));
}

pub fn create_element(node_name: &str) -> Element {
    DOCUMENT.with(|d| d.create_element(node_name).unwrap_throw())
}

pub fn get_value_by_query(selectors: &str) -> String {
    match query_selector(selectors) {
        Some(element) => {
            get_value_from_input(JsValue::from(element))
        }
        None => { "".to_string() }
    }
}

pub fn get_input_value(name: &str) -> String {
    get_value_by_query(&format!("[name={name}]"))
}

pub fn query_selector(selectors: &str) -> Option<Element> {
    DOCUMENT.with(|d| d.query_selector(selectors).unwrap_throw())
}

pub fn query_selector_all(selectors: &str) -> Vec<HtmlElement> {
    DOCUMENT.with(|d| {
        let mut list: Vec<HtmlElement> = vec![];
        let node_list = d.query_selector_all(selectors).unwrap_throw();
        for ind in 0..node_list.length() {
            if let Some(html_elem) = get_html_element(get_element_from_node(node_list.get(ind))) {
                list.push(html_elem);
            }
        }
        list
    })
}

pub fn get_html_element(el: Option<Element>) -> Option<HtmlElement> {
    match el {
        Some(el) => match el.dyn_into::<HtmlElement>() {
            Ok(el) => Some(el),
            Err(_) => None
        }
        None => None
    }
}

pub fn get_element_from_node(el: Option<Node>) -> Option<Element> {
    match el {
        Some(el) => match el.dyn_into::<Element>() {
            Ok(el) => Some(el),
            Err(_) => None
        }
        None => None
    }
}

pub fn exec_command(command_id: &str) -> bool {
    HTML_DOCUMENT.with(|d| d.exec_command(command_id).unwrap_throw())
}

pub fn exec_command_full(command_id: &str, show_ui: bool, value: &str) -> bool {
    HTML_DOCUMENT.with(|d| d.exec_command_with_show_ui_and_value(command_id, show_ui, value).unwrap_throw())
}

pub fn attr_data(key: &str) -> String {
    format!("data-{key}")
}

pub fn from_dataset(target: Option<EventTarget>, key: &str) -> String {
    match get_element_from_target(target) {
        Some(element) => {
            match element.dataset().get(key) {
                Some(value) => value,
                None => "".to_string()
            }
        }
        None => "".to_string()
    }
}

pub fn get_element_from_target(target: Option<EventTarget>) -> Option<HtmlElement> {
    match target {
        Some(target) => {
            match JsValue::from(target).dyn_ref::<HtmlElement>() {
                Some(element) => Some(element.clone()),
                None => None
            }
        }
        None => None
    }
}

pub fn get_selection() -> Option<Selection> {
    WINDOW.with(|w| w.get_selection().unwrap_throw())
}

pub fn location_reload() {
    WINDOW.with(|w| {
        match w.location().reload() {
            Ok(_) => {}
            Err(_) => {}
        }
    });
}

pub fn node_parent(node: Node, node_name: &str) -> Option<Node> {
    let node_name = node_name.to_uppercase();
    let mut node = Some(node);
    while let Some(current) = node {
        if current.node_name() == node_name {
            return Some(current);
        }
        node = current.parent_node();
    }
    None
}

pub fn drop_element(elem: Element) {
    match elem.parent_node() {
        Some(parent) => {
            while let Some(child) = elem.first_child() {
                if let Ok(_) = parent.insert_before(&child, Some(&elem)) {}
            }
            elem.remove();
        }
        None => {}
    };
}

pub fn view_email(label: &str, email: &str) -> String {
    let email = if email.contains("@") { email } else { "" };
    if email.is_empty() {
        "".to_string()
    } else if label.is_empty() {
        email.to_string()
    } else {
        format!("{label} <{email}>")
    }
}

pub fn obj_to_string<T>(data: &T) -> String
    where T: serde::Serialize
{
    if let Ok(data) = serde_wasm_bindgen::to_value(data) {
        if let Ok(text) = js_sys::JSON::stringify(&data) {
            return String::from(text);
        }
    }
    "".to_string()
}


/*
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "setTimeout", catch)]
    pub fn set_timeout(handler: &Function, timeout: i32) -> Result<i32, JsValue>;

    #[wasm_bindgen(js_name = "clearTimeout")]
    pub fn clear_timeout(handle: i32);

    //#[wasm_bindgen(js_name = "getSelection")]
    //pub fn get_selection() -> Result<Option<Selection>, JsValue>;
    // pub fn get_selection(this: &Window) -> Result<Option<Selection>, JsValue>;

    //#[wasm_bindgen(js_name = "setInterval", catch)]
    //pub fn set_interval(handler: &Function, timeout: i32) -> Result<i32, JsValue>;

    //#[wasm_bindgen(js_name = "clearInterval")]
    //pub fn clear_interval(handle: i32);
}
*/