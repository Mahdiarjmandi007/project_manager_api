# Project Manager API — API Reference

This document describes the HTTP endpoints currently exposed by the Project Manager API.

## 1. General Information

- **Base URL:** `http://127.0.0.1:8080`
- **Content type:** `application/json` for JSON request and response bodies, unless otherwise noted.
- **Path parameters:** `{id}`, `{userId}` and `{taskId}` represent integer IDs. In the routed task endpoints, `{id}` in `/tasks/{id}` is the task ID.
- **Authentication:** Protected endpoints use a JWT access token in the `Authorization` header:

```http
Authorization: Bearer <access_token>
```

The current implementation creates access tokens that last **15 minutes** and refresh tokens that last **7 days**.

## 2. Roles

| Role | General permissions |
|---|---|
| `admin` | User administration and broad access to projects and tasks. |
| `manager` | Create projects; manage projects, members, and tasks for projects they own. They may view projects/tasks they belong to. |
| `member` | View projects/tasks they belong to and change the status of tasks assigned to them. |

A new public registration creates a `member`. The first administrator must be created through the seed command, not public registration.

## 3. Authentication Endpoints

### 3.1 Register

`POST /auth/register`

Creates a new member account.

**Authentication:** Not required.

**Request body**

```json
{
  "email": "user@example.com",
  "password": "example-password",
  "fullname": "Example User"
}
```

| Field | Type | Required | Notes |
|---|---|---:|---|
| `email` | string | Yes | User email. Must be unique. |
| `password` | string | Yes | At least 8 characters. |
| `fullname` | string | Yes | User's full name. |

**Success — `201 Created`**

```json
{
  "message": "user registered successfully"
}
```

**Main errors:**

- `400 Bad Request` — password is shorter than 8 characters.
- `409 Conflict` — email already exists.
- `500 Internal Server Error` — unexpected database or server error.

### 3.2 Login

`POST /auth/login`

Checks the supplied credentials and returns an access token and a refresh token.

**Authentication:** Not required.

**Request body**

```json
{
  "email": "user@example.com",
  "password": "example-password"
}
```

**Success — `200 OK`**

```json
{
  "message": "user logged in successfully",
  "access_token": "<access_token>",
  "refresh_token": "<refresh_token>"
}
```

**Main errors:** `401 Unauthorized` for credentials that cannot be accepted; `500 Internal Server Error` for unexpected errors. The current implementation may return different messages for an unknown email and an incorrect password.

### 3.3 Refresh Tokens

`POST /auth/refresh`

Validates a refresh token, revokes the old token, and returns a new access/refresh token pair.

**Authentication:** Send the refresh token in the JSON body; no access-token header is required.

**Request body**

```json
{
  "refresh_token": "<refresh_token>"
}
```

**Success — `200 OK`**

```json
{
  "message": "user logged in successfully",
  "access_token": "<new_access_token>",
  "refresh_token": "<new_refresh_token>"
}
```

Use the new refresh token from the response; the previous refresh token is revoked.

**Main errors:** `401 Unauthorized` for an invalid, expired, revoked, or otherwise unacceptable refresh token; `401 Unauthorized` if the associated account is inactive; `500 Internal Server Error` for an unexpected error.

### 3.4 Logout

`POST /auth/logout`

Revokes the supplied refresh token.

**Authentication:** Send the refresh token in the JSON body.

**Request body**

```json
{
  "refresh_token": "<refresh_token>"
}
```

**Success — `200 OK`**

```json
{
  "message": "user logged out successfully"
}
```

**Main errors:** `401 Unauthorized` if the refresh token is invalid, expired, or already revoked; `500 Internal Server Error` for an unexpected error.

### 3.5 Get Current User

`GET /auth/me`

Returns information about the user associated with the access token.

**Authentication:** Required.

**Success — `200 OK`**

```json
{
  "id": 12,
  "email": "user@example.com",
  "fullname": "Example User",
  "role": "member"
}
```

**Main errors:** `401 Unauthorized` for a missing, malformed, invalid, or expired access token, or if the token subject does not identify an existing user; `403 Forbidden` if the account is inactive; `500 Internal Server Error` for an unexpected database error.

### 3.6 Change Current User's Password

`PUT /auth/me/password`

Changes the authenticated user's password after verifying the current password. On success, all existing refresh tokens for that user are revoked.

**Authentication:** Required.

**Request body**

```json
{
  "current_password": "old-password",
  "new_password": "new-password-123"
}
```

**Success — `200 OK`**

```json
{
  "message": "password changed successfully"
}
```

**Main errors:** `400 Bad Request` if the new password is shorter than 8 characters; `401 Unauthorized` if the access token or current password is invalid; `403 Forbidden` if the account is inactive; `500 Internal Server Error` for an unexpected error.

