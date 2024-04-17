
use jsonwebtoken::get_current_timestamp;
use gd_backend::auth::UserToken;

pub fn get_default_user_token() -> UserToken {
    UserToken {
        id: uuid::Uuid::new_v4().to_string(),
        exp: get_current_timestamp(),
    }
}