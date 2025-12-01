use crate::auth::CurrentUser;
use crate::database;
use crate::db::get_client;
use crate::error::app_error::AppError;
use crate::models::budget::{BudgetRequest, BudgetResponse};
use deadpool_postgres::Pool;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use uuid::Uuid;

#[rocket::post("/budgets", data = "<payload>")]
pub async fn create_budget(
    pool: &State<Pool>,
    _current_user: CurrentUser,
    payload: Json<BudgetRequest>,
) -> Result<(Status, Json<BudgetResponse>), AppError> {
    let client = get_client(pool).await?;
    let budget = database::budget::create_budget(&client, &payload).await?;
    Ok((Status::Created, Json(BudgetResponse::from(&budget))))
}

#[rocket::get("/budgets")]
pub async fn list_all_budgets(
    pool: &State<Pool>,
    _current_user: CurrentUser,
) -> Result<Json<Vec<BudgetResponse>>, AppError> {
    let client = get_client(pool).await?;
    let budgets = database::budget::list_budgets(&client).await?;
    Ok(Json(budgets.iter().map(BudgetResponse::from).collect()))
}

#[rocket::get("/budgets/<id>")]
pub async fn get_budget(
    pool: &State<Pool>,
    _current_user: CurrentUser,
    id: &str,
) -> Result<Json<BudgetResponse>, AppError> {
    let client = get_client(pool).await?;
    let uuid = Uuid::parse_str(id)?;
    if let Some(budget) = database::budget::get_budget_by_id(&client, &uuid).await? {
        Ok(Json(BudgetResponse::from(&budget)))
    } else {
        Err(AppError::NotFound("Budget not found".to_string()))
    }
}

#[rocket::put("/budgets/<id>", data = "<payload>")]
pub async fn put_budget(
    pool: &State<Pool>,
    _current_user: CurrentUser,
    id: &str,
    payload: Json<BudgetRequest>,
) -> Result<(Status, Json<BudgetResponse>), AppError> {
    let client = get_client(pool).await?;
    let budget = database::budget::update_budget(&client, &Uuid::parse_str(id)?, &payload).await?;
    Ok((Status::Ok, Json(BudgetResponse::from(&budget))))
}

#[rocket::delete("/budgets/<id>")]
pub async fn delete_budget(
    pool: &State<Pool>,
    _current_user: CurrentUser,
    id: &str,
) -> Result<Status, AppError> {
    let client = get_client(pool).await?;
    let uuid = Uuid::parse_str(id)?;
    database::budget::delete_budget(&client, &uuid).await?;
    Ok(Status::Ok)
}
