use rocket::Config;
use rocket::fairing::AdHoc;
use rocket::figment::Figment;
use rocket::figment::providers::{Format, Serialized, Toml};
use serde::{Deserialize, Serialize};

#[derive(Debug,Eq, PartialEq,Serialize,Deserialize,Clone)]
pub struct TokenInfo {
    pub token_field:String,
    pub token_key: String,
    pub exp:u64
}

impl Default for TokenInfo {
    fn default() -> Self {
        Self {
            token_field: String::from("_token"),
            token_key: String::from("xiyuxx"),
            exp: 24 * 60 * 60 * 7, // one week
        }
    }
}

pub fn get_custom_figment() -> Figment {
    Figment::from(Config::default())
        .merge(Toml::file("Rocket.toml").nested())
        .merge(Serialized::defaults(TokenInfo::default()))
}

pub fn init_my_config() -> AdHoc {
    AdHoc::config::<TokenInfo>()
}