-- Your SQL goes here
create table tickets (
    id integer primary key not null,
    title varchar not null,
    body text not null,
    created text not null,
    last_modified text not null
);