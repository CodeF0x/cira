-- Your SQL goes here
create table tickets (
    id integer primary key not null,
    title varchar not null,
    body text not null,
    created text not null,
    last_modified text not null,
    labels text not null,
    assigned_user integer,
    status text not null
);

create table users (
    id integer primary key not null,
    display_name text not null,
    email text not null,
    password text not null
);

create table sessions (
    id integer primary key not null,
    token text not null
);