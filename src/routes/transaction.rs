use crate::auth::CurrentUser;
use crate::database;
use crate::db::get_client;
use crate::error::app_error::AppError;
use crate::models::transaction::{TransactionRequest, TransactionResponse};
use deadpool_postgres::Pool;
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use uuid::Uuid;

#[rocket::post("/transactions", data = "<payload>")]
pub async fn create_transaction(
    pool: &State<Pool>,
    _current_user: CurrentUser,
    payload: Json<TransactionRequest>,
) -> Result<(Status, Json<TransactionResponse>), AppError> {
    let client = get_client(pool).await?;
    let tx = database::transaction::create_transaction(&client, &payload).await?;
    Ok((Status::Created, Json(TransactionResponse::from(&tx))))
}

#[rocket::get("/transactions")]
pub async fn list_all_transactions(
    pool: &State<Pool>,
    _current_user: CurrentUser,
) -> Result<Json<Vec<TransactionResponse>>, AppError> {
    let client = get_client(pool).await?;
    let txs = database::transaction::list_transactions(&client).await?;
    Ok(Json(txs.iter().map(TransactionResponse::from).collect()))
}

#[rocket::get("/transactions/<id>")]
pub async fn get_transaction(
    pool: &State<Pool>,
    _current_user: CurrentUser,
    id: &str,
) -> Result<Json<TransactionResponse>, AppError> {
    let client = get_client(pool).await?;
    let uuid = Uuid::parse_str(id)?;
    if let Some(tx) = database::transaction::get_transaction_by_id(&client, &uuid).await? {
        Ok(Json(TransactionResponse::from(&tx)))
    } else {
        Err(AppError::NotFound("Transaction not found".to_string()))
    }
}

#[rocket::delete("/transactions/<id>")]
pub async fn delete_transaction(
    pool: &State<Pool>,
    _current_user: CurrentUser,
    id: &str,
) -> Result<Status, AppError> {
    let client = get_client(pool).await?;
    let uuid = Uuid::parse_str(id)?;
    database::transaction::delete_transaction(&client, &uuid).await?;
    Ok(Status::Ok)
}
