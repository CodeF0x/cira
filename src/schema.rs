// @generated automatically by Diesel CLI.

#[rustfmt::skip]
diesel::table! {
    tickets (id) {
        id -> Integer,
        title -> Text,
        body -> Text,
        created -> Text,
        last_modified -> Text,
        labels -> Text,
        assigned_user -> Nullable<Integer>,
    }
}

#[rustfmt::skip]
diesel::table! {
    users (id) {
        id -> Integer,
        display_name -> Text,
        email -> Text,
        password -> Text,
    }
}

#[rustfmt::skip]
diesel::allow_tables_to_appear_in_same_query!(tickets, users,);
