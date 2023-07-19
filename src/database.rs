use crate::models::{DataBaseUser, Label, NewTicket, NewUser, SqliteTicket, Ticket};
use crate::payloads::{FilterPayload, TicketPayload};
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
    reset_database();
    let mut database = DataBase::new();

    let test_ticket = NewTicket {
        title: "Test Title".to_string(),
        body: "Test Body".to_string(),
        // moment as of writing this
        created: "1688587842815".to_string(),
        last_modified: "1688587842815".to_string(),
        labels: "[\"Bug\", \"InProgress\"]".to_string(),
        assigned_user: Some(1),
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

#[cfg(test)]
pub fn reset_database() {
    dotenv().ok();
    let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL not set in .env");
    run_script::run_script!(format!(
        "diesel migration redo --database-url {}",
        database_url
    ))
    .unwrap();
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

pub fn filter_tickets_in_database(
    connection: &mut SqliteConnection,
    filter_payload: FilterPayload,
) -> Vec<Ticket> {
    /*
     * It's probably a horrible idea to fetch all data from the database an then map and filter through each and every entry,
     * but SQLite does not support arrays, so we would have to use LIKE for the labels which would be almost equally as slow.
     * Plus it would lead to horrible string interpolation with the %-pattern.
     *
     * An alternative would of course be FTS, but I couldn't get it to work with diesel. From my understanding,
     * diesel needs us to specify a primary key, but FTS does that automatically and doesn't want us to do it ourselves.
     */
    let all_tickets = tickets.load::<SqliteTicket>(connection).unwrap_or(vec![]);
    let parsed_tickets: Vec<Ticket> = all_tickets
        .iter()
        .map(|sqlite_ticket| sqlite_ticket.to_ticket())
        .collect();

    parsed_tickets
        .iter()
        .filter(|t| {
            filter_by_title(&filter_payload.title, t)
                && filter_by_assigned_user(filter_payload.assigned_user, t)
                && filter_by_labels(&filter_payload.labels, t)
        })
        .cloned()
        .collect::<Vec<_>>()
}

// function that takes in an option. if okay, filter by value. if none, simply return true
fn filter_by_assigned_user(user_id: Option<i32>, ticket: &Ticket) -> bool {
    match user_id {
        Some(user_id) => ticket.assigned_user.unwrap_or(0) == user_id,
        None => true,
    }
}

fn filter_by_title(ticket_title: &Option<String>, ticket: &Ticket) -> bool {
    match ticket_title {
        Some(ticket_title) => ticket.title.contains(ticket_title),
        None => true,
    }
}

fn filter_by_labels(ticket_labels: &Option<Vec<Label>>, ticket: &Ticket) -> bool {
    match ticket_labels {
        Some(ticket_labels) => {
            let mut includes = false;

            for lbl in ticket_labels.iter() {
                includes = ticket.labels.contains(lbl);
            }

            includes
        }
        None => true,
    }
}
