use crate::database::user::{create_user, get_user_by_email, verify_password};
use crate::db::get_client;
use crate::error::app_error::AppError;
use crate::models::user::{CreateUserRequest, LoginRequest, User};
use deadpool_postgres::Pool;
use rocket::State;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::serde::json::Json;

#[rocket::post("/users", data = "<payload>")]
pub async fn post_user(
    pool: &State<Pool>,
    payload: Json<CreateUserRequest>,
) -> Result<Json<User>, AppError> {
    let client = get_client(pool).await?;
    let user = get_user_by_email(&client, &payload.email).await?;
    if user.is_some() {
        return Err(AppError::UserAlreadyExists(payload.email.clone()));
    }

    let user = create_user(&client, &payload.name, &payload.email, &payload.password).await?;

    if let Some(user) = user {
        Ok(Json(user))
    } else {
        Err(AppError::Db("User does not exist".to_string()))
    }
}

#[rocket::post("/users/login", data = "<payload>")]
pub async fn post_user_login(
    pool: &State<Pool>,
    cookies: &CookieJar<'_>,
    payload: Json<LoginRequest>,
) -> Result<(), AppError> {
    let client = get_client(pool).await?;
    if let Some(user) = get_user_by_email(&client, &payload.email).await? {
        verify_password(&user, &payload.password).await?;
        let value = format!("{}:{}", user.id, user.email);
        cookies.add_private(Cookie::build(("user", value)).path("/").build());
    }

    Ok(())
}

#[rocket::post("/users/logout")]
pub fn post_user_logout(cookies: &CookieJar<'_>) -> Status {
    cookies.remove_private(Cookie::build("user").build());
    Status::Ok
}