## 4. User Endpoints

All endpoints in this section require an access token.

### 4.1 List Users

`GET /users?limit=20&offset=0`

Returns a paginated user list. **Admin only.**

**Query parameters**

| Parameter | Default | Rules |
|---|---:|---|
| `limit` | `20` | Must be between `1` and `20`. |
| `offset` | `0` | Must be `0` or greater. |

**Success — `200 OK`**

The response is a JSON array:

```json
[
  {
    "id": 12,
    "email": "user@example.com",
    "fullname": "Example User",
    "role": "member",
    "is_active": true,
    "created_at": "2026-10-16 12:30:00"
  }
]
```

The password and password hash are not returned.

**Main errors:** `400 Bad Request` for invalid pagination; `401 Unauthorized` for invalid/missing access token; `403 Forbidden` for non-admin users; `500 Internal Server Error` for a database error.

### 4.2 Get User by ID

`GET /users/{id}`

**Allowed:** Admin, or the user whose ID is in the path.

**Success — `200 OK`**

```json
{
  "id": 12,
  "email": "user@example.com",
  "fullname": "Example User",
  "role": "member",
  "is_active": true,
  "created_at": "2026-10-16 12:30:00"
}
```

**Main errors:** `401 Unauthorized` for invalid/missing token; `403 Forbidden` if a non-admin attempts to view another user; `404 Not Found` if the user does not exist; `500 Internal Server Error` for a database error.

### 4.3 Change User Role

`PATCH /users/{id}/role`

Changes a user's role. **Admin only.** An administrator cannot change their own role through this endpoint.

**Request body**

```json
{
  "role": "manager"
}
```

Allowed values: `admin`, `manager`, `member`.

**Success — `200 OK`**

```json
{
  "message": "user role updated successfully",
  "user_id": 12,
  "role": "manager"
}
```

**Main errors:** `400 Bad Request` for an invalid role; `401 Unauthorized` for an invalid/missing token; `403 Forbidden` for non-admin users or an attempt to change one's own role; `404 Not Found` if the target user does not exist; `500 Internal Server Error` for a database error.

### 4.4 Change User Status

`PATCH /users/{id}/status`

Activates or deactivates an account. **Admin only.** An administrator cannot deactivate their own account.

**Request body**

```json
{
  "status": false
}
```

- `true`: active account.
- `false`: inactive account.

**Success — `200 OK`**

```json
{
  "message": "user status updated successfully",
  "user_id": 12
}
```

**Main errors:** `401 Unauthorized` for invalid/missing token; `403 Forbidden` for non-admin users or an attempt to deactivate oneself; `404 Not Found` if the target user does not exist; `500 Internal Server Error` for a database error.

## 5. Project Endpoints

All endpoints in this section require an access token.

### 5.1 Create Project

`POST /projects`

**Allowed:** Admin and manager.

**Request body**

```json
{
  "name": "Website Redesign",
  "description": "Redesign the company website"
}
```

`description` may be omitted or set to `null`.

**Success — `201 Created`**

```json
{
  "message": "project created successfully",
  "project_id": 7
}
```

The creator becomes the project owner and is added to the project membership list.

**Main errors:** `400 Bad Request` if the project name is empty; `401 Unauthorized` for invalid/missing token; `403 Forbidden` for members; `500 Internal Server Error` for a database error.

### 5.2 List Accessible Projects

`GET /projects?page=1&limit=10`

Returns projects visible to the authenticated user. Admins can list all projects; other roles receive projects from their memberships.

**Query parameters**

| Parameter | Default | Rules |
|---|---:|---|
| `page` | `1` | Must be `1` or greater. |
| `limit` | `10` | Must be between `1` and `100`. |

**Success — `200 OK`**

```json
{
  "projects": [
    {
      "id": 7,
      "name": "Website Redesign",
      "description": "Redesign the company website",
      "owner_id": 12,
      "created_at": "2026-10-16 12:30:00"
    }
  ],
  "page": 1,
  "limit": 10
}
```

**Main errors:** `400 Bad Request` for invalid pagination; `401 Unauthorized` for invalid/missing token; `500 Internal Server Error` for a database error.

### 5.3 Get Project by ID

`GET /projects/{id}`

**Allowed:** Admin or a member of the project. The project creator is automatically added as a member.

**Success — `200 OK`**

```json
{
  "id": 7,
  "name": "Website Redesign",
  "description": "Redesign the company website",
  "owner_id": 12,
  "created_at": "2026-10-16 12:30:00"
}
```

**Main errors:** `401 Unauthorized` for invalid/missing token; `403 Forbidden` if the user cannot access the project; `404 Not Found` if the project does not exist; `500 Internal Server Error` for a database error.

