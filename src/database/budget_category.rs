use crate::error::app_error::AppError;
use crate::models::budget_category::{BudgetCategory, BudgetCategoryRequest};
use crate::models::category::Category;
use deadpool_postgres::Client;
use tokio_postgres::Row;
use uuid::Uuid;

pub async fn create_budget_category(
    client: &Client,
    request: &BudgetCategoryRequest,
) -> Result<BudgetCategory, AppError> {
    let rows = client
        .query(
            r#"
            INSERT INTO budget_category (category_id, budgeted_value)
            VALUES ($1, $2)
            RETURNING id, category_id, budgeted_value, created_at, deleted, deleted_at
            "#,
            &[&request.category_id, &(request.budgeted_value as i32)],
        )
        .await?;

    if let Some(row) = rows.first() {
        Ok(map_row_to_budget_category(row))
    } else {
        Err(AppError::Db(
            "Error mapping created budget_category".to_string(),
        ))
    }
}

pub async fn get_budget_category_by_id(
    client: &Client,
    id: &Uuid,
) -> Result<Option<BudgetCategory>, AppError> {
    let rows = client
        .query(
            r#"
            SELECT
                bc.id,
                bc.category_id,
                bc.budgeted_value,
                bc.created_at,
                bc.deleted,
                bc.deleted_at,
                c.id as category_id,
                c.name as category_name,
                COALESCE(c.color, '') as category_color,
                COALESCE(c.icon, '') as category_icon,
                c.parent_id as category_parent_id,
                c.category_type::text as category_category_type,
                c.created_at as category_created_at,
                c.deleted as category_deleted,
                c.deleted_at as category_deleted_at
            FROM budget_category bc
            JOIN category c
                ON c.id = bc.category_id
                AND c.deleted = false
            WHERE bc.id = $1
                AND bc.deleted = false
            "#,
            &[id],
        )
        .await?;

    if let Some(row) = rows.first() {
        Ok(Some(map_row_to_budget_category(row)))
    } else {
        Ok(None)
    }
}

pub async fn list_budget_categories(client: &Client) -> Result<Vec<BudgetCategory>, AppError> {
    let rows = client
        .query(
            r#"
            SELECT
                bc.id,
                bc.category_id,
                bc.budgeted_value,
                bc.created_at,
                bc.deleted,
                bc.deleted_at,
                c.id as category_id,
                c.name as category_name,
                COALESCE(c.color, '') as category_color,
                COALESCE(c.icon, '') as category_icon,
                c.parent_id as category_parent_id,
                c.category_type::text as category_category_type,
                c.created_at as category_created_at,
                c.deleted as category_deleted,
                c.deleted_at as category_deleted_at
            FROM budget_category bc
            JOIN category c
                ON c.id = bc.category_id
                AND c.deleted = false
            WHERE bc.deleted = false
            ORDER BY bc.created_at DESC
            "#,
            &[],
        )
        .await?;

    Ok(rows
        .into_iter()
        .map(|r| map_row_to_budget_category(&r))
        .collect())
}

pub async fn delete_budget_category(client: &Client, id: &Uuid) -> Result<(), AppError> {
    client
        .execute(
            r#"
            UPDATE budget_category
            SET deleted = true,
                deleted_at = now()
            WHERE id = $1
            "#,
            &[id],
        )
        .await?;
    Ok(())
}

fn map_row_to_budget_category(row: &Row) -> BudgetCategory {
    BudgetCategory {
        id: row.get("id"),
        category_id: row.get("category_id"),
        budgeted_value: row.get::<_, i32>("budgeted_value") as u32,
        created_at: row.get("created_at"),
        deleted: row.get("deleted"),
        deleted_at: row.get("deleted_at"),
        category: Category {
            id: row.get("category_id"),
            name: row.get("category_name"),
            color: row.get("category_color"),
            icon: row.get("category_icon"),
            parent_id: row.get("category_parent_id"),
            category_type: crate::database::category::category_type_from_db(
                row.get::<_, &str>("category_category_type"),
            ),
            created_at: row.get("category_created_at"),
            deleted: row.get("category_deleted"),
            deleted_at: row.get("category_deleted_at"),
        },
    }
}
