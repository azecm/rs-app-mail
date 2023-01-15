use postgres_types::ToSql;

use shared::types::NotesChannel;

use crate::{db_conn, DBNotes};
use crate::db::{db_query, db_update_query};
use crate::db_types::{DBCount, DBIdn, DBNotesMini};
use crate::sse::{Message, sse_channel};
use crate::types::SessionStruct;

fn send(session: &SessionStruct, data: NotesChannel) {
    match serde_json::to_string(&data) {
        Ok(text) => {
            sse_channel(session, Message::Notes(text));
        }
        Err(err) => {
            tracing::error!("serde_json[NotesChannel] {:?}", err);
        }
    }
}

pub async fn db_notes_select(idu: &i32) -> Vec<DBNotes> {
    db_query(DBNotes::from, include_str!("../sql/select_notes.sql"), &[idu]).await
}

pub async fn db_notes_route(session: &SessionStruct, data: NotesChannel) {
    if data.remove.unwrap_or(false) {
        db_remove(session, &data.idn).await;
    } else if data.insert.unwrap_or(false) {
        db_insert(session, data).await;
    } else {
        db_update(session, data).await;
    }
}

async fn db_insert(session: &SessionStruct, data: NotesChannel) {
    let idu = &session.idu;
    let mut fields: Vec<String> = Vec::new();
    let mut linked: Vec<String> = Vec::new();
    let mut values: Vec<String> = Vec::new();

    if let Some(label) = &data.label {
        fields.push("label".to_string());
        linked.push(label.clone());
        values.push(format!("${}", linked.len()));
    }
    if let Some(email) = &data.email {
        fields.push("email".to_string());
        linked.push(email.clone());
        values.push(format!("${}", linked.len()));
    }
    if let Some(content) = &data.content {
        fields.push("content".to_string());
        linked.push(content.clone());
        values.push(format!("${}", linked.len()));
    }

    let idp = data.idp.unwrap_or(0);
    fields.push("idp".to_string());
    values.push(idp.to_string());

    let mut prev = db_notes_position_prev(idu, &idp).await;

    let position = data.position.unwrap_or((prev.len() + 1) as i32);
    fields.push("position".to_string());
    values.push(position.to_string());

    fields.push("idu".to_string());
    values.push(idu.to_string());

    if !fields.is_empty() {
        let prepared_values = linked.iter().map(|a| a as &(dyn ToSql + Sync)).collect::<Vec<_>>();

        let rows = db_query(DBIdn::from, &format!("insert into emails.notes ({}) values ({}) returning idn;", fields.join(","), values.join(",")), &prepared_values[..]).await;

        if rows.len() == 1 {
            let idn = rows[0].idn;
            send(session, NotesChannel {
                idn: idn as i32,
                insert: Some(true),
                label: data.label,
                email: data.email,
                content: data.content,
                idp: Some(idp),
                position: Some(position),
                ..NotesChannel::default()
            });

            prev.insert((position - 1) as usize, DBNotesMini {
                idp,
                position,
                idn: idn as i32,
            });
            db_notes_test_position(session, prev).await;
        }
    }
}

async fn db_remove(session: &SessionStruct, idn: &i32) {
    let idu = &session.idu;
    let mut prev = db_notes_position_prev(idu, &db_notes_idp(idu, idn).await).await;

    let rows = db_query(DBCount::from, include_str!("../sql/select_notes_idp.sql"), &[&idu, &idn]).await;
    if rows[0].count == 0 {
        send(session, NotesChannel { idn: *idn, remove: Some(true), ..NotesChannel::default() });
        prev.retain(|row| &row.idn != idn);
        db_notes_test_position(session, prev).await;
    }
}

async fn db_update(session: &SessionStruct, data: NotesChannel) {
    let idu = &session.idu;
    let idn = data.idn;
    let mut fields: Vec<String> = Vec::new();
    let mut values: Vec<String> = Vec::new();
    let label = data.label;
    let email = data.email;
    let content = data.content;
    let idp = data.idp;
    let event = data.event;

    let position = if let Some(idp) = &idp {
        Some((db_notes_position_prev(idu, idp).await.len() + 1) as i32)
    } else {
        data.position
    };

    // ===

    if let Some(label) = &label {
        fields.push(format!("label=${}", fields.len() + 1));
        values.push(label.clone());
    }
    if let Some(email) = &email {
        fields.push(format!("email=${}", fields.len() + 1));
        values.push(email.clone());
    }
    if let Some(content) = &content {
        fields.push(format!("content=${}", fields.len() + 1));
        values.push(content.clone());
    }
    if let Some(position) = &position {
        fields.push(format!("position={position}"));
    }
    if let Some(idp) = &idp {
        fields.push(format!("idp={idp}"));
    }
    if let Some(event_current) = &event {
        if event_current.date.is_empty() {
            fields.push("event=null".to_string());
        } else if let Ok(txt) = serde_json::to_string(&event_current) {
            fields.push(format!("event=$${}$$", txt));
        }
    }
    if !fields.is_empty() {
        let mut prev = match position.is_some() || idp.is_some() {
            true => db_notes_position_prev(idu, &db_notes_idp(idu, &idn).await).await,
            false => vec![]
        };
        let prepared_values = values.iter().map(|a| a as &(dyn ToSql + Sync)).collect::<Vec<_>>();

        if db_update_query(&format!("update emails.notes set {} where idu={idu} and idn={idn};", fields.join(",")), &prepared_values[..]).await {
            let to = if idp.is_some() { None } else { position };
            let position = if idp.is_some() { position } else { None };
            send(session, NotesChannel { idn, label, email, content, event, idp, position, to, ..NotesChannel::default() });

            if idp.is_some() {
                let current = prev.iter().position(|row| row.idn == idn);
                if let Some(current) = current {
                    prev.remove(current);
                }
                db_notes_test_position(session, prev).await;
            } else if let Some(position) = position {
                let current = prev.iter().position(|row| row.idn == idn);
                if let Some(current) = current {
                    let item = prev.remove(current);
                    prev.insert((position - 1) as usize, item);
                }
                db_notes_test_position(session, prev).await;
            }
        }
    }
}

async fn db_notes_idp(idu: &i32, idn: &i32) -> i32 {
    let rows = db_query(DBNotesMini::from, "select idn, idp, position from emails.notes where idu=$1 and idn=$2;", &[idu, idn]).await;
    if !rows.is_empty() {
        rows[0].idp
    } else {
        0
    }
}

async fn db_notes_position_prev(idu: &i32, idp: &i32) -> Vec<DBNotesMini> {
    db_query(DBNotesMini::from, include_str!("../sql/select_notes_position.sql"), &[idu, idp]).await
}

async fn db_notes_test_position(session: &SessionStruct, prev: Vec<DBNotesMini>) {
    match db_conn().await {
        Ok(db) => {
            for (ind, item) in prev.iter().enumerate() {
                let position = (ind + 1) as i32;
                if item.position != position {
                    let idn = item.idn;
                    if (db.query("update emails.notes set position=$1 where idn=$2;", &[&position, &idn]).await).is_ok() {
                        send(session, NotesChannel { idn, position: Some(position), ..NotesChannel::default() });
                    }
                }
            }
        }
        Err(err) => {
            tracing::error!("db_notes_position_prev: {:?}", err);
        }
    };
}

