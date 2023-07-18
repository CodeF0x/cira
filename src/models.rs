use diesel::prelude::*;
use diesel::{FromSqlRow, Queryable};
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
    pub assigned_user: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, FromSqlRow, Clone)]
pub struct Ticket {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub created: String,
    pub last_modified: String,
    pub labels: Vec<Label>,
    pub assigned_user: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Label {
    Feature,
    Bug,
    WontFix,
    Done,
    InProgress,
    Open,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::tickets)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewTicket {
    pub title: String,
    pub body: String,
    pub created: String,
    pub last_modified: String,
    pub labels: String,
    pub assigned_user: Option<i32>,
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
            assigned_user: self.assigned_user,
        }
    }
}

#[derive(Serialize, Deserialize, Queryable)]
pub struct DataBaseUser {
    pub id: i32,
    pub display_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewUser {
    pub display_name: String,
    pub email: String,
    pub password: String,
}
