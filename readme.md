# Cira - A Minimalistic Ticket System Backend

Cira is a minimalistic backend for managing tickets in small projects. It provides essential APIs to handle ticket creation, updates, deletion, labeling, assignment, and filtering.

## Overview
Cira offers foundational API functionalities for managing tickets including:
- Creation and deletion of tickets
- Updating existing tickets
- Grouping tickets by labels
- Assigning tickets to users
- Filtering tickets by various criteria
- User authentication via bearer tokens

## Setup & Usage

### Prerequisites
Ensure you have installed the following tools:
- [Rust](https://rust-lang.org)
- [Git](https://git-scm.com/)
- [Diesel](https://diesel.rs)
- [SQLite](https://www.sqlite.org/)

### Running Locally

1. **Clone the repository**:
   ```bash
   git clone https://github.com/CodeF0x/cira.git
   cd cira
   ```
2. **Install Diesel CLI for SQLite** and set up database:
   ```bash
   cargo install diesel_cli --no-default-features --features "sqlite"
   diesel setup
   ```
3. **Build and run**:
   ```bash
   cargo build --release
   ./target/release/cira
   ```

**Note**: Make sure to update both `HASH_SECRET` and `JWT_SECRET` in the `.env` file with cryptographically secure values!

### Troubleshooting

If you encounter errors during installation or setup:
- On Debian, if you get a `linking with 'cc' failed: exit status: 1` error, ensure `build-essential` is installed.
- For issues while installing Diesel CLI, try running: `sudo apt install -y libsqlite3-dev libpq-dev libmysqlclient-dev`.

---

## API Documentation

### General Information
- **Authorization**: Every endpoint requires a Bearer Token for authentication except the endpoints related to user login and signup (`/api/login` and `/api/signup`).

### Ticket Management

#### Create a New Ticket

```http
POST /api/tickets
```

**Payload**:

| Property        | Type            | Description                                                                   |
|:----------------|:----------------|:------------------------------------------------------------------------------|
| `title`         | `string`        | **Required**. Title of the ticket                                            |
| `body`          | `string`        | **Required**. Body content of the ticket                                     |
| `labels`        | `Array<string>` | **Required**. Labels for categorizing the ticket                            |
| `status`        | `string`        | **Required**. Status (e.g., 'Open')                                         |
| `assigned_user` | `id \| null`    | **Optional**. ID of user assigned to this ticket or `null`.                  |

**Status Options**:

- Open
- Closed

**Label Options**:

- Feature
- Bug
- WontFix
- Done
- InProgress

#### Get All Tickets

```http
GET /api/tickets
```

Retrieves all tickets.

#### Delete a Ticket

```http
DELETE /api/tickets/{id}
```

**Path Parameters**:

| Parameter | Type   | Description                        |
|:----------|:-------|:-----------------------------------|
| `id`      | number | **Required**. ID of the ticket     |

#### Edit a Ticket

```http
PUT /api/tickets/{id}
```

**Path Parameters**: (Same as Delete Ticket)

**Payload**: (Same structure as Create a New Ticket)

### User Authentication

#### Sign Up

```http
POST /api/signup
```

**Payload**:

| Property       | Type   | Description                            |
|:---------------|:-------|:---------------------------------------|
| `display_name` | string | **Required**. Display name             |
| `email`        | string | **Required**. Email address            |
| `password`     | string | **Required**. Password                 |

#### Login

```http
POST /api/login
```

**Payload**:

| Property  | Type   | Description                            |
|:----------|:-------|:---------------------------------------|
| `email`   | string | **Required**. Email address            |
| `password` | string | **Required**. Password                 |

Returns a bearer token upon successful login.

#### Logout

```http
POST /api/logout
```

Logs out the user by invalidating their current token.

### User Management

#### Get All Users

```http
GET /api/users
```

Retrieves all users in a simplified format (`id`, `email`, `display_name`).

### Filter Tickets

```http
POST /api/filter
```

**Payload**:

| Property        | Type                    | Description                                                     |
|:----------------|:------------------------|:----------------------------------------------------------------|
| `title`         | `string \| null`        | **Optional**. Title to search for. Can be omitted or null       |
| `labels`        | `Array<string> \| null` | **Optional**. Labels to search for. Can be omitted or null      |
| `status`        | `string \| null`        | **Optional**. Status to search for. Can be omitted or null      |
| `assigned_user` | `id \| null`            | **Optional**. Assignee ID to search for. Can be omitted or null |

Returns filtered results.

---

## Contributing

Contributions in any form (issues, PRs, feedback) are welcome!

---

## License

[MIT](https://choosealicense.com/licenses/mit/)
