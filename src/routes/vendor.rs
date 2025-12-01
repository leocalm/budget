use crate::auth::CurrentUser;
use crate::database;
use crate::db::get_client;
use crate::error::app_error::AppError;
use crate::models::vendor::{VendorRequest, VendorResponse};
use deadpool_postgres::Pool;
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use uuid::Uuid;

#[rocket::post("/vendors", data = "<payload>")]
pub async fn create_vendor(
    pool: &State<Pool>,
    _current_user: CurrentUser,
    payload: Json<VendorRequest>,
) -> Result<(Status, Json<VendorResponse>), AppError> {
    let client = get_client(pool).await?;
    let vendor = database::vendor::create_vendor(&client, &payload).await?;
    Ok((Status::Created, Json(VendorResponse::from(&vendor))))
}

#[rocket::get("/vendors")]
pub async fn list_all_vendors(
    pool: &State<Pool>,
    _current_user: CurrentUser,
) -> Result<Json<Vec<VendorResponse>>, AppError> {
    let client = get_client(pool).await?;
    let vendors = database::vendor::list_vendors(&client).await?;
    Ok(Json(vendors.iter().map(VendorResponse::from).collect()))
}

#[rocket::get("/vendors/<id>")]
pub async fn get_vendor(
    pool: &State<Pool>,
    _current_user: CurrentUser,
    id: &str,
) -> Result<Json<VendorResponse>, AppError> {
    let client = get_client(pool).await?;
    let uuid = Uuid::parse_str(id)?;
    if let Some(vendor) = database::vendor::get_vendor_by_id(&client, &uuid).await? {
        Ok(Json(VendorResponse::from(&vendor)))
    } else {
        Err(AppError::NotFound("Vendor not found".to_string()))
    }
}

#[rocket::delete("/vendors/<id>")]
pub async fn delete_vendor(
    pool: &State<Pool>,
    _current_user: CurrentUser,
    id: &str,
) -> Result<Status, AppError> {
    let client = get_client(pool).await?;
    let uuid = Uuid::parse_str(id)?;
    database::vendor::delete_vendor(&client, &uuid).await?;
    Ok(Status::Ok)
}
