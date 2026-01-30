use std::str::FromStr;
use uuid::Uuid;
use validator::ValidationError;

pub mod account;
pub mod budget;
pub mod budget_category;
pub mod budget_period;
pub mod category;
pub mod currency;
pub mod dashboard;
pub mod transaction;
pub mod user;
pub mod vendor;

fn validate_uuid_v4(id: &str) -> Result<(), ValidationError> {
    if Uuid::from_str(id).is_ok() {
        Ok(())
    } else {
        Err(ValidationError::new("Valid Uuid"))
    }
}
