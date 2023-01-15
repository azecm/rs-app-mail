use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use web_sys::{Document, Element, EventTarget, HtmlDocument, HtmlElement, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement, Location, Node, Selection, Window};

fn get_window() -> Option<Window> {
    web_sys::window()
}

fn get_document() -> Option<Document> {
    get_window().and_then(|w| w.document())
}

fn get_html_document() -> Option<HtmlDocument> {
    get_document().and_then(|d|d.dyn_into::<HtmlDocument>().ok())
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


pub fn get_location() -> Option<Location> {
    get_window().map(|w|w.location())
}

pub fn set_title(text: &str) {
    if let Some(d) = get_document() {
        d.set_title(text);
    }
}

pub fn create_element(node_name: &str) -> Option<Element> {
    get_document().and_then(|d| d.create_element(node_name).ok())
}

pub fn get_value_by_query(selectors: &str) -> String {
    query_selector(selectors)
        .map(|element|get_value_from_input(JsValue::from(element)))
        .unwrap_or_default()
}

pub fn get_input_value(name: &str) -> String {
    get_value_by_query(&format!("[name={name}]"))
}

pub fn query_selector(selectors: &str) -> Option<Element> {
    get_document().and_then(|d| d.query_selector(selectors).ok()).and_then(|e|e)
}

pub fn query_selector_all(selectors: &str) -> Vec<HtmlElement> {
    let mut list: Vec<HtmlElement> = Vec::new();
    if let Some(d) = get_document() {
        let node_list = d.query_selector_all(selectors).unwrap_throw();
        for ind in 0..node_list.length() {
            if let Some(html_elem) = get_html_element(get_element_from_node(node_list.get(ind))) {
                list.push(html_elem);
            }
        }
    }
    list
}

pub fn get_html_element(el: Option<Element>) -> Option<HtmlElement> {
    el.map(|el| el.dyn_into::<HtmlElement>().ok()).and_then(|el| el)
}

pub fn get_element_from_node(el: Option<Node>) -> Option<Element> {
    el.map(|el| el.dyn_into::<Element>().ok()).and_then(|el| el)
}

pub fn exec_command(command_id: &str) -> bool {
    get_html_document().and_then(|d|d.exec_command(command_id).ok()).unwrap_or_default()
}

pub fn exec_command_full(command_id: &str, show_ui: bool, value: &str) -> bool {
    get_html_document().and_then(|d|d.exec_command_with_show_ui_and_value(command_id, show_ui, value).ok()).unwrap_or_default()
}

pub fn attr_data(key: &str) -> String {
    format!("data-{key}")
}

pub fn from_dataset(target: Option<EventTarget>, key: &str) -> String {
    get_element_from_target(target)
        .and_then(|element|element.dataset().get(key))
        .unwrap_or_default()
}

pub fn get_element_from_target(target: Option<EventTarget>) -> Option<HtmlElement> {
    target
        .map(|target| JsValue::from(target).dyn_ref::<HtmlElement>().cloned())
        .and_then(|t| t)
}

pub fn get_selection() -> Option<Selection> {
    get_window().and_then(|w| w.get_selection().ok()).and_then(|s|s)
}

pub fn location_reload() {
    if let Some(w) = get_window() {
        w.location().reload().ok();
    }
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
    if let Some(parent) = elem.parent_node() {
        while let Some(child) = elem.first_child() {
            if parent.insert_before(&child, Some(&elem)).is_ok() {}
        }
        elem.remove();
    };
}

pub fn view_email(label: &str, email: &str) -> String {
    let email = if email.contains('@') { email } else { "" };
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
    serde_wasm_bindgen::to_value(data).ok()
        .and_then(|val|js_sys::JSON::stringify(&val).ok())
        .map(String::from)
        .unwrap_or_default()
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