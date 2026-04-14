# quotes-api

A production-quality REST API built in Rust, intended to serve as the data layer for [quotes.cmon1975.com](https://quotes.cmon1975.com).

## Stack

- **axum**: HTTP framework
- **sqlx** + **SQLite**: async database access with compile-time query verification
- **tokio**: async runtime
- **tracing** + **tracing-subscriber**: structured logging

## Architecture

```
src/
├── main.rs       # Server startup, wiring
├── config.rs     # Environment-based configuration
├── routes.rs     # HTTP handlers and router
├── db.rs         # Database access functions
├── models.rs     # Data structures
├── auth.rs       # API key authentication
└── errors.rs     # Typed error responses
```

## Endpoints

| Method | Path          | Auth | Description                                        |
| ------ | ------------- | ---- | -------------------------------------------------- |
| GET    | `/quotes`     | No   | List quotes with optional filtering and pagination |
| GET    | `/quotes/:id` | No   | Single quote by ID                                 |
| POST   | `/quotes`     | Yes  | Add a quote                                        |
| PUT    | `/quotes/:id` | Yes  | Update a quote                                     |
| DELETE | `/quotes/:id` | Yes  | Delete a quote                                     |

### Query Parameters — `GET /quotes`

| Parameter  | Type    | Description                                        |
| ---------- | ------- | -------------------------------------------------- |
| `author`   | string  | Filter by author (case-insensitive, partial match) |
| `tag`      | string  | Filter by tag (exact match)                        |
| `page`     | integer | Page number (default: 1)                           |
| `per_page` | integer | Results per page (default: 10, max: 100)           |

## Setup

### Prerequisites

- Rust (edition 2024)
- sqlx-cli: `cargo install sqlx-cli --no-default-features --features sqlite`

### Running locally

```bash
cp .env.example .env
# Edit .env and set a secure API_KEY value
sqlx database create
sqlx migrate run
cargo run
```

### Running tests

```bash
cargo test
```

## Authentication

Protected routes require a bearer token in the `Authorization` header:

`Authorization: Bearer <your-api-key>`

The key is set via the `API_KEY` environment variable.

## Error Responses

All errors return a consistent JSON shape:

```json
{
    "error": {
        "code": "NOT_FOUND",
        "message": "resource not found
    }
}
```

## Development Approach

This project was built following strict TDD: tests were written before or alongside each implementation. All database tests use an in-memory SQLite pool isolated per test.