## Cira, a Jira clone

Basically no functionality yet.

### Setup

1. Install diesel_cli
   1. `cargo install diesel_cli`
   2. Create a `.env` file with the database url: `DATABASE_URL="<url>"`
   3. run `diesel setup`
   4. run `diesel migration run` to create the database and all tables (to reset the database, run `diesel migrate redo`)
2. run `cargo run`

