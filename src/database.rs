use crate::models::{DataBaseUser, Label, NewTicket, NewUser, SqliteTicket};
use crate::payloads::TicketPayload;
use crate::schema::tickets::dsl::tickets;
use crate::schema::tickets::{body, id, labels, last_modified, title};
use crate::schema::users;
use argonautica::Hasher;
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
    new_title: String,
    new_body: String,
    new_labels: Vec<Label>,
    new_user: Option<i32>,
) -> QueryResult<SqliteTicket> {
    use crate::schema::tickets;

    let now_in_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::new(0, 0));
    let new_ticket = NewTicket {
        title: new_title,
        body: new_body,
        created: now_in_millis.as_millis().to_string(),
        last_modified: now_in_millis.as_millis().to_string(),
        labels: serde_json::to_string(&new_labels).unwrap(),
        assigned_user: new_user,
    };

    diesel::insert_into(tickets::table)
        .values(&new_ticket)
        .get_result(connection)
}

pub fn get_all_tickets(connection: &mut SqliteConnection) -> QueryResult<Vec<SqliteTicket>> {
    tickets.load::<SqliteTicket>(connection)
}

pub fn delete_ticket(
    connection: &mut SqliteConnection,
    ticked_id: i32,
) -> QueryResult<SqliteTicket> {
    diesel::delete(tickets.filter(id.eq(ticked_id))).get_result(connection)
}

/**
* Setup test database before each test to make sure tests don't depend on each other and always have the same state.
**/
#[cfg(test)]
pub fn setup_database() {
    dotenv().ok();
    let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL not set in .env");
    let mut database = DataBase::new();
    run_script::run_script!(format!(
        "diesel migration redo --database-url {}",
        database_url
    ))
    .unwrap();

    let test_ticket = NewTicket {
        title: "Test Title".to_string(),
        body: "Test Body".to_string(),
        // moment as of writing this
        created: "1688587842815".to_string(),
        last_modified: "1688587842815".to_string(),
        labels: "[]".to_string(),
        assigned_user: None,
    };
    let test_user = NewUser {
        display_name: "user".to_string(),
        email: "test@example.com".to_string(),
        password: "asdg7asd8g7".to_string(),
    };

    diesel::insert_into(tickets)
        .values(&test_ticket)
        .execute(&mut database.connection)
        .expect("Could not write test data into test database");
    diesel::insert_into(users::table)
        .values(test_user)
        .execute(&mut database.connection)
        .expect("Could not write test user into test database");
}

pub fn edit_ticket(
    connection: &mut SqliteConnection,
    ticket: TicketPayload,
    ticket_id: i32,
) -> QueryResult<SqliteTicket> {
    let now_in_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::new(0, 0));

    diesel::update(tickets.filter(id.eq(ticket_id)))
        .set((
            title.eq(ticket.title),
            body.eq(ticket.body),
            labels.eq(serde_json::to_string(&ticket.labels).unwrap()),
            last_modified.eq(now_in_millis.as_millis().to_string()),
        ))
        .get_result(connection)
}

pub fn create_user(
    connection: &mut SqliteConnection,
    user_payload: NewUser,
) -> QueryResult<DataBaseUser> {
    dotenv().ok();

    let hash_secret = env::var("HASH_SECRET").expect("HASH_SECRET not set!");
    let mut hasher = Hasher::default();

    let hash = hasher
        .with_password(user_payload.password)
        .with_secret_key(hash_secret)
        .hash()
        .unwrap();

    // be careful with order of properties.
    // values are written to database in the order they are in the struct
    // keep in mind to not "assign" e. g. password to email
    let new_user = NewUser {
        display_name: user_payload.display_name,
        email: user_payload.email,
        password: hash,
    };

    diesel::insert_into(users::table)
        .values(new_user)
        .get_result(connection)
}
