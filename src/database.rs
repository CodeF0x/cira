use crate::models::{NewTicket, Ticket};
use crate::schema::tickets::dsl::tickets;
use diesel::{Connection, QueryResult, RunQueryDsl, SqliteConnection};
use dotenvy::dotenv;
use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct DataBase {
    pub connection: SqliteConnection,
}

impl DataBase {
    pub fn new() -> Self {
        dotenv().ok();

        #[cfg(test)]
        let variable_name = "TEST_DATABASE_URL";

        #[cfg(not(test))]
        let variable_name = "DATABASE_URL";

        let database_url = env::var(variable_name).expect("Could not find database url in .env");

        DataBase {
            connection: SqliteConnection::establish(&database_url)
                .unwrap_or_else(|_| panic!("Error connecting to {}", database_url)),
        }
    }
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

pub fn get_all_tickets(connection: &mut SqliteConnection) -> QueryResult<Vec<Ticket>> {
    tickets.load::<Ticket>(connection)
}
