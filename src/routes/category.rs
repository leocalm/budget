use crate::auth::CurrentUser;
use crate::database;
use crate::db::get_client;
use crate::error::app_error::AppError;
use crate::models::category::{CategoryRequest, CategoryResponse};
use deadpool_postgres::Pool;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use uuid::Uuid;

#[rocket::post("/categories", data = "<payload>")]
pub async fn create_category(
    pool: &State<Pool>,
    _current_user: CurrentUser,
    payload: Json<CategoryRequest>,
) -> Result<(Status, Json<CategoryResponse>), AppError> {
    let client = get_client(pool).await?;
    let category = database::category::create_category(&client, &payload).await?;
    Ok((Status::Created, Json(CategoryResponse::from(&category))))
}

#[rocket::get("/categories")]
pub async fn list_all_categories(
    pool: &State<Pool>,
    _current_user: CurrentUser,
) -> Result<Json<Vec<CategoryResponse>>, AppError> {
    let client = get_client(pool).await?;
    let categories = database::category::list_categories(&client).await?;
    Ok(Json(
        categories.iter().map(CategoryResponse::from).collect(),
    ))
}

#[rocket::get("/categories/<id>")]
pub async fn get_category(
    pool: &State<Pool>,
    _current_user: CurrentUser,
    id: &str,
) -> Result<Json<CategoryResponse>, AppError> {
    let client = get_client(pool).await?;
    let uuid = Uuid::parse_str(id)?;
    if let Some(category) = database::category::get_category_by_id(&client, &uuid).await? {
        Ok(Json(CategoryResponse::from(&category)))
    } else {
        Err(AppError::NotFound("Category not found".to_string()))
    }
}

#[rocket::delete("/categories/<id>")]
pub async fn delete_category(
    pool: &State<Pool>,
    _current_user: CurrentUser,
    id: &str,
) -> Result<Status, AppError> {
    let client = get_client(pool).await?;
    let uuid = Uuid::parse_str(id)?;
    database::category::delete_category(&client, &uuid).await?;
    Ok(Status::Ok)
}

#[rocket::get("/categories/not-in-budget")]
pub async fn list_categories_not_in_budget(
    pool: &State<Pool>,
    _current_user: CurrentUser,
) -> Result<Json<Vec<CategoryResponse>>, AppError> {
    let client = get_client(pool).await?;
    let categories = database::category::list_categories_not_in_budget(&client).await?;
    Ok(Json(
        categories.iter().map(CategoryResponse::from).collect(),
    ))
}
