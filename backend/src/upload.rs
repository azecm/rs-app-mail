use bytes::Buf;
use futures_util::TryStreamExt;
use tokio::fs;
use uuid::Uuid;
use warp::multipart::{FormData, Part};

use shared::types::{BoxMailAttachmentItem, BoxMailAttachments, MessageRequest};

use crate::constants::{path_to_temp_upload, path_to_temp_with_ind};
use crate::sse::{Message, sse_personal_channel};
use crate::types::SessionStruct;

pub async fn upload(session: &SessionStruct, form: FormData) {
    let parts: Result<Vec<Part>, ()> = form.try_collect().await.map_err(|e| {
        tracing::warn!("files_handler: {}", e);
    });
    if let Ok(parts) = parts {
        let mut files: Vec<(String, String)> = vec![];
        let mut current = "".to_string();
        for mut p in parts {
            match p.name() {
                "current" => {
                    current = part_as_string(&mut p).await;
                }
                "files" => {
                    let file_name_temp = part_as_file(&mut p).await;
                    let file_name = p.filename().unwrap_or_default();
                    if !file_name_temp.is_empty() && !file_name.is_empty() {
                        files.push((file_name_temp, file_name.to_string()));
                    }
                }
                _ => {}
            }
        }
        if !files.is_empty() {
            if let Ok(attachments) = serde_json::from_str::<BoxMailAttachments>(&current) {
                let key = attachments.key.clone();
                let mut list = attachments.list.clone();
                let mut ind = if let Some(v) = list.iter().map(|r| r.id).max() {
                    v
                } else { 0 };
                ind += 1;
                for (file_name_temp, file_name) in files.iter() {
                    if let Ok(metadata) = fs::metadata(&path_to_temp_upload(file_name_temp)).await {
                        if (fs::rename(&path_to_temp_upload(file_name_temp), &path_to_temp_with_ind(&key, &ind)).await).is_ok() {
                            list.push(BoxMailAttachmentItem { file_name: file_name.to_string(), id: ind, size: metadata.len() });
                            ind += 1;
                        }
                    }
                }
                let attachments = Some(BoxMailAttachments { key, list });
                let data = MessageRequest { idb: 0, attachments, ..MessageRequest::default() };
                if let Ok(text) = serde_json::to_string(&data) {
                    sse_personal_channel(session, Message::Message(text));
                }
            }
        }
    }
}

async fn part_as_string(p: &mut Part) -> String {
    if let Some(Ok(v1)) = p.data().await {
        let v2 = v1.chunk();
        return String::from_utf8_lossy(v2).to_string();
    }
    "".to_string()
}

async fn part_as_file(p: &mut Part) -> String {
    let fine_name = Uuid::new_v4().to_string();
    if (fs::create_dir_all(&path_to_temp_upload("")).await).is_ok() {
        if let Some(Ok(e)) = p.data().await {
            if (fs::write(&path_to_temp_upload(&fine_name), &e.chunk()).await).is_ok() {
                return fine_name;
            }
        }
    }
    "".to_string()
}