use crate::models::account::{Account, AccountResponse};
use crate::models::category::{Category, CategoryResponse};
use crate::models::validate_uuid_v4;
use crate::models::vendor::{Vendor, VendorResponse};
use chrono::NaiveDate;
use rocket::serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Debug, Clone, Default)]
pub struct Transaction {
    pub id: Uuid,
    pub amount: i32,
    pub description: String,
    pub occurred_at: NaiveDate,
    pub category: Category,
    pub from_account: Account,
    pub to_account: Option<Account>,
    pub vendor: Option<Vendor>,
}

#[derive(Deserialize, Debug, Validate)]
pub struct TransactionRequest {
    #[validate(range(min = 0))]
    pub amount: i32,
    #[validate(length(min = 3))]
    pub description: String,
    pub occurred_at: NaiveDate,
    #[validate(length(equal = 36), custom(function = "validate_uuid_v4"))]
    pub category_id: String,
    #[validate(length(equal = 36), custom(function = "validate_uuid_v4"))]
    pub from_account_id: String,
    #[validate(length(equal = 36), custom(function = "validate_uuid_v4"))]
    pub to_account_id: Option<String>,
    #[validate(length(equal = 36), custom(function = "validate_uuid_v4"))]
    pub vendor_id: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct TransactionResponse {
    pub id: Uuid,
    pub amount: i32,
    pub description: String,
    pub occurred_at: NaiveDate,
    pub category: CategoryResponse,
    pub from_account: AccountResponse,
    pub to_account: Option<AccountResponse>,
    pub vendor: Option<VendorResponse>,
}

impl From<&Transaction> for TransactionResponse {
    fn from(transaction: &Transaction) -> Self {
        Self {
            id: transaction.id,
            amount: transaction.amount,
            description: transaction.description.clone(),
            occurred_at: transaction.occurred_at,
            category: CategoryResponse::from(&transaction.category),
            from_account: AccountResponse::from(&transaction.from_account),
            to_account: transaction.to_account.as_ref().map(AccountResponse::from),
            vendor: transaction.vendor.as_ref().map(VendorResponse::from),
        }
    }
}
