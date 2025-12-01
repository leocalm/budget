use crate::database::category::category_type_from_db;
use crate::error::app_error::AppError;
use crate::models::account::Account;
use crate::models::category::Category;
use crate::models::currency::Currency;
use crate::models::transaction::{Transaction, TransactionRequest, TransactionType};
use crate::models::vendor::Vendor;
use deadpool_postgres::Client;
use tokio_postgres::Row;
use uuid::Uuid;

pub async fn create_transaction(
    client: &Client,
    request: &TransactionRequest,
) -> Result<Transaction, AppError> {
    let rows = client
        .query(
            r#"
            INSERT INTO transaction (
                amount,
                description,
                occurred_at,
                transaction_type,
                category_id,
                from_account_id,
                to_account_id,
                vendor_id
            )
            VALUES ($1, $2, $3, $4::text::transaction_type, $5, $6, $7, $8)
            RETURNING id
            "#,
            &[
                &request.amount,
                &request.description,
                &request.occurred_at,
                &request.transaction_type_to_db(),
                &request.category_id,
                &request.from_account_id,
                &request.to_account_id,
                &request.vendor_id,
            ],
        )
        .await?;

    if let Some(row) = rows.first() {
        let id: Uuid = row.get("id");

        if let Some(new_transaction) = get_transaction_by_id(&client, &id).await? {
            Ok(new_transaction)
        } else {
            Err(AppError::Db(
                "Error mapping created transaction".to_string(),
            ))
        }
    } else {
        Err(AppError::Db(
            "Error mapping created transaction".to_string(),
        ))
    }
}

pub async fn get_transaction_by_id(
    client: &Client,
    id: &Uuid,
) -> Result<Option<Transaction>, AppError> {
    let rows = client
        .query(
            r#"
            SELECT
                t.id,
                t.amount,
                t.description,
                t.occurred_at,
                t.transaction_type::text as transaction_type,
                t.deleted,
                t.deleted_at,
                c.id as category_id,
                c.name as category_name,
                COALESCE(c.color, '') as category_color,
                COALESCE(c.icon, '') as category_icon,
                c.parent_id as category_parent_id,
                c.category_type::text as category_category_type,
                c.created_at as category_created_at,
                c.deleted as category_deleted,
                c.deleted_at as category_deleted_at,
                fa.id as from_account_id,
                fa.name as from_account_name,
                fa.color as from_account_color,
                fa.icon as from_account_icon,
                fa.account_type::text as from_account_account_type,
                fa.balance as from_account_balance,
                fa.created_at as from_account_created_at,
                fa.deleted as from_account_deleted,
                fa.deleted_at as from_account_deleted_at,
                cfa.id as from_account_currency_id,
                cfa.name as from_account_currency_name,
                cfa.symbol as from_account_currency_symbol,
                cfa.currency as from_account_currency_code,
                cfa.decimal_places as from_account_currency_decimal_places,
                cfa.created_at as from_account_currency_created_at,
                cfa.deleted as from_account_currency_deleted,
                cfa.deleted_at as from_account_currency_deleted_at,
                ta.id as to_account_id,
                ta.name as to_account_name,
                ta.color as to_account_color,
                ta.icon as to_account_icon,
                ta.account_type::text as to_account_account_type,
                ta.balance as to_account_balance,
                ta.created_at as to_account_created_at,
                ta.deleted as to_account_deleted,
                ta.deleted_at as to_account_deleted_at,
                cta.id as to_account_currency_id,
                cta.name as to_account_currency_name,
                cta.symbol as to_account_currency_symbol,
                cta.currency as to_account_currency_code,
                cta.decimal_places as to_account_currency_decimal_places,
                cta.created_at as to_account_currency_created_at,
                cta.deleted as to_account_currency_deleted,
                cta.deleted_at as to_account_currency_deleted_at,
                v.id as vendor_id,
                v.name as vendor_name,
                v.created_at as vendor_created_at,
                v.deleted as vendor_deleted,
                v.deleted_at as vendor_deleted_at
            FROM transaction t
            JOIN category c ON t.category_id = c.id
            JOIN account fa ON t.from_account_id = fa.id
            JOIN currency cfa ON fa.currency_id = cfa.id
            LEFT JOIN account ta ON t.to_account_id = ta.id
            LEFT JOIN currency cta ON ta.currency_id = cta.id
            JOIN vendor v ON t.vendor_id = v.id
            WHERE t.id = $1
                AND t.deleted = false
            "#,
            &[id],
        )
        .await?;

    if let Some(row) = rows.first() {
        Ok(Some(map_row_to_transaction(row)))
    } else {
        Ok(None)
    }
}

