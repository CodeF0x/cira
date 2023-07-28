// @generated automatically by Diesel CLI.

diesel::table! {
    tickets (id) {
        id -> Integer,
        title -> Text,
        body -> Text,
        created -> Text,
        last_modified -> Text,
        labels -> Text,
        assigned_user -> Nullable<Integer>,
        status -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        display_name -> Text,
        email -> Text,
        password -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    tickets,
    users,
);
