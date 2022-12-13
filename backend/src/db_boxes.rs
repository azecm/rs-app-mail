use postgres_types::ToSql;
use tokio::fs;
use uuid::Uuid;

use shared::constants::BY_PAGE;
use shared::types::{BoxMailAttachmentItem, BoxMailAttachments, MailBoxes, MessageRequest, MessagesRequest, NotesChannel};
use shared::utils::box_type_index;

use crate::constants::{path_to_attachment, path_to_temp_with_ind};
use crate::db::{db_query, db_update_query};
use crate::db_notes::db_notes_route;
use crate::db_types::{DBBox, DBMailAddress, DBMailAttachments, DBPageResponse};
use crate::db_user::db_user_email;
use crate::receive::get_email;
use crate::send::send_message;
use crate::sse::{Message, sse_channel, sse_personal_channel};
use crate::state::{USER_BY_EMAIL, USER_BY_ID};
use crate::types::SessionStruct;
use crate::utils::get_dir_path;

pub async fn db_message_route(session: &SessionStruct, data: MessageRequest) {
    if data.send.is_some() {
        let session = session.clone();
        tokio::task::spawn(async move {
            send_message_init(&session, &data).await;
        });
    } else if let Some(notes_idp) = data.notes_idp {
        db_message_to_notes(session, notes_idp, data.idb).await;
    } else if let Some(_) = data.attachments {
        db_message_attachments(session, &data).await;
    } else {
        db_message_update(session, data).await;
    }
}

async fn send_message_init(session: &SessionStruct, data: &MessageRequest) {
    let sender = match USER_BY_ID.lock() {
        Ok(users) => {
            match users.get(&session.idu) {
                Some(user) => format!("{} <{}>", user.name, user.email),
                None => {
                    return;
                }
            }
        }
        Err(_) => {
            return;
        }
    };
    let recipient = match &data.recipient {
        Some(val) => val.clone(),
        None => "".to_string()
    };
    let subject = match &data.subject {
        Some(val) => val.clone(),
        None => "".to_string()
    };
    let content = match &data.content {
        Some(val) => val.clone(),
        None => "".to_string()
    };
    let attachments = data.attachments.clone();

    let send_result = send_message(&sender, &recipient, &subject, &content, &attachments).await;

    if send_result {
        let (name, address) = get_email(&sender);
        let sender: DBMailAddress = DBMailAddress { name, address };
        if let Some(attachments) = &attachments {
            let key = attachments.key.clone();
            for item in attachments.list.iter() {
                let target_file = path_to_attachment(&sender.address, &key, &item.id);
                let source_file = path_to_temp_with_ind(&key, &item.id);
                if let Ok(_) = fs::create_dir_all(get_dir_path(&target_file)).await {
                    if let Err(err) = fs::rename(&source_file, &target_file).await {
                        tracing::error!("send_message_init {err}");
                    }
                }
            }
        }

        let (name, address) = get_email(&recipient);
        let recipient: DBMailAddress = DBMailAddress { name, address };
        let attachments = match &attachments {
            Some(attachments) => Some(DBMailAttachments::from(attachments)),
            None => None
        };
        db_box_add(
            session.idu.clone(),
            box_type_index(&MailBoxes::Sent),
            true,
            sender,
            recipient,
            subject,
            content,
            attachments,
        );
    }

    message_personal(
        session,
        MessageRequest { send: Some(send_result), ..MessageRequest::default() },
    );
}

fn message_personal(session: &SessionStruct, data: MessageRequest) {
    match serde_json::to_string(&data) {
        Ok(text) => {
            sse_personal_channel(session, Message::Message(text));
        }
        Err(err) => {
            tracing::error!("serde_json[db_message_attachments] {:?}", err);
        }
    }
}

async fn db_message_attachments(session: &SessionStruct, data: &MessageRequest) {
    if data.remove_id.is_none() && data.idb == 0 {
        if let Some(attachments) = &data.attachments {
            // удаляем все
            if attachments.list.len() > 0 {
                let key = attachments.key.clone();
                for item in attachments.list.iter() {
                    if let Err(_) = fs::remove_file(&path_to_temp_with_ind(&key, &item.id)).await {}
                }
                return;
            }
        }
    }

    let key = Uuid::new_v4().to_string();

    // новый ключ
    let mut next_attachments = Some(BoxMailAttachments { key: key.clone(), list: vec![] });

    if let Some(remove_id) = &data.remove_id {
        // удаляем одну позицию
        if let Some(attachments) = &data.attachments {
            let key = attachments.key.clone();
            let mut list = attachments.list.clone();
            let list = if let Some(pos) = attachments.list.iter().position(|row| &row.id == remove_id) {
                if let Err(_) = fs::remove_file(&path_to_temp_with_ind(&key, &remove_id)).await {}
                list.remove(pos);
                list.clone()
            } else {
                list.clone()
            };
            next_attachments = Some(BoxMailAttachments { key, list });
        }
    } else if data.idb > 0 {
        // копируем из пересылаемого
        if let Some(row) = get_attachments(session, &data.idb).await {
            if let Some(email) = db_user_email(&session.idu).await {
                if let Some(prev) = row.attachments {
                    if prev.list.len() > 0 {
                        let mut ind: usize = 0;
                        let mut list: Vec<BoxMailAttachmentItem> = vec![];
                        for item in prev.list.iter() {
                            let source = path_to_attachment(&email, &prev.key, &item.id);
                            if let Ok(_) = fs::copy(source, &path_to_temp_with_ind(&key, &(ind + 1))).await {
                                ind = ind + 1;
                                list.push(BoxMailAttachmentItem {
                                    id: ind,
                                    size: item.size.clone(),
                                    file_name: item.file_name.clone(),
                                });
                            }
                        }
                        next_attachments = Some(BoxMailAttachments { key: key.clone(), list });
                    }
                }
            }
        }
    }

    message_personal(
        session,
        MessageRequest { idb: 0, attachments: next_attachments, ..MessageRequest::default() },
    );
}

