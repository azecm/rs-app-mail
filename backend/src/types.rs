use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DownloadStruct {
    pub filename: String,
    pub email: String,
    pub user: String,
    pub temp: Option<usize>,
}

#[derive(Clone, Default, Debug)]
pub struct SessionStruct {
    pub idu: i32,
    pub channel_id: usize,
    pub current: String,
}

impl SessionStruct {
    pub fn new(idu: &i32) -> Self {
        Self {
            idu: *idu,
            channel_id: 0,
            current: "".to_string(),
        }
    }
    pub fn with_key(idu: &i32, channel_id: &usize, current: &str) -> Self {
        Self {
            idu: *idu,
            channel_id: *channel_id,
            current: current.to_string(),
        }
    }
}


