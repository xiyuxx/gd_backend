pub mod token;
mod fairing;
mod db_service;
pub mod validate;
pub mod route;


use std::io::Cursor;
use rocket::{ FromForm, Request, Response};
use rocket::http::{ContentType};
use rocket::response::Responder;
use serde::{Deserialize, Serialize};
use crate::auth::token::set_token;
use crate::types::{RtData, SignData};

#[derive(Serialize,Deserialize,Clone,Eq, PartialEq,Debug)]
pub struct UserToken{
    pub id: String,
    pub exp: u64,
}

#[derive(Debug)]
pub struct AuthCheck {
    pub is_valid_token: bool
}

#[derive(Debug,Serialize,Deserialize,Clone,Eq, PartialEq,FromForm)]
pub struct AddUser{ //直接添加成员
    #[field(name="name")]
    name : String,
    #[field(name="pwd")]
    pwd: String,
    #[field(name="organization")]
    organization:String, // 组织的uuid
    #[field(name="phone")]
    phone:Option<String>,
    #[field(name="email")]
    email:Option<String>,
    #[field(name="workId")]
    work_id:Option<String>,
    #[field(name="gender")]
    gender:Option<String>,
}

impl Into<(String,String,String,String)> for AddUser {
    fn into(self) -> (String, String, String, String) {
        (self.name,self.pwd,
         self.email.unwrap_or("".to_string()),self.phone.unwrap_or("".to_string()))
    }
}

impl Into<(String,String,String,
           Option<String>,Option<String>,Option<String>,Option<String>)> for AddUser{
    fn into(self) -> (String, String, String,
                      Option<String>, Option<String>, Option<String>, Option<String>) {
        (self.name,self.pwd,self.organization,self.phone,self.email,self.gender,self.work_id)
    }
}
#[derive(Debug,Serialize,Deserialize,Clone,Eq, PartialEq,FromForm)]
pub struct RegisterUser{
    #[field(name="phone")]
    phone:String,
    #[field(name="name")]
    name:String,
    #[field(name="pwd")]
    pwd:String,
    #[field(name="organization")]
    organization:String
}
impl Into<(String,String,String,String)> for RegisterUser{
    fn into(self) ->(String,String,String,String) {
        (self.name,self.pwd,self.phone,self.organization)
    }
}
#[derive(Debug,Serialize,Deserialize,Clone,Eq, PartialEq)]
pub enum MoreUser{
    Add(AddUser),
    Create(RegisterUser)
}

#[derive(Serialize,Deserialize,Debug,Clone,Eq, PartialEq)]
pub enum RegisterResult {
    Exist(String),
    Success(SignData)
}

impl<'r> Responder<'r,'static> for RtData<RegisterResult> {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let mut res = Response::build();
        res.header(ContentType::JSON);
        request.local_cache(|| AuthCheck {
            is_valid_token:true,
        });
        if let RegisterResult::Success(sign_data) = &self.data{
            let (token_field,token) = set_token(request,sign_data.id.as_str());
            res.raw_header(token_field,token);
        }
        let data = self.to_string();
        res.sized_body(data.len(),Cursor::new(data)).ok()
    }
}

#[derive(Debug,Serialize,Deserialize,Clone,Eq, PartialEq,FromForm)]
pub struct LoginData{
    #[field(name="login_key")]
    login_key:String,
    #[field(name="pwd")]
    pwd:String,
}





