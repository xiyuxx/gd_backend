
use jsonwebtoken::get_current_timestamp;
use rocket::http::Status;

use crate::auth::UserToken;
use crate::auth::validate::ValidateData;

pub fn get_default_user_token() -> UserToken {
    UserToken {
        id: uuid::Uuid::new_v4().to_string(),
        exp: get_current_timestamp(),
    }
}

pub fn init_validate_instance() -> Result<ValidateData,Status> {
    let instance = ValidateData::new()?;
    Ok(instance)
}