use crate::error::app_error::AppError;
use crate::models::category::{Category, CategoryRequest, CategoryType};
use deadpool_postgres::Client;
use tokio_postgres::Row;
use uuid::Uuid;

pub async fn create_category(
    client: &Client,
    request: &CategoryRequest,
) -> Result<Category, AppError> {
    let rows = client
        .query(
            r#"
            INSERT INTO category (name, color, icon, parent_id, category_type)
            VALUES ($1, $2, $3, $4, $5::text::category_type)
            RETURNING
                id,
                name,
                COALESCE(color, '') as color,
                COALESCE(icon, '') as icon,
                parent_id,
                category_type::text as category_type,
                created_at,
                deleted,
                deleted_at
            "#,
            &[
                &request.name,
                &request.color,
                &request.icon,
                &request.parent_id,
                &request.category_type_to_db(),
            ],
        )
        .await?;

    if let Some(row) = rows.first() {
        Ok(map_row_to_category(row))
    } else {
        Err(AppError::Db("Error mapping created category".to_string()))
    }
}

pub async fn get_category_by_id(client: &Client, id: &Uuid) -> Result<Option<Category>, AppError> {
    let rows = client
        .query(
            r#"
            SELECT
                id,
                name,
                COALESCE(color, '') as color,
                COALESCE(icon, '') as icon,
                parent_id,
                category_type::text as category_type,
                created_at,
                deleted,
                deleted_at
            FROM category
            WHERE id = $1
                AND deleted = false
            "#,
            &[id],
        )
        .await?;

    if let Some(row) = rows.first() {
        Ok(Some(map_row_to_category(row)))
    } else {
        Ok(None)
    }
}

pub async fn list_categories(client: &Client) -> Result<Vec<Category>, AppError> {
    let rows = client
        .query(
            r#"
            SELECT
                id,
                name,
                COALESCE(color, '') as color,
                COALESCE(icon, '') as icon,
                parent_id,
                category_type::text as category_type,
                created_at,
                deleted,
                deleted_at
            FROM category
            WHERE deleted = false
            ORDER BY created_at DESC
            "#,
            &[],
        )
        .await?;

    Ok(rows.into_iter().map(|r| map_row_to_category(&r)).collect())
}

pub async fn delete_category(client: &Client, id: &Uuid) -> Result<(), AppError> {
    client
        .execute(
            r#"
            UPDATE category
            SET deleted = true,
                deleted_at = now()
            WHERE id = $1
            "#,
            &[id],
        )
        .await?;
    Ok(())
}

pub async fn list_categories_not_in_budget(client: &Client) -> Result<Vec<Category>, AppError> {
    let rows = client
        .query(
            r#"
            SELECT
                c.id,
                c.name,
                COALESCE(c.color, '') as color,
                COALESCE(c.icon, '') as icon,
                c.parent_id,
                c.category_type::text as category_type,
                c.created_at,
                c.deleted,
                c.deleted_at
            FROM category c
            LEFT JOIN budget_category bc
                ON c.id = bc.category_id
            WHERE c.deleted = false
              AND (bc.id is null OR bc.deleted = true)
            ORDER BY created_at DESC
            "#,
            &[],
        )
        .await?;

    Ok(rows.into_iter().map(|r| map_row_to_category(&r)).collect())
}

fn map_row_to_category(row: &Row) -> Category {
    Category {
        id: row.get("id"),
        name: row.get("name"),
        color: row.get("color"),
        icon: row.get("icon"),
        parent_id: row.get("parent_id"),
        category_type: category_type_from_db(row.get::<_, &str>("category_type")),
        created_at: row.get("created_at"),
        deleted: row.get("deleted"),
        deleted_at: row.get("deleted_at"),
    }
}

pub fn category_type_from_db<T: AsRef<str>>(value: T) -> CategoryType {
    match value.as_ref() {
        "Incoming" => CategoryType::Incoming,
        "Outgoing" => CategoryType::Outgoing,
        "Transfer" => CategoryType::Transfer,
        other => panic!("Unknown category type: {}", other),
    }
}

trait CategoryRequestDbExt {
    fn category_type_to_db(&self) -> String;
}

impl CategoryRequestDbExt for CategoryRequest {
    fn category_type_to_db(&self) -> String {
        match self.category_type {
            CategoryType::Incoming => "Incoming".to_string(),
            CategoryType::Outgoing => "Outgoing".to_string(),
            CategoryType::Transfer => "Transfer".to_string(),
        }
    }
}