### 5.4 Update Project

`PATCH /projects/{id}`

**Allowed:** Admin or the manager who owns the project.

**Request body** — send at least one field:

```json
{
  "name": "Website Redesign Phase 2",
  "description": "Updated project description"
}
```

Both fields are optional individually; an empty body is rejected. A supplied name cannot be blank.

**Success — `200 OK`**

```json
{
  "message": "project updated successfully"
}
```

**Main errors:** `400 Bad Request` for no fields or an empty name; `401 Unauthorized` for invalid/missing token; `403 Forbidden` if the caller is not allowed to edit the project; `404 Not Found` if the project does not exist; `500 Internal Server Error` for a database error.

### 5.5 Delete Project

`DELETE /projects/{id}`

**Allowed:** Admin or the manager who owns the project.

**Success — `204 No Content`**

No response body. The current implementation deletes the project, its membership records, and its tasks.

**Main errors:** `401 Unauthorized` for invalid/missing token; `403 Forbidden` if the caller is not allowed to delete the project; `404 Not Found` if the project does not exist; `500 Internal Server Error` for a database error.

### 5.6 Add a Project Member

`POST /projects/{id}/members`

**Allowed:** Admin or the manager who owns the project.

**Request body**

```json
{
  "user_id": 15
}
```

**Success — `201 Created`**

```json
{
  "message": "project member added successfully",
  "project_id": 7,
  "user_id": 15
}
```

**Main errors:** `400 Bad Request` for a non-positive user ID; `401 Unauthorized` for invalid/missing token; `403 Forbidden` if the caller cannot manage the project; `404 Not Found` if the project or user does not exist; `409 Conflict` if the user is already a member; `500 Internal Server Error` for a database error.

### 5.7 Remove a Project Member

`DELETE /projects/{id}/members/{userId}`

**Allowed:** Admin or the manager who owns the project. The project owner cannot be removed.

**Success — `204 No Content`**

No response body.

**Main errors:** `401 Unauthorized` for invalid/missing token; `403 Forbidden` if the caller cannot manage the project; `404 Not Found` if the project or membership does not exist; `409 Conflict` if attempting to remove the project owner; `500 Internal Server Error` for a database error.

## 6. Task Endpoints

All endpoints in this section require an access token.

### 6.1 Create Task in a Project

`POST /projects/{id}/tasks`

**Allowed:** Admin or the manager who owns the project.

**Request body**

```json
{
  "title": "Create login page",
  "description": "Build the login form and validation",
  "assignee_id": 15
}
```

- `title` is required and cannot be blank.
- `description` is optional.
- `assignee_id` is optional. If provided, the user must be a member of the same project.
- New tasks start with status `todo`.

**Success — `201 Created`**

```json
{
  "message": "Task created successfully",
  "task_id": 31
}
```

**Main errors:** `400 Bad Request` for a blank title; `401 Unauthorized` for invalid/missing token; `403 Forbidden` if the caller cannot create tasks in the project; `404 Not Found` if the project does not exist; `422 Unprocessable Entity` if the assignee is not a project member; `500 Internal Server Error` for a database error.

### 6.2 List Project Tasks

`GET /projects/{id}/tasks?page=1&limit=10&status=todo&assignee_id=15`

Returns tasks in a project the user is allowed to access.

**Query parameters**

| Parameter | Default | Description / rules |
|---|---:|---|
| `page` | `1` | Must be `1` or greater. |
| `limit` | `10` | Must be between `1` and `100`. |
| `status` | No filter | Optional; `todo`, `in_progress`, or `done`. |
| `assignee_id` | No filter | Optional integer user ID. |

**Success — `200 OK`**

```json
{
  "tasks": [
    {
      "id": 31,
      "project_id": 7,
      "title": "Create login page",
      "description": "Build the login form and validation",
      "status": "todo",
      "assignee_id": 15,
      "created_by": 12,
      "created_at": "2026-10-16 12:30:00",
      "updated_at": "2026-10-16 12:30:00"
    }
  ],
  "page": 1,
  "limit": 10
}
```

**Main errors:** `400 Bad Request` for invalid pagination or status; `401 Unauthorized` for invalid/missing token; `403 Forbidden` if the caller cannot access the project; `404 Not Found` if the project does not exist; `500 Internal Server Error` for a database error.

### 6.3 Get Task by ID

`GET /tasks/{id}`

**Allowed:** Admin; managers who own or belong to the project's membership list; members who belong to the project.

**Success — `200 OK`**

