# Budget API

Rust backend for a budgeting tool, using Rocket for the HTTP server and PostgreSQL via SQLx.

## Prerequisites

- Rust (stable)
- PostgreSQL running locally or remotely

Set `DATABASE_URL` in your environment, e.g.:

```bash
export DATABASE_URL=postgres://user:password@localhost:5432/budget_db
```

## Running the API

```bash
cargo run
```

The server will start (by default) on `http://127.0.0.1:8000`.

### Endpoints

- `GET /api/health` – simple health check.
- `POST /api/budgets` – create a budget.
- `GET /api/budgets` – list budgets.

## Database schema & migrations

Migrations live under the `migrations/` folder.
The initial migration `0001_init.sql` creates the `budgets` and `transactions` tables:

```sql
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS budgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    budget_id UUID NOT NULL REFERENCES budgets(id) ON DELETE CASCADE,
    amount NUMERIC(12, 2) NOT NULL,
    description TEXT,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

You can apply the migration manually, e.g. using `psql`:

```bash
psql "$DATABASE_URL" -f migrations/0001_init.sql
```

Or with a migration tool like `sqlx-cli` (optional):

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
sqlx migrate run
```
