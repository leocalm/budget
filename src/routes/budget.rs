use crate::auth::CurrentUser;
use crate::database::budget::{create_budget, get_all_budgets};
use crate::db::get_client;
use crate::error::app_error::AppError;
use crate::models::budget::{Budget, CreateBudgetRequest};
use crate::models::health::HealthResponse;
use deadpool_postgres::Pool;
use rocket::{State, serde::json::Json};

#[rocket::get("/health")]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

#[rocket::post("/budgets", data = "<payload>")]
pub async fn post_budget(
    pool: &State<Pool>,
    _current_user: CurrentUser,
    payload: Json<CreateBudgetRequest>,
) -> Result<Json<Budget>, AppError> {
    let client = get_client(pool).await?;
    let budget = create_budget(&client, &payload.name).await?;

    if let Some(budget) = budget {
        Ok(Json(budget))
    } else {
        Err(AppError::Db("Error creating budget".to_string()))
    }
}

#[rocket::get("/budgets")]
pub async fn list_budgets(
    pool: &State<Pool>,
    _current_user: CurrentUser,
) -> Result<Json<Vec<Budget>>, AppError> {
    let client = get_client(pool).await?;
    let budgets = get_all_budgets(&client).await?;

    Ok(Json(budgets))
}
