use crate::elements::app_login::app_state;

mod elements;
mod utils;
mod constants;
mod state;
mod notes;
mod types;
mod dialog;
mod connect_sse;
mod connect_fetch;
mod editor;
pub mod loader;
mod connect_files;

pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    dominator::append_dom(&dominator::body(), app_state());
}
