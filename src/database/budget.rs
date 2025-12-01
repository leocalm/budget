use crate::error::app_error::AppError;
use crate::models::budget::{Budget, BudgetRequest};
use deadpool_postgres::Client;
use tokio_postgres::Row;
use uuid::Uuid;

pub async fn create_budget(client: &Client, request: &BudgetRequest) -> Result<Budget, AppError> {
    let rows = client
        .query(
            r#"
            INSERT INTO budget (name, start_day)
            VALUES ($1, $2)
            RETURNING id, name, start_day, created_at, deleted, deleted_at
            "#,
            &[&request.name, &(request.start_day as i32)],
        )
        .await?;

    if let Some(row) = rows.first() {
        Ok(map_row_to_budget(row))
    } else {
        Err(AppError::Db("Error mapping created budget".to_string()))
    }
}

pub async fn get_budget_by_id(client: &Client, id: &Uuid) -> Result<Option<Budget>, AppError> {
    let rows = client
        .query(
            r#"
            SELECT id, name, start_day, created_at, deleted, deleted_at
            FROM budget
            WHERE id = $1
                AND deleted = false
            "#,
            &[id],
        )
        .await?;

    if let Some(row) = rows.first() {
        Ok(Some(map_row_to_budget(row)))
    } else {
        Ok(None)
    }
}

pub async fn list_budgets(client: &Client) -> Result<Vec<Budget>, AppError> {
    let rows = client
        .query(
            r#"
            SELECT id, name, start_day, created_at, deleted, deleted_at
            FROM budget
            WHERE deleted = false
            ORDER BY created_at DESC
            "#,
            &[],
        )
        .await?;

    Ok(rows.into_iter().map(|r| map_row_to_budget(&r)).collect())
}

pub async fn delete_budget(client: &Client, id: &Uuid) -> Result<(), AppError> {
    client
        .execute(
            r#"
            UPDATE budget
            SET deleted = true,
                deleted_at = now()
            WHERE id = $1
            "#,
            &[id],
        )
        .await?;
    Ok(())
}

pub async fn update_budget(
    client: &Client,
    id: &Uuid,
    budget: &BudgetRequest,
) -> Result<Budget, AppError> {
    let rows = client
        .query(
            r#"
            UPDATE budget
            SET name = $1, start_day = $2
            WHERE id = $3
            RETURNING id, name, start_day, created_at, deleted, deleted_at
            "#,
            &[&budget.name, &budget.start_day, &id],
        )
        .await?;

    if let Some(row) = rows.first() {
        Ok(map_row_to_budget(row))
    } else {
        Err(AppError::Db("Error mapping created budget".to_string()))
    }
}

fn map_row_to_budget(row: &Row) -> Budget {
    Budget {
        id: row.get("id"),
        name: row.get("name"),
        start_day: row.get::<_, i32>("start_day"),
        created_at: row.get("created_at"),
        deleted: row.get("deleted"),
        deleted_at: row.get("deleted_at"),
    }
}
