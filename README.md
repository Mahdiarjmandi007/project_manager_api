# Project Manager API

A RESTful API built with Rust for managing projects, team members, and tasks.

The API provides authentication, role-based access control, and project and task management features. It uses SQLite for data persistence and supports secure password hashing and token-based authentication.

## Features

- User registration and login
- JWT-based authentication
- Access and refresh token support
- Secure password hashing using Argon2
- Role-based access control (RBAC)
- User management for administrators
- Project creation and management
- Project member management
- Task creation, assignment, and management
- Task status updates (`todo`, `in_progress`, `done`)
- Task filtering and pagination
- SQLite database integration

## Tech Stack

- **Language:** Rust
- **Web Framework:** Axum
- **Database:** SQLite
- **Database Access:** SQLx
- **Asynchronous Runtime:** Tokio
- **Authentication:** JSON Web Tokens (JWT)
- **Password Hashing:** Argon2
- **Serialization and Deserialization:** Serde
- **Environment Configuration:** dotenvy
- **Testing:** Rust's built-in testing framework

## Design Decisions

### 1. Language and Framework

Rust was chosen for its type safety and performance. Axum provides an asynchronous framework for defining HTTP routes and building REST API endpoints.

### 2. Database

SQLite was selected because it is lightweight and does not require a separate database server. SQLx is used to interact with the database asynchronously.

### 3. Password Security

Argon2 is used to hash passwords before storing them in the database. Plain-text passwords are never stored.



### 4. HTTP Status Codes

The API distinguishes between authentication and authorization errors:

- `401 Unauthorized`: The request does not contain valid authentication credentials.
- `403 Forbidden`: The user is authenticated but does not have permission to perform the requested operation.
- `404 Not Found`: The requested resource does not exist.

Returning `403` for an existing resource that the user cannot access makes the distinction between missing resources and insufficient permissions explicit.


## Installation & Setup

### Prerequisites

Make sure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install)
- [Git](https://git-scm.com/)

### 1. Clone the Repository

```bash
git clone https://github.com/Mahdiarjmandi007/project_manager_api.git
cd project_manager_api
```

### 2. Configure Environment Variables

Create a `.env` file in the root directory of the project:

```env
JWT_SECRET=replace_with_a_secure_secret
SEED_ADMIN_EMAIL=admin@example.com
SEED_ADMIN_PASSWORD=replace_with_a_strong_password
SEED_ADMIN_FULLNAME=Administrator
```

Replace the example values with your own configuration. Never commit real secrets to the repository.

### 3. Initialize the Database and Create the Admin Account

```bash
cargo run -- seed-admin
```

### 4. Start the Server

```bash
cargo run
```

The API server will be available at:

http://127.0.0.1:8080

## API Endpoints

All endpoints that require authentication must include a valid access token in the `Authorization` header:

```http
Authorization: Bearer <access_token>
```

## Database Management

### Reset the Database

To reset the database and recreate all tables, run:

```bash
cargo run -- reset-db
```

**Warning:** This command permanently deletes the existing database and all stored data, including users, projects, tasks, and refresh tokens.

Stop the API server before resetting the database.

After resetting the database, create the administrator account again:

```bash
cargo run -- seed-admin
```

Make sure your `.env` file contains the required administrator credentials before running the seed command.

### Authentication

| Method | Endpoint | Description |
|---|---|---|
| POST | `/auth/register` | Register a new user |
| POST | `/auth/login` | Authenticate a user and obtain tokens |
| POST | `/auth/refresh` | Refresh access and refresh tokens |
| POST | `/auth/logout` | Revoke a refresh token |
| GET | `/auth/me` | Get the current user's information |
| PUT | `/auth/me/password` | Change the current user's password |

### Users

| Method | Endpoint | Description |
|---|---|---|
| GET | `/users` | List users (Admin only) |
| GET | `/users/{id}` | Get a user by ID |
| PATCH | `/users/{id}/role` | Update a user's role (Admin only) |
| PATCH | `/users/{id}/status` | Activate or deactivate a user (Admin only) |

### Projects

| Method | Endpoint | Description |
|---|---|---|
| POST | `/projects` | Create a project |
| GET | `/projects` | List accessible projects |
| GET | `/projects/{id}` | Get project details |
| PATCH | `/projects/{id}` | Update a project |
| DELETE | `/projects/{id}` | Delete a project |
| POST | `/projects/{id}/members` | Add a member to a project |
| DELETE | `/projects/{id}/members/{userId}` | Remove a member from a project |

### Tasks

| Method | Endpoint | Description |
|---|---|---|
| POST | `/projects/{id}/tasks` | Create a task in a project |
| GET | `/projects/{id}/tasks` | List tasks in a project |
| GET | `/tasks/{id}` | Get task details |
| PATCH | `/tasks/{id}` | Update task information |
| PATCH | `/tasks/{id}/status` | Update a task's status |
| DELETE | `/tasks/{id}` | Delete a task |

### Query Parameters

Project and task listing endpoints support pagination using `page` and `limit`.

The task listing endpoint also supports filtering by `status` and `assignee_id`.

Example:

```http
GET /projects/1/tasks?page=1&limit=10&status=todo
```

Valid task statuses are:

- `todo`
- `in_progress`
- `done`

### Access Control

Access to protected endpoints depends on the user's role and permissions.

- **Admin:** Manages users and has broad access to project and task operations.
- **Manager:** Creates and manages their own projects and their tasks.
- **Member:** Accesses permitted projects and tasks and can update the status of assigned tasks.

## Project Structure

```text
project_manager_api/
├── database/
│   ├── database.rs
│   ├── users.rs
│   ├── refresh_tokens.rs
│   ├── projects.rs
│   └── tasks.rs
├── src/
│   ├── routes/
│   │   ├── auth_routes.rs
│   │   ├── users_routes.rs
│   │   ├── projects_routes.rs
│   │   └── tasks_routes.rs
│   ├── auth.rs
│   ├── auth_middleware.rs
│   ├── lib.rs
│   ├── main.rs
│   ├── routes.rs
│   ├── seed.rs
│   └── token.rs
├── tests/
│   ├── auth_api.rs
│   └── users_api.rs
├── Cargo.toml
├── Cargo.lock
└── README.md
```

## Usage Examples

The following examples use `curl` to send HTTP requests to the API.

### 1. Register a User

```bash
curl -X POST http://127.0.0.1:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "your_password",
    "fullname": "Example User"
  }'
```

### 2. Login

```bash
curl -X POST http://127.0.0.1:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "your_password"
  }'
```

A successful login returns an access token and a refresh token.

### 3. Get the Current User

Replace `<access_token>` with the access token returned by the login endpoint.

```bash
curl http://127.0.0.1:8080/auth/me \
  -H "Authorization: Bearer <access_token>"
```

### 4. Refresh Tokens

Send your current refresh token to obtain a new pair of tokens.

```bash
curl -X POST http://127.0.0.1:8080/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "<refresh_token>"
  }'
```

### 5. Logout

Send the refresh token to log out and revoke the token.

```bash
curl -X POST http://127.0.0.1:8080/auth/logout \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "<refresh_token>"
  }'
```

## Future Improvements

If I had more time, I would:

- Complete the remaining automated tests.
- Refactor some functions related to access-token validation.
- Improve the handling of inactive accounts to ensure they cannot access protected endpoints.