async fn get_attachments(session: &SessionStruct, idb: &u64) -> Option<DBBox> {
    let idu = &session.idu;
    let rows = db_query(DBBox::from, &format!("select * from emails.boxes where idu={idu} and idb={idb};"), &[]).await;
    if rows.len() == 1 {
        Some(rows[0].clone())
    } else {
        None
    }
}

async fn db_message_to_notes(session: &SessionStruct, notes_idp: i32, idb: u64) {
    if let Some(data) = get_attachments(session, &idb).await {
        let content = data.content.clone();
        let sender = &data.sender;

        db_notes_route(session, NotesChannel {
            insert: Some(true),
            content: Some(content),
            label: sender.name.clone(),
            email: Some(sender.address.clone()),
            idp: Some(notes_idp),
            ..NotesChannel::default()
        }).await;
    }
}

async fn db_message_update(session: &SessionStruct, data: MessageRequest) {
    let mut message_updated = false;

    let mut fields: Vec<String> = vec![];
    if let Some(unread) = data.unread {
        fields.push(format!("unread={unread}"));
    }
    if let Some(box_target) = data.box_target {
        fields.push(format!("box={box_target}"));
    }

    if fields.len() > 0 {
        let idu = &session.idu;
        let idb = &data.idb;
        let fields = fields.join(",");
        message_updated = db_update_query(&format!("update emails.boxes set {fields} where idb={idb} and idu={idu};"), &[]).await;
    }

    if message_updated {
        match serde_json::to_string(&data) {
            Ok(text) => {
                sse_channel(session, Message::Message(text));
            }
            Err(err) => {
                tracing::error!("serde_json[db_message] {:?}", err);
            }
        }
    }
}

pub async fn db_messages_route(session: &SessionStruct, data: MessagesRequest) {
    let rows = db_box_page(&session.idu, &data.email_box, &data.page).await;
    let result = DBPageResponse { email_box: data.email_box.clone(), page: data.page.clone(), data: rows, news: false };
    match serde_json::to_string(&result) {
        Ok(text) => {
            sse_personal_channel(session, Message::Messages(text));
        }
        Err(err) => {
            tracing::error!("serde_json[BoxPageRequest] {:?}", err);
        }
    }
}

async fn db_box_page(idu: &i32, email_box: &i32, page: &usize) -> Vec<DBBox> {
    let limit: i64 = (*page as i64) * BY_PAGE;
    db_query(DBBox::from, include_str!("../sql/select_box_page.sql"), &[idu, email_box, &limit, &BY_PAGE]).await
}

pub fn db_box_add_received(flag_spam: bool, current_email: String, sender: DBMailAddress, recipient: DBMailAddress, subject: String, content: String, attachments: Option<DBMailAttachments>) {
    let box_num = box_type_index(if flag_spam { &MailBoxes::Trash } else { &MailBoxes::Inbox });
    let unread = !flag_spam;
    let idu = match USER_BY_EMAIL.lock() {
        Ok(users) => {
            match users.get(&current_email) {
                Some(val) => val.clone(),
                None => {
                    return;
                }
            }
        }
        Err(err) => {
            tracing::error!("db_box_insert[1]: {:?}", err);
            return;
        }
    };
    db_box_add(idu, box_num, unread, sender, recipient, subject, content, attachments);
}

pub fn db_box_add(idu: i32, box_num: usize, unread: bool, sender: DBMailAddress, recipient: DBMailAddress, subject: String, content: String, attachments: Option<DBMailAttachments>) {
    tokio::task::spawn(async move {
        let mut fields: Vec<String> = Vec::new();
        let mut linked: Vec<String> = Vec::new();
        let mut values: Vec<String> = Vec::new();

        if let Ok(txt) = serde_json::to_string(&sender) {
            fields.push("sender".to_string());
            values.push(format!("$${}$$", txt));
        }

        if let Ok(txt) = serde_json::to_string(&recipient) {
            fields.push("recipient".to_string());
            values.push(format!("$${}$$", txt));
        }

        if let Some(attachments) = attachments {
            if let Ok(txt) = serde_json::to_string(&attachments) {
                fields.push("attachments".to_string());
                values.push(format!("$${}$$", txt));
            }
        }

        fields.push("subject".to_string());
        linked.push(subject);
        values.push(format!("${}", linked.len()));

        fields.push("content".to_string());
        linked.push(content);
        values.push(format!("${}", linked.len()));

        fields.push("idu".to_string());
        values.push(idu.to_string());

        fields.push("box".to_string());
        values.push(box_num.to_string());

        fields.push("unread".to_string());
        values.push(unread.to_string());

        let prepared_linked = linked.iter().map(|a| a as &(dyn ToSql + Sync)).collect::<Vec<_>>();

        let rows = db_query(DBBox::from, &format!("insert into emails.boxes ({}) values ({}) returning idb, date, unread, sender, recipient, subject, content, attachments;", fields.join(","), values.join(",")), &prepared_linked[..]).await;
        if rows.len() == 1 {
            send_to_user(&idu, box_num as i32, rows);
        }
    });
}

fn send_to_user(idu: &i32, email_box: i32, data: Vec<DBBox>) {
    let result = DBPageResponse { email_box, page: 0, data, news: true };
    match serde_json::to_string(&result) {
        Ok(text) => {
            sse_channel(&SessionStruct::new(idu), Message::Messages(text));
        }
        Err(err) => {
            tracing::error!("send_to_user`; {:?}", err);
        }
    }
}
