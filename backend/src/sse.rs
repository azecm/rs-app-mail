use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering}, Mutex,
};

use futures_util::{Stream, StreamExt};
use once_cell::sync::Lazy;
use serde::Serialize;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::sse::Event;

use shared::constants::{CHANNEL_INIT, CHANNEL_MESSAGE, CHANNEL_MESSAGES, CHANNEL_NOTES, CHANNEL_USER_KEY};
use shared::types::MessagesRequest;

use crate::db_boxes::db_messages_route;
use crate::db_notes::db_notes_select;
use crate::db_types::DBNotes;
use crate::db_user::{db_user_select, DBUserSelect};
use crate::state::USER_AUTH;
use crate::types::SessionStruct;
use crate::utils::get_hash;

static NEXT_CHANNEL_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Clone)]
pub enum Message {
    Reply(String),
    Notes(String),
    Messages(String),
    Message(String),
    Init(String),
    User(String),
}

pub struct Client {
    pub idu: i32,
    pub sender: mpsc::UnboundedSender<Message>,
}

impl Client {
    pub fn new(idu: i32, sender: mpsc::UnboundedSender<Message>) -> Self {
        Self {
            idu,
            sender,
        }
    }
}

type UsersSse = Arc<Mutex<HashMap<usize, Client>>>;

static USERS_SSE: Lazy<UsersSse> = Lazy::new(|| {
    UsersSse::default()
});

pub fn user_sse_connected(idu: i32) -> impl Stream<Item=Result<Event, warp::Error>> + Send + 'static {
    let channel_id = NEXT_CHANNEL_ID.fetch_add(1, Ordering::Relaxed);

    let (tx, rx) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);

    tx.send(Message::Reply("".to_string())).unwrap();
    USERS_SSE.lock().unwrap().insert(channel_id, Client::new(idu.clone(), tx));

    init_data(SessionStruct::with_key(&idu, &channel_id, ""));

    rx.map(|msg| match msg {
        Message::Notes(reply) => {
            Ok(Event::default().event(CHANNEL_NOTES).data(reply))
        }
        Message::Messages(reply) => {
            Ok(Event::default().event(CHANNEL_MESSAGES).data(reply))
        }
        Message::Message(reply) => {
            Ok(Event::default().event(CHANNEL_MESSAGE).data(reply))
        }
        Message::Init(reply) => {
            Ok(Event::default().event(CHANNEL_INIT).data(reply))
        }
        Message::User(reply) => {
            Ok(Event::default().event(CHANNEL_USER_KEY).data(reply))
        }
        Message::Reply(reply) => {
            Ok(Event::default().data(reply))
        }
    })
}

pub fn sse_cleaner() {
    let users_len = match USER_AUTH.lock() {
        Ok(users) => users.len(),
        Err(_) => 0
    };
    match USERS_SSE.lock() {
        Ok(mut sse) => {
            tracing::info!("sse_cleaner::start {}", sse.len());
            sse.retain(|_uid, client| {
                client.sender.send(Message::Reply("".to_string())).is_ok()
            });
            tracing::info!("sse_cleaner::end {} -- USER_AUTH: {users_len}", sse.len());
        }
        Err(err) => tracing::error!("sse_channel: {:?}", err)
    }
}

pub fn sse_channel(session: &SessionStruct, msg: Message) {
    let send_to = session.idu.clone();
    match USERS_SSE.lock() {
        Ok(mut sse) => {
            sse.retain(|_uid, client| {
                if send_to == client.idu {
                    client.sender.send(msg.clone()).is_ok()
                } else {
                    true
                }
            });
        }
        Err(err) => tracing::error!("sse_channel: {:?}", err)
    }

    if session.channel_id > 0 {
        sse_next_key(session);
    }
}

pub fn sse_next_key(session: &SessionStruct) {
    let key = get_hash(format!("{}-{}", session.idu, Uuid::new_v4().to_string()));
    if let Ok(mut user_auth) = USER_AUTH.lock() {
        if sse_personal(&session, Message::User(key.clone())) {
            user_auth.insert(key.clone(), SessionStruct::with_key(&session.idu, &session.channel_id, &key));
            user_auth.remove(&session.current);
        } else {
            tracing::warn!("key not delivered");
        }
    }
}

pub fn sse_personal_channel(session: &SessionStruct, msg: Message) {
    sse_personal(&session, msg);
    sse_next_key(&session);
}

fn sse_personal(session: &SessionStruct, msg: Message) -> bool {
    let send_to = &session.channel_id;
    let mut delivered = false;
    match USERS_SSE.lock() {
        Ok(mut sse) => {
            sse.retain(|channel_id, client| {
                if send_to == channel_id {
                    match client.sender.send(msg.clone()) {
                        Ok(_) => {
                            delivered = true;
                            true
                        }
                        Err(err) => {
                            tracing::error!("sse_personal {:?}", err);
                            false
                        }
                    }
                } else {
                    true
                }
            });
        }
        Err(err) => tracing::error!("sse_personal: {:?}", err)
    }
    delivered
}


#[derive(Serialize)]
struct InitialStruct {
    notes: Vec<DBNotes>,
    user: DBUserSelect,
}

fn init_data(session: SessionStruct) {
    tokio::task::spawn(async move {
        db_messages_route(&session, MessagesRequest { page: 0, email_box: 0 }).await;
        let data = InitialStruct {
            notes: db_notes_select(&session.idu).await,
            user: db_user_select(&session.idu).await,
        };
        match serde_json::to_string(&data) {
            Ok(text) => {
                sse_personal(&session, Message::Init(text));
                sse_next_key(&session);
            }
            Err(err) => {
                tracing::error!("serde_json[init_data] {:?}", err);
            }
        }
    });
}
