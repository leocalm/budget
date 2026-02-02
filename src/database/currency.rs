use crate::database::postgres_repository::PostgresRepository;
use crate::error::app_error::AppError;
use crate::models::currency::{Currency, CurrencyRequest};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait CurrencyRepository {
    async fn get_currency_by_code(&self, currency_code: &str) -> Result<Option<Currency>, AppError>;
    async fn get_currencies(&self, name: &str) -> Result<Vec<Currency>, AppError>;
    async fn create_currency(&self, currency: &CurrencyRequest) -> Result<Currency, AppError>;
    async fn delete_currency(&self, currency_id: &Uuid) -> Result<(), AppError>;
    async fn update_currency(&self, id: &Uuid, request: &CurrencyRequest) -> Result<Currency, AppError>;
}

#[async_trait::async_trait]
impl CurrencyRepository for PostgresRepository {
    async fn get_currency_by_code(&self, currency_code: &str) -> Result<Option<Currency>, AppError> {
        Ok(sqlx::query_as!(
            Currency,
            r#"
              SELECT id, name, symbol, currency, decimal_places, created_at
              FROM currency
              WHERE currency = $1
            "#,
            currency_code
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    async fn get_currencies(&self, name: &str) -> Result<Vec<Currency>, AppError> {
        let pattern = format!("%{}%", name);

        Ok(sqlx::query_as!(
            Currency,
            r#"
        SELECT id, name, symbol, currency, decimal_places, created_at
        FROM currency
        WHERE lower(name) LIKE lower($1)
        "#,
            pattern
        )
        .fetch_all(&self.pool)
        .await?)
    }

    async fn create_currency(&self, currency: &CurrencyRequest) -> Result<Currency, AppError> {
        Ok(sqlx::query_as!(
            Currency,
            r#"
        INSERT INTO currency (name, symbol, currency, decimal_places)
        VALUES ($1, $2, $3, $4)
        RETURNING
            id,
            name,
            symbol,
            currency,
            decimal_places,
            created_at
        "#,
            currency.name,
            currency.symbol,
            currency.currency,
            currency.decimal_places
        )
        .fetch_one(&self.pool)
        .await?)
    }

    async fn delete_currency(&self, currency_id: &Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#"
        DELETE FROM currency
        WHERE id = $1
        "#,
            currency_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_currency(&self, id: &Uuid, request: &CurrencyRequest) -> Result<Currency, AppError> {
        Ok(sqlx::query_as!(
            Currency,
            r#"
            UPDATE currency
            SET name = $1, symbol = $2, currency = $3, decimal_places = $4
            WHERE id = $5
            RETURNING id, name, symbol, currency, decimal_places, created_at
            "#,
            request.name,
            request.symbol,
            request.currency,
            request.decimal_places,
            id,
        )
        .fetch_one(&self.pool)
        .await?)
    }
}
