use crate::filters::{
    filter_by_assigned_user, filter_by_labels, filter_by_status, filter_by_title,
};
use crate::models::{
    DataBaseUser, DatabaseSession, Label, NewSession, NewTicket, NewUser, SqliteTicket, Status,
    Ticket,
};
use crate::payloads::{FilterPayload, TicketPayload};
use crate::schema::sessions::dsl::sessions;
use crate::schema::sessions::token;
use crate::schema::tickets::dsl::tickets;
use crate::schema::tickets::{body, id, labels, last_modified, status, title};
use crate::schema::users::dsl::users;
use crate::schema::users::email;
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
        // status is required to be sent by user to not have ugly null handling in update function,
        // so even if user sends "Closed", set it to open.
        // Makes not sense to create a closed ticket.
        status: Status::Open.to_string(),
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
            status.eq(ticket.status.to_string()),
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

    diesel::insert_into(users)
        .values(new_user)
        .get_result(connection)
}

pub fn get_user_by_email(
    user_email: String,
    connection: &mut SqliteConnection,
) -> QueryResult<DataBaseUser> {
    users.filter(email.eq(user_email)).get_result(connection)
}

pub fn filter_tickets_in_database(
    connection: &mut SqliteConnection,
    filter_payload: FilterPayload,
) -> Result<Vec<Ticket>, ()> {
    /*
     * It's probably a horrible idea to fetch all data from the database an then map and filter through each and every entry,
     * but SQLite does not support arrays, so we would have to use LIKE for the labels which would be almost equally as slow.
     * Plus it would lead to horrible string interpolation with the %-pattern.
     *
     * An alternative would of course be FTS, but I couldn't get it to work with diesel. From my understanding,
     * diesel needs us to specify a primary key, but FTS does that automatically and doesn't want us to do it ourselves.
     */
    return match tickets.load::<SqliteTicket>(connection) {
        Ok(all_tickets) => {
            let parsed_tickets: Vec<Ticket> = all_tickets
                .iter()
                .map(|sqlite_ticket| sqlite_ticket.to_ticket())
                .collect();

            Ok(parsed_tickets
                .iter()
                .filter(|t| {
                    filter_by_title(&filter_payload.title, t)
                        && filter_by_assigned_user(filter_payload.assigned_user, t)
                        && filter_by_labels(&filter_payload.labels, t)
                        && filter_by_status(filter_payload.status, t)
                })
                .cloned()
                .collect::<Vec<_>>())
        }
        Err(_) => Err(()),
    };
}

pub fn write_session_to_db(new_session: NewSession, connection: &mut SqliteConnection) {
    diesel::insert_into(sessions)
        .values(new_session)
        .execute(connection)
        .unwrap();
}

pub fn remove_session_from_db(session_token: String, connection: &mut SqliteConnection) {
    diesel::delete(sessions.filter(token.eq(session_token)))
        .execute(connection)
        .unwrap();
}

pub fn session_in_db(session_token: String, connection: &mut SqliteConnection) -> bool {
    match sessions
        .filter(token.eq(session_token))
        .get_result::<DatabaseSession>(connection)
    {
        Ok(_) => true,
        _ => false,
    }
}