pub async fn list_transactions(client: &Client) -> Result<Vec<Transaction>, AppError> {
    let rows = client
        .query(
            r#"
            SELECT
                t.id,
                t.amount,
                t.description,
                t.occurred_at,
                t.transaction_type::text as transaction_type,
                t.deleted,
                t.deleted_at,
                c.id as category_id,
                c.name as category_name,
                COALESCE(c.color, '') as category_color,
                COALESCE(c.icon, '') as category_icon,
                c.parent_id as category_parent_id,
                c.category_type::text as category_category_type,
                c.created_at as category_created_at,
                c.deleted as category_deleted,
                c.deleted_at as category_deleted_at,
                fa.id as from_account_id,
                fa.name as from_account_name,
                fa.color as from_account_color,
                fa.icon as from_account_icon,
                fa.account_type::text as from_account_account_type,
                fa.balance as from_account_balance,
                fa.created_at as from_account_created_at,
                fa.deleted as from_account_deleted,
                fa.deleted_at as from_account_deleted_at,
                cfa.id as from_account_currency_id,
                cfa.name as from_account_currency_name,
                cfa.symbol as from_account_currency_symbol,
                cfa.currency as from_account_currency_code,
                cfa.decimal_places as from_account_currency_decimal_places,
                cfa.created_at as from_account_currency_created_at,
                cfa.deleted as from_account_currency_deleted,
                cfa.deleted_at as from_account_currency_deleted_at,
                ta.id as to_account_id,
                ta.name as to_account_name,
                ta.color as to_account_color,
                ta.icon as to_account_icon,
                ta.account_type::text as to_account_account_type,
                ta.balance as to_account_balance,
                ta.created_at as to_account_created_at,
                ta.deleted as to_account_deleted,
                ta.deleted_at as to_account_deleted_at,
                cta.id as to_account_currency_id,
                cta.name as to_account_currency_name,
                cta.symbol as to_account_currency_symbol,
                cta.currency as to_account_currency_code,
                cta.decimal_places as to_account_currency_decimal_places,
                cta.created_at as to_account_currency_created_at,
                cta.deleted as to_account_currency_deleted,
                cta.deleted_at as to_account_currency_deleted_at,
                v.id as vendor_id,
                v.name as vendor_name,
                v.created_at as vendor_created_at,
                v.deleted as vendor_deleted,
                v.deleted_at as vendor_deleted_at
            FROM transaction t
            JOIN category c ON t.category_id = c.id
            JOIN account fa ON t.from_account_id = fa.id
            JOIN currency cfa ON fa.currency_id = cfa.id
            LEFT JOIN account ta ON t.to_account_id = ta.id
            LEFT JOIN currency cta ON ta.currency_id = cta.id
            JOIN vendor v ON t.vendor_id = v.id
            WHERE t.deleted = false
            ORDER BY occurred_at DESC
            "#,
            &[],
        )
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| map_row_to_transaction(&row))
        .collect())
}

pub async fn delete_transaction(client: &Client, id: &Uuid) -> Result<(), AppError> {
    client
        .execute(
            r#"
            UPDATE transaction
            SET deleted = true,
                deleted_at = now()
            WHERE id = $1
            "#,
            &[id],
        )
        .await?;

    Ok(())
}

