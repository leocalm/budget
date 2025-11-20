use rocket::serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
pub struct Budget {
    pub id: Uuid,
    pub name: String,
}

#[derive(Deserialize)]
pub struct CreateBudgetRequest {
    pub name: String,
}
