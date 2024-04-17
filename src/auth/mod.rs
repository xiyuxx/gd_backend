mod token;
mod fairing;

use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize,Clone,Eq, PartialEq,Debug)]
pub struct UserToken{
    pub id: String,
    pub exp: u64,
}

#[derive(Debug)]
pub struct AuthCheck {
    pub is_valid_token: bool
}


