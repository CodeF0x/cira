#[cfg(test)]
pub mod helpers {
    use crate::database::DataBase;
    use crate::models::{NewTicket, NewUser, Status};
    use crate::schema::tickets::dsl::tickets;
    use crate::schema::users;
    use diesel::RunQueryDsl;
    use dotenvy::dotenv;
    use std::env;

    /**
     * Setup test database before each test to make sure tests don't depend on each other and always have the same state.
     **/
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
            status: Status::Open.to_string(),
        };
        let test_user = NewUser {
            display_name: "user".to_string(),
            email: "test@example.com".to_string(),
            // hash of string "123"
            password: "$argon2id$v=19$m=4096,t=192,p=24$0QaRo64feVRR8Ash0tB4tMDZeEcdYVUAB8j1QmJ/Uuc$NOYTu4UQ1cC8WSAaA3W05ognuj1z2WaTS7fvxhbTKQk".to_string(),
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

    pub fn reset_database() {
        dotenv().ok();
        let database_url =
            env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL not set in .env");
        run_script::run_script!(format!(
            "diesel migration redo --database-url {}",
            database_url
        ))
        .unwrap();
    }
}
