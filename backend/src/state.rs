use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;

use crate::db_user::DBUserInit;
use crate::types::SessionStruct;

pub static USER_AUTH: Lazy<Arc<Mutex<HashMap<String, SessionStruct>>>> = Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub static USER_BY_EMAIL: Lazy<Arc<Mutex<HashMap<String, i32>>>> = Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub static USER_BY_ID: Lazy<Arc<Mutex<HashMap<i32, DBUserInit>>>> = Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));