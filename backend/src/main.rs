use std::convert::Infallible;

use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{Filter, http, Rejection, Reply};
use warp::http::StatusCode;
use warp::reject::Reject;

use shared::constants::{API_EVENT, API_FILE, API_FILES, API_LOGIN, API_NOTES, CHANNEL_MESSAGE, CHANNEL_MESSAGES, HEADER_USER_KEY, ROOT_API};
use state::USER_AUTH;

use crate::constants::test_dirs;
use crate::db::db_conn;
use crate::db_types::DBNotes;
use crate::db_user::db_user_init;
use crate::filters::{with_body_filter, with_hash};
use crate::receive::mail_watcher;
use crate::routes::{file_handler, files_handler, route_login, route_message, route_messages, route_notes_update};
use crate::sse::user_sse_connected;
use crate::tasks::run_tasks;
use crate::types::DownloadStruct;

mod db;
mod db_types;
pub mod db_notes;
mod sse;
mod routes;
mod db_user;
mod types;
mod filters;
mod utils;
mod state;
mod db_boxes;
mod constants;
mod upload;
mod receive;
mod send;
mod tasks;

#[tokio::main(worker_threads = 2)]
async fn main() {
    tokio::task::spawn(async {
        db_user_init().await;
        test_dirs();
        mail_watcher().await;
        //if let Err(_) = mail_watcher_() {}
    });

    tokio::task::spawn(async {
        run_tasks().await;
    });

    let with_ansi = cfg!(target_os = "macos");

    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_ansi(with_ansi)
        .with_max_level(Level::INFO)
        .with_thread_ids(true)
        .with_target(false)
        .with_span_events(FmtSpan::CLOSE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let user_login = warp::path(API_LOGIN)
        .and(with_hash())
        .and(warp::body::content_length_limit(512))
        .and(warp::body::json())
        .and_then(route_login);

    let message_filter = warp::path(CHANNEL_MESSAGE)
        .and(warp::body::content_length_limit(1024 * 1024))
        .and(warp::header::<String>(HEADER_USER_KEY))
        .and(with_body_filter())
        .and_then(route_message);

    let messages_filter = warp::path(CHANNEL_MESSAGES)
        .and(warp::body::content_length_limit(1024 * 100))
        .and(warp::header::<String>(HEADER_USER_KEY))
        .and(with_body_filter())
        .and_then(route_messages);

    let notes_filter = warp::path(API_NOTES)
        .and(warp::body::content_length_limit(1024 * 100))
        .and(warp::header::<String>(HEADER_USER_KEY))
        .and(with_body_filter())
        .and_then(route_notes_update);

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["*"])
        .allow_methods(vec!["GET"]);

    let file_filter = warp::path(ROOT_API)
        .and(warp::path(API_FILE))
        .and(warp::path::tail())
        .and(warp::query::<DownloadStruct>())
        .and_then(file_handler)
        ;

    let files_filter = warp::path(API_FILES)
        .and(warp::header::<String>(HEADER_USER_KEY))
        .and(warp::multipart::form().max_length(50 * 1024 * 1024))
        .and_then(files_handler)
        ;

    let event = warp::path(ROOT_API)
        .and(warp::path(API_EVENT))
        .and(with_hash()
            .and_then(|hash: String| async move {
                let res = if let Ok(mut user_auth) = USER_AUTH.lock() {
                    if let Some(session) = user_auth.remove(&hash) {
                        Ok(session.idu)
                    } else { Err(()) }
                } else { Err(()) };
                res.map_err(|_| warp::reject::custom(NotUtf8))
            })
        )
        .map(|idu: i32| {
            let stream = user_sse_connected(idu);
            warp::sse::reply(warp::sse::keep_alive().stream(stream))
        });

    let routes_dir = warp::fs::dir("/Users/mac-user/Documents/development/rs-app-mail/frontend/dist");

    let routes = warp::get()
        .and(event.or(file_filter).or(routes_dir))
        .or(
            warp::post().and(
                warp::path(ROOT_API)
                    .and(notes_filter.or(files_filter).or(message_filter).or(messages_filter).or(user_login))
            )
        );

    warp::serve(
        routes
            .recover(handle_rejection)
            .with(warp::trace::request()).with(cors),
    )
        .run(([127, 0, 0, 1], 3031))
        .await;
}


#[tracing::instrument]
async fn handle_rejection(_err: Rejection) -> Result<impl Reply, Infallible> {
    Ok(http::Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body("not found"))
}

#[derive(Debug)]
pub struct AppErr {
    pub reason: String,
}

impl Reject for AppErr {}

#[derive(Debug)]
struct NotUtf8;

impl Reject for NotUtf8 {}