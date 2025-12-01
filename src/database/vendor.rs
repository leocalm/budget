use crate::error::app_error::AppError;
use crate::models::vendor::{Vendor, VendorRequest};
use deadpool_postgres::Client;
use tokio_postgres::Row;
use uuid::Uuid;

pub async fn create_vendor(client: &Client, request: &VendorRequest) -> Result<Vendor, AppError> {
    let rows = client
        .query(
            r#"
            INSERT INTO vendor (name)
            VALUES ($1)
            RETURNING id, name, created_at, deleted, deleted_at
            "#,
            &[&request.name],
        )
        .await?;

    if let Some(row) = rows.first() {
        Ok(map_row_to_vendor(row))
    } else {
        Err(AppError::Db("Error mapping created vendor".to_string()))
    }
}

pub async fn get_vendor_by_id(client: &Client, id: &Uuid) -> Result<Option<Vendor>, AppError> {
    let rows = client
        .query(
            r#"
            SELECT id, name, created_at, deleted, deleted_at
            FROM vendor
            WHERE id = $1
                AND deleted is false
            "#,
            &[id],
        )
        .await?;

    if let Some(row) = rows.first() {
        Ok(Some(map_row_to_vendor(row)))
    } else {
        Ok(None)
    }
}

pub async fn list_vendors(client: &Client) -> Result<Vec<Vendor>, AppError> {
    let rows = client
        .query(
            r#"
            SELECT id, name, created_at, deleted, deleted_at
            FROM vendor
            WHERE deleted = false
            ORDER BY created_at DESC
            "#,
            &[],
        )
        .await?;

    Ok(rows.into_iter().map(|r| map_row_to_vendor(&r)).collect())
}

pub async fn delete_vendor(client: &Client, id: &Uuid) -> Result<(), AppError> {
    client
        .execute(
            r#"
            UPDATE vendor
            SET deleted = true,
                deleted_at = now()
            WHERE id = $1
            "#,
            &[id],
        )
        .await?;
    Ok(())
}

fn map_row_to_vendor(row: &Row) -> Vendor {
    Vendor {
        id: row.get("id"),
        name: row.get("name"),
        created_at: row.get("created_at"),
        deleted: row.get("deleted"),
        deleted_at: row.get("deleted_at"),
    }
}
