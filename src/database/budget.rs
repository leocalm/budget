use crate::error::app_error::AppError;
use crate::models::budget::Budget;
use deadpool_postgres::Client;

pub async fn get_all_budgets(client: &Client) -> Result<Vec<Budget>, AppError> {
    let rows = client
        .query(
            r#"
        SELECT id, name
        FROM budgets
        ORDER BY created_at DESC
        "#,
            &[],
        )
        .await?;

    Ok(rows
        .into_iter()
        .map(|r| Budget {
            id: r.get("id"),
            name: r.get("name"),
        })
        .collect())
}

pub async fn create_budget(client: &Client, name: &str) -> Result<Option<Budget>, AppError> {
    let rows = client
        .query(
            r#"
        INSERT INTO budgets (name)
        VALUES ($1)
        RETURNING id, name
        "#,
            &[&name],
        )
        .await?;

    if let Some(row) = rows.first() {
        Ok(Some(Budget {
            id: row.get("id"),
            name: row.get("name"),
        }))
    } else {
        Ok(None)
    }
}
