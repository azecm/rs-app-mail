use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

use crate::db::db_query;
use crate::state::{USER_BY_EMAIL, USER_BY_ID};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DBUserSelect {
    pub prefix: String,
    pub signature: String,
}

impl From<Row> for DBUserSelect {
    fn from(row: Row) -> Self {
        Self {
            prefix: row.get("name"),
            signature: row.get("signature"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DBUserIdu {
    pub idu: i32,
}

impl From<Row> for DBUserIdu {
    fn from(row: Row) -> Self {
        Self {
            idu: row.get("idu"),
        }
    }
}

pub async fn db_user_select(idu: &i32) -> DBUserSelect {
    let rows = db_query(DBUserSelect::from, include_str!("../sql/select_user.sql"), &[idu]).await;
    if rows.len() == 1 { rows[0].clone() } else { DBUserSelect::default() }
}

// ===

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DBUserInit {
    pub idu: i32,
    pub email: String,
    pub name: String,
}

impl From<Row> for DBUserInit {
    fn from(row: Row) -> Self {
        Self {
            idu: row.get("idu"),
            email: row.get("email"),
            name: row.get("name"),
        }
    }
}

pub async fn db_user_init() {
    let rows = db_query(DBUserInit::from, "select idu, email, name from emails.users;", &[]).await;
    for row in rows.iter() {
        if let Ok(mut users) = USER_BY_EMAIL.lock() {
            users.insert(row.email.clone(), row.idu);
        }
        if let Ok(mut users) = USER_BY_ID.lock() {
            users.insert(row.idu, DBUserInit {
                idu: row.idu,
                email: row.email.clone(),
                name: row.name.clone(),
            });
        }
    }
}

pub async fn db_user_login(mail_box: String, user_name: String, user_pass: String) -> i32 {
    let rows = db_query(DBUserIdu::from, include_str!("../sql/select_user_login.sql"), &[&mail_box, &user_name, &user_pass]).await;
    if rows.len() == 1 {
        rows[0].idu
    } else {
        0
    }
}

// ===

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DBUserEmail {
    pub email: String,
}

impl From<Row> for DBUserEmail {
    fn from(row: Row) -> Self {
        Self {
            email: row.get("email"),
        }
    }
}

pub async fn db_user_email(idu: &i32) -> Option<String> {
    let rows = db_query(DBUserEmail::from, include_str!("../sql/select_user.sql"), &[idu]).await;
    if rows.len() == 1 { Some(rows[0].email.clone()) } else { None }
}