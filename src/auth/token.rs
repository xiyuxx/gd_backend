use jsonwebtoken::errors::Error;
use jsonwebtoken::{decode, DecodingKey, encode, EncodingKey, get_current_timestamp, Header, TokenData, Validation};
use rocket::Request;

use crate::auth::UserToken;
use crate::config::TokenInfo;

pub fn decode_token(token: &str, key:&str) -> Result<TokenData<UserToken>,Error> {
    decode(token, &DecodingKey::from_secret(key.as_ref()), &Validation::default())

}

pub fn encode_token(user_id:String,exp:u64,key:&str)-> String {
    let user_token = UserToken {
        id: user_id,
        exp,
    };
    encode::<UserToken>(&Header::default(),
                        &user_token,
                        &EncodingKey::from_secret(key.as_ref()))
        .expect("encode token error")
}

pub fn set_token<'r>(req:&'r Request<'_>,user_id:&str) -> (String,String) {
    let config = req.rocket().state::<TokenInfo>()
        .expect("get global state error when response in login");

    let token_field = config.token_field.as_str();
    let token_key = config.token_key.as_str();
    let exp = config.exp + get_current_timestamp();

    let token = encode_token(user_id.to_string(),exp,token_key);

    (token_field.to_string(),token)
}