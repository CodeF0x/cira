> Warning: You should probably not use this in production or with any data that is sensitive as the authentication
> system
> is made very amateurish.

# Cira - a minimalistic ticket system backend

Cira is a minimalistic ticket system backend for small and simple projects.
You could even use it as an overcomplicated todo app.

The only thing it does is to provide an api.
The rest is up to you.
Create a native desktop or smartphone app, or keep in the web as most ticket boards to.
You decide, the possibilities are endless.

## Features

Cira gives you the foundation to:

- create and delete tickets
- update tickets after creation
- group tickets by labels
- assign tickets to users
- filter tickets by labels, assignee, status and labels
- authentication with bearer tokens

## Run Locally

Simply clone the repository, set up the database and compile the application locally.
Then start it.
You are required to have installed: [the rust programming language](https://rust-lang.org),
[git](https://git-scm.com/), [diesel](https://diesel.rs), [sqlite](https://www.sqlite.org/)

```bash
git clone https://github.com/CodeF0x/cira.git
cd cira
cargo install diesel_cli --no-default-features --features "sqlite"
diesel setup
cargo build --release
./target/release/cira
```

If you're on Debian and get `error: linking with 'cc' failed: exit status: 1`, make sure to have `build-essential`
installed. Same goes for other distros (build-essential might be called different / have an equivalent).

If this error accours while installing `diesel_cli`, try `sudo apt install -y libsqlite3-dev libpq-dev libmysqlclient-dev`.

There are some default values set in the .env file, you can adjust them as you wish.
Keep in mind to change the code as well. For example, if you change the database file name, change it in the .env file
as well.

If everything went well and there is no output after running the last command, cira is listening on port `8080`.

You can also launch it in a screen or in a container, so it runs without an active shell session.

## Running Tests

To run tests, set up a fake database that is independent of the actual production database.
You need to have installed [diesel](https://diesel.rs), [sqlite](https://www.sqlite.org/) and [rust](https://rust-lang.org).

```bash
diesel migration run --database-url test-backend.sqlite
cargo test
```

## API Reference

#### Create a new ticket

```http
  POST /tickets
```

Your payload must be valid JSON and contain the following properties:

| Property        | Type            | Description                                                                   |
|:----------------|:----------------|:------------------------------------------------------------------------------|
| `title`         | `string`        | **Required**. The title of your ticket                                        |
| `body`          | `string`        | **Required**. The body of your ticket                                         |
| `labels`        | `Array<string>` | **Required**. Labels of your ticket                                           |
| `status`        | `string`        | **Required**. The status of your ticket (set it to "Open")                    |
| `assigned_user` | `id \| null`    | **Optional**. A user the ticket should be assigned to. Can be omitted or null |

Possible Status options:

- `Open`
- `Closed`

Possible Label options:

- `Feature`
- `Bug`
- `WontFix`
- `Done`
- `InProgress`

Creates a new ticket and returns it.

#### Get tickets

```http
  GET /tickets
```

Get all tickets.

#### Delete ticket

```http
  DELETE /tickets/{id}
```

URL parameters:

| Property | Type     | Description                        |
|:---------|:---------|:-----------------------------------|
| `id`     | `number` | **Required**. The id of the ticket |

Deletes a ticket and returns it.

#### Edit a ticket

```http
  POST /tickets/{id}
```

URL parameters:

| Property | Type     | Description                        |
|:---------|:---------|:-----------------------------------|
| `id`     | `number` | **Required**. The id of the ticket |

Your payload must be valid JSON and contain the following properties:

| Property        | Type            | Description                                                                   |
|:----------------|:----------------|:------------------------------------------------------------------------------|
| `title`         | `string`        | **Required**. The title of your ticket                                        |
| `body`          | `string`        | **Required**. The body of your ticket                                         |
| `labels`        | `Array<string>` | **Required**. Labels of your ticket                                           |
| `status`        | `string`        | **Required**. The status of your ticket (set it to "Open")                    |
| `assigned_user` | `id \| null`    | **Optional**. A user the ticket should be assigned to. Can be omitted or null |

Possible Status options:

- `Open`
- `Closed`

Possible Label options:

- `Feature`
- `Bug`
- `WontFix`
- `Done`
- `InProgress`

Updates a ticket and returns it.

#### Sign up

```
  POST /users
```

Your payload must be valid JSON and contain the following properties:

| Property       | Type     | Description                            |
|:---------------|:---------|:---------------------------------------|
| `display_name` | `string` | **Required**. The user's display name  |
| `email`        | `string` | **Required**. The user's email address |
| `password`     | `string` | **Required**. The user's password      |

Create a new user and return it.

#### Filter tickets

```http
  POST /filter
```

Your payload must be valid JSON and contain the following properties:

| Property        | Type                    | Description                                                     |
|:----------------|:------------------------|:----------------------------------------------------------------|
| `title`         | `string \| null`        | **Optional**. Title to search for. Can be omitted or null       |
| `labels`        | `Array<string> \| null` | **Optional**. Labels to search for. Can be omitted or null      |
| `status`        | `string \| null`        | **Optional**. Status to search for. Can be omitted or null      |
| `assigned_user` | `id \| null`            | **Optional**. Assignee id to search for. Can be omitted or null |

Lets you filter for tickets and return results.

## Contributing

Contributions are always welcome!

Either by submitting issues, pull requests or just general constructive feedback in the form of issues,
emails or direct messages on Telegram.

## License

[MIT](https://choosealicense.com/licenses/mit/)

MIT or "I do not care what you do with this as long as you don't claim it to be your own"-license.