fn map_row_to_transaction(row: &Row) -> Transaction {
    let a: Option<Uuid> = row.get("to_account_id");
    let to_account = if a.is_some() {
        Some(Account {
            id: row.get("to_account_id"),
            name: row.get("to_account_name"),
            color: row.get("to_account_color"),
            icon: row.get("to_account_icon"),
            account_type: crate::database::account::account_type_from_db(
                row.get::<_, &str>("to_account_account_type"),
            ),
            currency: Currency {
                id: row.get("to_account_currency_id"),
                name: row.get("to_account_currency_name"),
                symbol: row.get("to_account_currency_symbol"),
                currency: row.get("to_account_currency_code"),
                decimal_places: row.get("to_account_currency_decimal_places"),
                created_at: row.get("to_account_currency_created_at"),
                deleted: row.get("to_account_currency_deleted"),
                deleted_at: row.get("to_account_currency_deleted_at"),
            },
            balance: row.get("to_account_balance"),
            created_at: row.get("to_account_created_at"),
            deleted: row.get("to_account_deleted"),
            deleted_at: row.get("to_account_deleted_at"),
        })
    } else {
        None
    };

    Transaction {
        id: row.get("id"),
        amount: row.get("amount"),
        description: row.get("description"),
        occurred_at: row.get("occurred_at"),
        transaction_type: transaction_type_from_db(row.get::<_, &str>("transaction_type")),
        category: Category {
            id: row.get("category_id"),
            name: row.get("category_name"),
            color: row.get("category_color"),
            icon: row.get("category_icon"),
            parent_id: row.get("category_parent_id"),
            category_type: category_type_from_db(row.get::<_, &str>("category_category_type")),
            created_at: row.get("category_created_at"),
            deleted: row.get("category_deleted"),
            deleted_at: row.get("category_deleted_at"),
        },
        from_account: Account {
            id: row.get("from_account_id"),
            name: row.get("from_account_name"),
            color: row.get("from_account_color"),
            icon: row.get("from_account_icon"),
            account_type: crate::database::account::account_type_from_db(
                row.get::<_, &str>("from_account_account_type"),
            ),
            currency: Currency {
                id: row.get("from_account_currency_id"),
                name: row.get("from_account_currency_name"),
                symbol: row.get("from_account_currency_symbol"),
                currency: row.get("from_account_currency_code"),
                decimal_places: row.get("from_account_currency_decimal_places"),
                created_at: row.get("from_account_currency_created_at"),
                deleted: row.get("from_account_currency_deleted"),
                deleted_at: row.get("from_account_currency_deleted_at"),
            },
            balance: row.get("from_account_balance"),
            created_at: row.get("from_account_created_at"),
            deleted: row.get("from_account_deleted"),
            deleted_at: row.get("from_account_deleted_at"),
        },
        to_account,
        vendor: Vendor {
            id: row.get("vendor_id"),
            name: row.get("vendor_name"),
            created_at: row.get("vendor_created_at"),
            deleted: row.get("vendor_deleted"),
            deleted_at: row.get("vendor_deleted_at"),
        },
        deleted: row.get("deleted"),
        deleted_at: row.get("deleted_at"),
    }
}

fn transaction_type_from_db<T: AsRef<str>>(value: T) -> TransactionType {
    match value.as_ref() {
        "Incoming" => TransactionType::Incoming,
        "Outgoing" => TransactionType::Outgoing,
        "Transfer" => TransactionType::Transfer,
        other => panic!("Unknown transaction type: {}", other),
    }
}

// Helper method for TransactionRequest to map to DB enum/text value
trait TransactionRequestDbExt {
    fn transaction_type_to_db(&self) -> String;
}

impl TransactionRequestDbExt for TransactionRequest {
    fn transaction_type_to_db(&self) -> String {
        match self.transaction_type {
            TransactionType::Incoming => "Incoming".to_string(),
            TransactionType::Outgoing => "Outgoing".to_string(),
            TransactionType::Transfer => "Transfer".to_string(),
        }
    }
}
