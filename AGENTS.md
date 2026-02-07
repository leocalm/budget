# AGENTS.md

This file provides guidance to AI coding agents when working with code in this repository.

## Project Overview

This is a Rust-based budgeting API built with Rocket web framework and PostgreSQL. The application provides a RESTful API for managing personal finances,
including budgets, transactions, accounts, categories, and vendors.

## Build and Development Commands

```bash
# Run the API server (default port 8000)
cargo run

# Build the project
cargo build

# Run in release mode
cargo build --release && cargo run --release

# Format code (max line width: 160)
cargo fmt

# Lint code
cargo clippy --workspace --all-targets -- -D warnings

# Run tests
cargo test

# Run specific test
cargo test <test_name>
```

## Configuration

Configuration is loaded via `figment` in priority order (highest wins):

1. **Rocket environment variables** (prefixed with `ROCKET_`) — takes precedence for Rocket-specific settings
2. **Budget environment variables** (prefixed with `BUDGET_`) — use `__` to separate nested keys (e.g. `BUDGET__DATABASE__URL`)
3. `Budget.toml` in the project root
4. Compiled-in defaults

**Important**: Rocket's server configuration (`address`, `port`) MUST be set via `ROCKET_ADDRESS` and `ROCKET_PORT` environment variables, not via `BUDGET__SERVER__*` variables.

Key sections and their defaults:

| Section | Key | Default | Environment Variable |
|---|---|---|---|
| `[database]` | `url` | `postgres://localhost/budget_db` | `BUDGET__DATABASE__URL` |
| | `max_connections` | 16 | `BUDGET__DATABASE__MAX_CONNECTIONS` |
| | `min_connections` | 4 | `BUDGET__DATABASE__MIN_CONNECTIONS` |
| | `connection_timeout` | 5 s | `BUDGET__DATABASE__CONNECTION_TIMEOUT` |
| | `acquire_timeout` | 5 s | `BUDGET__DATABASE__ACQUIRE_TIMEOUT` |
| `[server]` | `port` | 8000 | `ROCKET_PORT` ⚠️ |
| | `address` | `127.0.0.1` | `ROCKET_ADDRESS` ⚠️ |
| `[logging]` | `level` | `info` | `BUDGET__LOGGING__LEVEL` |
| | `json_format` | `false` | `BUDGET__LOGGING__JSON_FORMAT` |
| `[cors]` | `allowed_origins` | `["*"]` | `BUDGET__CORS__ALLOWED_ORIGINS` |
| | `allow_credentials` | `false` | `BUDGET__CORS__ALLOW_CREDENTIALS` |
| `[rate_limit]` | `read_limit` | 300 | `BUDGET__RATE_LIMIT__READ_LIMIT` |
| | `mutation_limit` | 60 | `BUDGET__RATE_LIMIT__MUTATION_LIMIT` |
| | `auth_limit` | 10 | `BUDGET__RATE_LIMIT__AUTH_LIMIT` |
| | `window_seconds` | 60 | `BUDGET__RATE_LIMIT__WINDOW_SECONDS` |
| | `cleanup_interval_seconds` | 60 | `BUDGET__RATE_LIMIT__CLEANUP_INTERVAL_SECONDS` |
| | `require_client_ip` | `true` | `BUDGET__RATE_LIMIT__REQUIRE_CLIENT_IP` |
| `[api]` | `base_path` | `/api/v1` | `BUDGET__API__BASE_PATH` |
| | `additional_base_paths` | `[]` | `BUDGET__API__ADDITIONAL_BASE_PATHS` |

> Wildcard origins (`*`) combined with `allow_credentials = true` is an invalid combination and will panic at startup.

### Docker Deployment Configuration

When running in Docker:

1. **Server Address**: MUST be set to `0.0.0.0` for inter-container communication
   ```bash
   ROCKET_ADDRESS=0.0.0.0
   ROCKET_PORT=8000
   ```

2. **Database URL**: Use Docker service name `db`, not `localhost`
   ```toml
   url = "postgres://postgres:password@db:5432/budget_db"
   ```

3. **Budget.toml in Container**: The `Budget.toml` file is copied into the Docker image and provides baseline defaults. Environment variables override these settings.

## Database Setup

Migrations are managed by sqlx-cli. Each migration lives in its own directory under `migrations/`
with `up.sql` (apply) and `down.sql` (rollback). Install sqlx-cli and apply:

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
sqlx migrate run       # apply all pending migrations
sqlx migrate revert    # roll back the last migration
sqlx migrate info      # show migration status
```

When adding a new migration:

```bash
sqlx migrate add <description>   # creates migrations/NNNN_description/{up,down}.sql
```

### Docker Database Migrations

When running in Docker, migrations are not automatically applied. Run them manually:

```bash
# Apply all migrations in order
docker-compose exec -T backend cat /app/migrations/0001_init/up.sql | \
  docker-compose exec -T db psql -U postgres -d budget_db

