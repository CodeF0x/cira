use crate::models::NewTicket;
use diesel::{Connection, QueryResult, RunQueryDsl, SqliteConnection};
use dotenvy::dotenv;
use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_ticket(
    connection: &mut SqliteConnection,
    title: String,
    body: String,
) -> QueryResult<usize> {
    use crate::schema::tickets;

    let now_in_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::new(0, 0));
    let new_ticket = NewTicket {
        title,
        body,
        created: now_in_millis.as_millis().to_string(),
        last_modified: now_in_millis.as_millis().to_string(),
    };

    diesel::insert_into(tickets::table)
        .values(&new_ticket)
        .execute(connection)
}
