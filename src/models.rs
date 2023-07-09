use diesel::prelude::*;
use diesel::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Deserialize, Serialize, Debug)]
#[diesel(table_name = crate::schema::tickets)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SqliteTicket {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub created: String,
    pub last_modified: String,
    pub labels: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ticket {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub created: String,
    pub last_modified: String,
    pub labels: Vec<Label>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Label {
    Feature,
    Bug,
    WontFix,
    Done,
    InProgress,
    Open,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::tickets)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewTicket {
    pub title: String,
    pub body: String,
    pub created: String,
    pub last_modified: String,
    pub labels: String,
}

impl SqliteTicket {
    pub fn to_ticket(&self) -> Ticket {
        Ticket {
            id: self.id,
            title: self.title.clone(),
            body: self.body.clone(),
            created: self.created.clone(),
            last_modified: self.last_modified.clone(),
            labels: serde_json::from_str(&self.labels).unwrap(),
        }
    }
}