docker-compose exec -T backend cat /app/migrations/0002_add_indexes/up.sql | \
  docker-compose exec -T db psql -U postgres -d budget_db

docker-compose exec -T backend cat /app/migrations/0003_medium_security/up.sql | \
  docker-compose exec -T db psql -U postgres -d budget_db

# Apply any additional migrations
docker-compose exec -T backend cat /app/migrations/*.sql | \
  docker-compose exec -T db psql -U postgres -d budget_db
```

## Architecture

### Layered Architecture Pattern

The codebase keeps a simple separation of concerns:

1. **Routes Layer** (`src/routes/`): Rocket handlers for HTTP I/O.
2. **Service Layer** (`src/service/`): Light business logic helpers (e.g., account aggregation, dashboard calculations).
3. **Database Layer** (`src/database/`): Concrete data access methods implemented directly on `PostgresRepository`.

### Repository Implementation (concrete, no traits)

There are **no repository traits**. Each `src/database/<entity>.rs` file implements `impl PostgresRepository { ... }` with async methods for that entity (CRUD, queries, helpers).

Benefits:
- Less boilerplate and indirection.
- Callers (routes/services) use the concrete repository directly.
- Tests rely on pure helper functions and sample data instead of mock trait impls.

### Database Connection Management

- Uses `sqlx::PgPool` configured in `src/db.rs` via a Rocket `AdHoc` fairing (`stage_db`).
- Pool options (`max_connections`, `min_connections`, `acquire_timeout`) come from `DatabaseConfig`. Additional hard-coded limits: idle timeout 30 s, max lifetime 1800 s.
- Routes receive `&State<PgPool>`, then construct `PostgresRepository { pool: pool.inner().clone() }`.
- No `deadpool-postgres` or trait objects involved.
- All repository methods receive `&current_user.id` and scope every query to that user.

### Authentication

- Cookie-based authentication implemented in `src/auth.rs` via the `CurrentUser` request guard (`FromRequest`).
- The guard reads the private (encrypted) `user` cookie. Expected format: `<uuid>:<username>`. Returns `401 Unauthorized` if the cookie is missing or unparseable.
- `CurrentUser.id` is threaded into every repository call to scope queries to the authenticated user.

### Domain Models

Models are split into two types in `src/models/<entity>.rs`:

- Domain models (e.g., `Budget`, `Transaction`, `Account`) representing database entities
- Request/Response DTOs (e.g., `BudgetRequest`, `BudgetResponse`) for API serialization

### API Endpoints Structure

All endpoints are mounted under `/api/v1` by default (configurable via `api.base_path`). The same routes can be exposed under additional base paths via `api.additional_base_paths`. The examples below assume the default base path. List endpoints use cursor-based pagination (see Pagination below).

- `/api/v1/health` — `GET /` runs `SELECT 1` against the pool; returns `{"status":"ok","database":"connected"}` or `503`
- `/api/v1/users` — create, login, logout, update, delete, `GET /me`
- `/api/v1/accounts` — CRUD + cursor-paginated list; list requires mandatory `period_id` query parameter to filter accounts by budget period. Returns 400 if `period_id` is missing ("Missing period_id query parameter") or invalid.
- `/api/v1/currency` — CRUD; lookup by code (`GET /<code>`) or name (`GET /name/<name>`)
- `/api/v1/categories` — CRUD + cursor-paginated list; `GET /not-in-budget` returns Outgoing categories not yet associated with a budget
- `/api/v1/budgets` — CRUD + cursor-paginated list
- `/api/v1/budget-categories` — CRUD + cursor-paginated list
- `/api/v1/budget_period` — CRUD + cursor-paginated list; `GET /current` returns the period whose date range covers today
- `/api/v1/transactions` — CRUD + cursor-paginated list; list accepts optional `period_id` query filter
- `/api/v1/vendors` — CRUD + cursor-paginated list; `GET /with_status?order_by=<name|most_used|more_recent>` returns vendors enriched with transaction-count stats
- `/api/v1/dashboard` — `budget-per-day`, `spent-per-category`, `monthly-burn-in`, `month-progress`, `recent-transactions`, `dashboard` (all accept `period_id`)
  `spent-per-category` returns `percentage_spent` in basis points (percent * 100). Example: 2534 = 25.34%.

404 and 409 responses are caught under `/api/v1` by default and returned as `{"message":"..."}` JSON.

### Pagination

List endpoints use keyset (cursor-based) pagination via `CursorParams` (`src/models/pagination.rs`):

- Query params: `cursor` (UUID of the last item on the previous page) and `limit` (default **50**, max **200**).
- Responses are wrapped in `CursorPaginatedResponse<T>` with `data` and `next_cursor` (`null` on the last page).
- The DB layer fetches `limit + 1` rows; if an extra row exists it is dropped and `next_cursor` is set to the `id` of the last returned item.
- Indexes on `(user_id, created_at DESC, id DESC)` (and `start_date` for budget periods) back the cursor queries.

### Error Handling

`src/error/app_error.rs` — `AppError` enum covers DB errors, validation, not found, invalid credentials, UUID parse, password-hash, and configuration errors. Implements `Responder`: logs via `tracing::error!`, maps to the appropriate HTTP status, and returns the error message as plain-text body. Route handlers return `Result<T, AppError>`.

`src/error/json.rs` — `JsonBody<T>` is a custom `FromData` extractor used instead of Rocket's built-in `Json<T>`. On a parse failure it logs the serde error location (line/column), the error category, and a preview of the request body (up to 500 chars), then returns **422 Unprocessable Entity**.

### Testing

- Test utilities in `src/test_utils.rs` provide **sample data helpers** (`sample_account`, `sample_transaction`, etc.) and conversions from request structs to models.
- Services expose pure helper functions for deterministic unit tests (e.g., dashboard helpers).
- Most route tests that hit the database remain `#[ignore]` unless a DB is available.

## Key Implementation Patterns

### Adding a New Entity

1. Add DB table via migration.
2. Create model structs in `src/models/<entity>.rs`.
3. Add concrete methods on `PostgresRepository` in `src/database/<entity>.rs`.
4. Add route handlers in `src/routes/<entity>.rs` and mount in `src/lib.rs`.
5. Add any needed sample data helpers in `src/test_utils.rs` for unit tests.

### Route Handler Pattern

Routes construct the concrete repository directly from the pooled `PgPool`:

```rust
pub async fn handler(
    pool: &State<PgPool>,
    current_user: CurrentUser,
) -> Result<Json<Response>, AppError> {
    let repo = PostgresRepository { pool: pool.inner().clone() };
    let result = repo.some_operation(&current_user.id).await?;
    Ok(Json(Response::from(&result)))
}
```

### Database Query Pattern

Repository methods use `sqlx` with `PgPool` (no trait objects, no deadpool). Mapping is usually done with `sqlx::FromRow` structs or manual conversions.

## Important Notes

- PostgreSQL connection details come from `Config` (see Configuration section above).
- IDs are UUIDs generated by PostgreSQL `gen_random_uuid()`.
- Amounts are stored as `BIGINT` (cents) in the database and exposed as `i64` in Rust.
- Timestamps use `TIMESTAMPTZ` with `chrono::DateTime<Utc>`.
- Every query is scoped to the authenticated user via `user_id`.

## CI Discipline

Always run the full PR check suite locally before pushing:
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo build --verbose`
- `cargo test --verbose`

This mirrors `.github/workflows/rust.yml` and keeps PR checks green.

## Docker Troubleshooting

### Backend Not Accessible from Caddy (502 Bad Gateway)

**Symptom**: Caddy returns 502 errors when trying to reach the backend API.

**Cause**: Rocket is binding to `127.0.0.1` instead of `0.0.0.0`, making it inaccessible from other containers.

**Solution**:
1. Ensure `ROCKET_ADDRESS=0.0.0.0` is set in docker-compose.yaml
2. Check backend logs for: `Rocket has launched from http://0.0.0.0:8000`
3. If it shows `127.0.0.1`, the environment variable isn't being applied

**Verify**:
```bash
# Should succeed
docker-compose exec backend curl http://0.0.0.0:8000/api/v1/health

# Should succeed from Caddy container
docker-compose exec caddy wget -q -O- http://backend:8000/api/v1/health
```

### Database Connection Pool Timeout

**Symptom**: Backend crashes with "pool timed out while waiting for an open connection"

**Causes**:
1. Database URL points to `localhost` instead of `db`
2. Connection/acquire timeouts are too short
3. Database is not ready when backend starts

**Solutions**:
1. Check `Budget.toml` database URL uses `db` as hostname:
   ```toml
   url = "postgres://postgres:password@db:5432/budget_db"
   ```

2. Increase timeouts in `Budget.toml`:
   ```toml
   connection_timeout = 120
   acquire_timeout = 120
   ```

3. Verify database is healthy:
   ```bash
   docker-compose exec db pg_isready -U postgres
   ```

### Environment Variables Not Applied

**Symptom**: Configuration changes via environment variables have no effect.

**Cause**: Incorrect environment variable naming (single vs double underscores).

**Solution**: Use the correct format:
- Rocket settings: `ROCKET_ADDRESS`, `ROCKET_PORT` (single underscore prefix)
- Budget settings: `BUDGET__DATABASE__URL` (double underscores for nesting)

### Checking Actual Configuration

```bash
# View Budget.toml in running container
docker-compose exec backend cat /app/Budget.toml

# Check environment variables
docker-compose exec backend env | grep -E "(ROCKET|BUDGET)"

# View backend logs
docker-compose logs backend --tail=100
```