```json
{
  "id": 31,
  "project_id": 7,
  "title": "Create login page",
  "description": "Build the login form and validation",
  "status": "todo",
  "assignee_id": 15,
  "created_by": 12,
  "created_at": "2026-10-16 12:30:00",
  "updated_at": "2026-10-16 12:30:00"
}
```

**Main errors:** `401 Unauthorized` for invalid/missing token; `403 Forbidden` if the caller cannot access the task's project; `404 Not Found` if the task does not exist; `500 Internal Server Error` for a database error.

### 6.4 Update Task Details

`PATCH /tasks/{id}`

**Allowed:** Admin or the manager who owns the task's project.

**Request body** — at least one field must be provided:

```json
{
  "title": "Finish login page",
  "description": "Include validation and error messages",
  "assignee_id": 15
}
```

All fields are optional individually. If `assignee_id` is supplied with a user ID, that user must belong to the task's project. In the current implementation, sending `assignee_id: null` is treated like an omitted value, so this endpoint cannot be used to unassign a task.

**Success — `200 OK`**

```json
{
  "message": "Task updated successfully"
}
```

**Main errors:** `400 Bad Request` if no fields are supplied or title is blank; `401 Unauthorized` for invalid/missing token; `403 Forbidden` if the caller cannot edit the task; `404 Not Found` if the task does not exist; `422 Unprocessable Entity` if the assignee is not a project member; `500 Internal Server Error` for a database error.

### 6.5 Update Task Status

`PATCH /tasks/{id}/status`

**Allowed:** Admin; the manager who owns the project; or the member assigned to the task.

**Request body**

```json
{
  "status": "in_progress"
}
```

Allowed values: `todo`, `in_progress`, `done`.

**Success — `200 OK`**

```json
{
  "message": "Task status updated successfully"
}
```

**Main errors:** `400 Bad Request` for an invalid status; `401 Unauthorized` for invalid/missing token; `403 Forbidden` if the caller is not allowed to change the task's status; `404 Not Found` if the task does not exist; `500 Internal Server Error` for a database error.

### 6.6 Delete Task

`DELETE /tasks/{id}`

**Allowed:** Admin or the manager who owns the task's project.

**Success — `204 No Content`**

No response body.

**Main errors:** `401 Unauthorized` for invalid/missing token; `403 Forbidden` if the caller cannot delete the task; `404 Not Found` if the task does not exist; `500 Internal Server Error` for a database error.

## 7. Root Endpoint

### Health / Welcome Message

`GET /`

No authentication required.

**Success — `200 OK`**

Returns the plain-text response:

```text
Project Manager API is running!
```

## 8. Common HTTP Status Codes

| Status | Meaning |
|---|---|
| `200 OK` | Request completed successfully. |
| `201 Created` | A resource was created successfully. |
| `204 No Content` | Request completed successfully without a response body. |
| `400 Bad Request` | Request input or query parameters are invalid. |
| `401 Unauthorized` | Authentication is missing or invalid, or credentials/token cannot be accepted. |
| `403 Forbidden` | The user is authenticated but lacks permission for the operation. |
| `404 Not Found` | The requested resource does not exist. |
| `409 Conflict` | The request conflicts with existing data, such as duplicate project membership. |
| `422 Unprocessable Entity` | The request is structurally valid but violates a business rule, such as assigning a task to a non-member. |
| `500 Internal Server Error` | An unexpected server or database error occurred. |

Error responses currently are not fully standardized: many handlers return JSON with a `message` field, while some return plain text or an empty body. Clients should not assume every error has exactly the same JSON structure.

## 9. cURL Examples

### Login and save the access token

```bash
curl -X POST http://127.0.0.1:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"example-password"}'
```

### Get the current user

```bash
curl http://127.0.0.1:8080/auth/me \
  -H "Authorization: Bearer <access_token>"
```

### Create a project

```bash
curl -X POST http://127.0.0.1:8080/projects \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <access_token>" \
  -d '{"name":"Website Redesign","description":"Redesign the company website"}'
```

### Create a task

```bash
curl -X POST http://127.0.0.1:8080/projects/7/tasks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <access_token>" \
  -d '{"title":"Create login page","description":"Build the login form","assignee_id":15}'
```

## 10. Current Implementation Notes

- The automated test suite does not yet cover every endpoint and authorization scenario. Treat this document as a reference to the current code, not as proof that all behavior has been verified by integration tests.
- The current login implementation does not check `is_active` before issuing tokens. The shared `authenticate` helper validates the JWT but does not itself reload the account's current `is_active` status; therefore, a deactivated user may still be able to use an existing access token on some protected endpoints. This should be fixed before production use.
- The API does not yet return a single consistent error-response format across all handlers.
- Task update currently cannot clear/unset an assignee by sending `"assignee_id": null`.
- No OpenAPI/Swagger specification is included yet.
