// @generated automatically by Diesel CLI.

diesel::table! {
    tickets (id) {
        id -> Integer,
        title -> Text,
        body -> Text,
        created -> Text,
        last_modified -> Text,
    }
}
