use headers::{AcceptRanges, ContentLength, ContentType, HeaderMapExt};
use serde::{Deserialize, Serialize};
use tokio::fs;
use warp::{reject, Rejection, Reply, reply};
use warp::hyper::Body;
use warp::multipart::FormData;
use warp::path::Tail;
use warp::reply::Response;

use shared::constants::TEST_USER_ID;
use shared::types::{MessageRequest, MessagesRequest, NotesChannel};

use crate::constants::{path_to_attachment_with_email_and_key, path_to_temp};
use crate::db_boxes::{db_message_route, db_messages_route};
use crate::db_notes::db_notes_route;
use crate::db_types::DBNotes;
use crate::db_user::{db_user_login, DBUserSelect};
use crate::sse::sse_next_key;
use crate::state::USER_AUTH;
use crate::types::{DownloadStruct, SessionStruct};
use crate::upload::upload;

#[derive(Serialize)]
struct InitialStruct {
    notes: Vec<DBNotes>,
    user: DBUserSelect,
}

fn get_session(user_key: &str) -> SessionStruct {
    match USER_AUTH.lock() {
        Ok(user_auth) => match user_auth.get(user_key) {
            Some(session) => session.clone(),
            None => SessionStruct::default()
        }
        Err(_) => SessionStruct::default()
    }
}

pub async fn route_message(user_key: String, msg: String) -> Result<impl Reply, Rejection> {
    let session = get_session(&user_key);
    if session.idu > 0 {
        if let Ok(data) = serde_json::from_str::<MessageRequest>(&msg) {
            db_message_route(&session, data).await;
        }
    }
    Ok(warp::reply())
}

pub async fn route_messages(user_key: String, msg: String) -> Result<impl Reply, Rejection> {
    let session = get_session(&user_key);
    if session.idu > 0 {
        if let Ok(data) = serde_json::from_str::<MessagesRequest>(&msg) {
            db_messages_route(&session, data).await;
        }
    }
    Ok(warp::reply())
}

pub async fn route_notes_update(user_key: String, msg: String) -> Result<impl Reply, Rejection> {
    let session = get_session(&user_key);
    if session.idu > 0 {
        if let Ok(data) = serde_json::from_str::<NotesChannel>(&msg) {
            db_notes_route(&session, data).await;
        }
    }
    Ok(warp::reply())
}

pub async fn route_login(hash: String, source: Vec<String>) -> Result<impl Reply, Rejection> {
    let mut idu = 0;

    if source.len() == 1 {
        let user_key = source[0].clone();
        let session = get_session(&user_key);
        idu = session.idu;
        if idu > 0 {
            if let Ok(mut user_auth) = USER_AUTH.lock() {
                user_auth.remove(&user_key);
            }
        } else if TEST_USER_ID > 0 {
            idu = TEST_USER_ID;
        }
    } else if source.len() == 3 {
        let mail_box = source[0]
            .split('!')
            .map(|t| t.split('+').collect::<Vec<_>>().join("."))
            .collect::<Vec<_>>().join("@");
        let user_name = source[1].clone();
        let user_pass = source[2].clone();
        idu = db_user_login(mail_box, user_name, user_pass).await;
    }
    if idu > 0 {
        if let Ok(mut user_auth) = USER_AUTH.lock() {
            user_auth.insert(hash, SessionStruct::new(&idu));
        }
    }

    Ok(reply::json(&LoginResult { result: idu > 0 }))
}

#[derive(Serialize, Deserialize)]
struct LoginResult {
    result: bool,
}

pub async fn files_handler(user_key: String, form: FormData) -> Result<impl Reply, Rejection> {
    let session = get_session(&user_key);
    if session.idu > 0 {
        upload(&session, form).await;
        sse_next_key(&session);
    }
    Ok(warp::reply())
}

pub async fn file_handler(
    tail: Tail,
    q: DownloadStruct,
) -> Result<Response, Rejection> {
    let session = get_session(&q.user);
    if session.idu > 0 {
        let is_temp = q.temp.unwrap_or_default() == 1;
        let filename = match is_temp {
            true => path_to_temp(tail.as_str()),
            false => path_to_attachment_with_email_and_key(&q.email, tail.as_str())
        };
        sse_next_key(&session);

        if let Ok(meta) = fs::metadata(&filename).await {
            if let Ok(body) = fs::read(&filename).await {
                let mut resp = Response::new(Body::from(body));
                resp.headers_mut().typed_insert(ContentLength(meta.len()));
                resp.headers_mut().typed_insert(ContentType::octet_stream());
                resp.headers_mut().typed_insert(AcceptRanges::bytes());
                return Ok(resp);
            }
        }
    }

    Err(reject())
}