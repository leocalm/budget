use crate::auth::CurrentUser;
use crate::database;
use crate::db::get_client;
use crate::error::app_error::AppError;
use crate::models::budget_category::{BudgetCategoryRequest, BudgetCategoryResponse};
use deadpool_postgres::Pool;
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use uuid::Uuid;

#[rocket::post("/budget-categories", data = "<payload>")]
pub async fn create_budget_category(
    pool: &State<Pool>,
    _current_user: CurrentUser,
    payload: Json<BudgetCategoryRequest>,
) -> Result<(Status, Json<BudgetCategoryResponse>), AppError> {
    let client = get_client(pool).await?;
    let bc = database::budget_category::create_budget_category(&client, &payload).await?;
    Ok((Status::Created, Json(BudgetCategoryResponse::from(&bc))))
}

#[rocket::get("/budget-categories")]
pub async fn list_all_budget_categories(
    pool: &State<Pool>,
    _current_user: CurrentUser,
) -> Result<Json<Vec<BudgetCategoryResponse>>, AppError> {
    let client = get_client(pool).await?;
    let list = database::budget_category::list_budget_categories(&client).await?;
    Ok(Json(
        list.iter().map(BudgetCategoryResponse::from).collect(),
    ))
}

#[rocket::get("/budget-categories/<id>")]
pub async fn get_budget_category(
    pool: &State<Pool>,
    _current_user: CurrentUser,
    id: &str,
) -> Result<Json<BudgetCategoryResponse>, AppError> {
    let client = get_client(pool).await?;
    let uuid = Uuid::parse_str(id)?;
    if let Some(bc) = database::budget_category::get_budget_category_by_id(&client, &uuid).await? {
        Ok(Json(BudgetCategoryResponse::from(&bc)))
    } else {
        Err(AppError::NotFound("Budget category not found".to_string()))
    }
}

#[rocket::delete("/budget-categories/<id>")]
pub async fn delete_budget_category(
    pool: &State<Pool>,
    _current_user: CurrentUser,
    id: &str,
) -> Result<Status, AppError> {
    let client = get_client(pool).await?;
    let uuid = Uuid::parse_str(id)?;
    database::budget_category::delete_budget_category(&client, &uuid).await?;
    Ok(Status::Ok)
}
