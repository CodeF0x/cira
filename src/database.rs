use crate::models::{NewTicket, Ticket};
use crate::schema::tickets::dsl::tickets;
use crate::schema::tickets::id;
use diesel::{Connection, ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl, SqliteConnection};
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

pub fn delete_ticket(connection: &mut SqliteConnection, ticked_id: i32) -> QueryResult<usize> {
    diesel::delete(tickets.filter(id.eq(ticked_id))).execute(connection)
}

/**
* Setup test database before each test to make sure tests don't depend on each other and always have the same state.
**/
#[cfg(test)]
pub fn setup_database() {
    let mut database = DataBase::new();

    let test_ticket = NewTicket {
        title: "Test Title".to_string(),
        body: "Test Body".to_string(),
        // moment as of writing this
        created: "1688587842815".to_string(),
        last_modified: "1688587842815".to_string(),
    };

    diesel::sql_query("drop table tickets")
        .execute(&mut database.connection)
        .expect("Could not drop table tickets in test database");
    diesel::sql_query("create table tickets (id integer primary key not null, title varchar not null, body text not null, created text not null, last_modified text not null);")
        .execute(&mut database.connection)
        .expect("Could not re-create table in test database");
    diesel::insert_into(tickets)
        .values(&test_ticket)
        .execute(&mut database.connection)
        .expect("Could not write test data into test database");
}
