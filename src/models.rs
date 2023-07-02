use crate::schema::tickets;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::tickets)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Ticket {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub created: String,
    pub last_modified: String,
}

#[derive(Insertable)]
#[diesel(table_name = tickets)]
pub struct NewTicket {
    pub title: String,
    pub body: String,
    pub created: String,
    pub last_modified: String,
}
