pub mod token;
mod fairing;
mod db_service;
pub mod validate;
pub mod route;


use uuid::Uuid;

use std::io::Cursor;
use rocket::{ FromForm, Request, Response};
use rocket::http::{ContentType};
use rocket::response::Responder;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
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

#[derive(Debug,Serialize,Deserialize,Clone,Eq,PartialEq,FromForm)]
pub struct User{
    #[field(name="id")]
    id:String,
    #[field(name="name")]
    name:Option<String>,
    #[field(name="pwd")]
    pwd:Option<String>,
    #[field(name="phone")]
    phone:Option<String>,
    #[field(name="gender")]
    gender:Option<String>,
    #[field(name="email")]
    email:Option<String>,
    #[field(name="avatar")]
    avatar:Option<String>,
    #[field(name="background")]
    background:Option<String>,
    #[field(name="work_id")]
    work_id:Option<String>,
}

impl Into<(Option<String>,Option<String>,Option<String>,Option<String>,
           Option<String>,Option<String>,Option<String>,Option<String>)> for User{
    fn into(self) -> (Option<String>,Option<String>, Option<String>, Option<String>,
                      Option<String>, Option<String>, Option<String>, Option<String>) {
        (self.name,self.pwd,self.phone,self.gender,
         self.email,self.avatar,self.background,self.work_id)
    }
}

#[derive(Debug,Serialize,Deserialize,Clone,Eq,PartialEq,FromRow)]
pub struct UserGetter {
    #[sqlx(try_from="Uuid")]
    pub id:String,
    pub name:String,
    #[sqlx(default)]
    pub phone:Option<String>,
    #[sqlx(default)]
    pub gender:Option<String>,
    #[sqlx(default)]
    pub email:Option<String>,
    #[sqlx(default)]
    pub work_id:Option<String>,
    #[sqlx(default)]
    pub avatar:Option<String>,
    #[sqlx(default)]
    pub background:Option<String>,
    #[sqlx(default)]
    pub create_time:Option<String>,
    #[sqlx(default)]
    pub role:Option<i16>,
    // organization_id no need to be passed
}

#[derive(Debug,Serialize,Deserialize,Clone,Eq,PartialEq)]
pub struct UserCollector{
    pub collector:Vec<UserGetter>
}

impl<'r> Responder<'r,'static> for RtData<UserCollector> {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let data = self.to_string();
        request.local_cache(||AuthCheck{
            is_valid_token:true
        });
        Response::build()
            .header(ContentType::JSON)
            .sized_body(data.len(),Cursor::new(data)).ok()
    }
}




