use std::io::Cursor;
use chrono::NaiveDateTime;

use rocket::{FromForm, Request, Response};
use rocket::http::{ContentType, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::response::Responder;
use serde::{Deserialize, Serialize};
use crate::auth::AuthCheck;
use crate::auth::token::{decode_token, set_token};
use crate::config::TokenInfo;
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Serialize,Deserialize,Debug,Eq, PartialEq,Clone)]
pub struct RtData<T> {
    pub data: T,
    pub msg: String,
    pub success: bool,
    pub status: RtStatus
}

impl<T:Serialize> RtData<T>{
    pub fn to_string(mut self) -> String{
        serde_json::to_string(&mut self).unwrap()
    }
}

impl<'r> Responder<'r,'static> for RtData<DefaultData> {
    fn respond_to(self, _request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let data = self.to_string();

        Response::build()
            .header(ContentType::JSON)
            .sized_body(data.len(),Cursor::new(data))
            .ok()
    }
}

#[derive(Debug,Serialize,Deserialize,Eq, PartialEq,Copy, Clone)]
pub enum RtStatus {
    AuthFail,
    AuthSuccess,
    Success,
    Fail,
    Error,
}


#[derive(Debug,Serialize,Deserialize,Eq, PartialEq, Copy, Clone)]
pub enum DefaultData{
    Success(()),
    Failure(())
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct SignData {
    pub name:String,
    pub id:String,
    pub org_id:String,
    pub org_name:String
}


#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserMsg {
    pub id: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserMsg {
    type Error = String;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        return if req
            .local_cache(|| AuthCheck {
                is_valid_token: false,
            })
            .is_valid_token
        {
            let my_config = req
                .rocket()
                .state::<TokenInfo>()
                .expect("get global custom config error in fairing");
            let token_field = my_config.token_field.as_str();
            let token_key = my_config.token_key.as_str();

            let header = req.headers();
            let token_data = header.get(token_field).next();
            if let Some(token) = token_data {
                let token = decode_token(token, token_key).unwrap();
                let id = token.claims.id;
                Outcome::Success(UserMsg { id })
            } else {
                Outcome::Failure((
                    Status::BadRequest,
                    String::from("user no login or token expired"),
                ))
            }
        } else {
            Outcome::Failure((
                Status::BadRequest,
                String::from("user no login or token expired"),
            ))
        }
    }
}


#[derive(Debug,Serialize,Deserialize,Clone,Eq, PartialEq,FromRow)]
pub struct LoginSuccessData{
    #[sqlx(rename = "id")]
    #[sqlx(try_from = "Uuid")]
    pub user_id:String,
    pub name:String,
    #[sqlx(default)]
    pub phone:Option<String>,
    #[sqlx(default)]
    pub gender:Option<String>,
    #[sqlx(default)]
    pub email:Option<String>,
    #[sqlx(try_from = "Uuid")]
    pub organization:String,
    pub org_name:String,
    #[sqlx(default)]
    pub work_id:Option<String>,
    pub create_time:NaiveDateTime,
    #[sqlx(default)]
    pub avatar:Option<String>,
    #[sqlx(default)]
    pub background:Option<String>,
    #[sqlx(default)]
    pub role:i16
}

impl<'r> Responder<'r,'static> for RtData<LoginSuccessData> {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let user_id = self.data.user_id.as_str().to_owned();

        let data = self.to_string();

        request.local_cache(||AuthCheck{
            is_valid_token:true
        });

        let (token_field,token) = set_token(request,user_id.as_str());

        Response::build()
            .header(ContentType::JSON)
            .raw_header(token_field,token)
            .sized_body(data.len(),Cursor::new(data)).ok()
    }
}


#[derive(Debug,Serialize,Deserialize,Eq, PartialEq,Clone)]
pub enum SingleEditResult {
    Exist(String),
    Success(String),
    Fail(String)
}

impl<'r> Responder<'r,'static> for RtData<SingleEditResult> {
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

#[derive(Debug,Serialize,Deserialize,Eq, PartialEq,Clone)]
pub enum DeleteResult{
    NoExist(String),
    Success(String),
}

impl<'r> Responder<'r,'static> for RtData<DeleteResult> {
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


#[derive(Debug,FromForm,Serialize,Deserialize,Eq, PartialEq,Clone)]
pub struct AddPartners {
    #[field(name="partners")]
    pub partners:Vec<String>,
    #[field(name="project_id")]
    pub project_id:String
}


#[derive(Debug,Serialize,Deserialize,Clone,FromRow,Eq, PartialEq)]
pub struct WorkMate{
    pub name:String,
    #[sqlx(default)]
    pub position:Option<String>,
    pub role:String,
}

#[derive(Debug,Serialize,Deserialize,Clone,FromRow,Eq, PartialEq)]
pub struct WorkMateCollector{
    pub collector:Vec<WorkMate>
}

impl<'r> Responder<'r,'static> for RtData<WorkMateCollector> {
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

#[derive(Debug,Serialize,Deserialize,Clone,FromRow,Eq, PartialEq)]
pub struct Project{
    #[sqlx(try_from="Uuid")]
    pub id:String,
    pub name:String,
    pub logo:String,
    #[sqlx(default)]
    pub description:Option<String>,
    pub last_update:NaiveDateTime,
    pub admin_name:String,
    #[sqlx(try_from="bool")]
    #[sqlx(rename="star")]
    pub if_star:bool
}

#[derive(Debug,Serialize,Deserialize,Eq, PartialEq,Clone)]
pub struct ProjectCollector{
    pub collector:Vec<Project>
}

impl<'r> Responder<'r,'static> for RtData<ProjectCollector> {
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

#[derive(Debug,Clone,Eq, PartialEq,Serialize,Deserialize,FromForm)]
pub struct ProjectSetter {
    #[field(name="userId")]
    pub user_id:String,
    #[field(name="id")]
    pub id:Option<String>,
    #[field(name="name")]
    pub name:String,
    #[field(name="logo")]
    pub logo:String,
    #[field(name="organization")]
    pub organization:String,
    #[field(name="description")]
    pub description:Option<String>,
    #[field(name="lastUpdate")]
    pub last_update:Option<String>,
    #[field(name="private")]
    pub private:Option<bool>
}

#[derive(Debug,Serialize,Deserialize,FromForm,Eq, PartialEq)]
pub struct ProjectStar{
    #[field(name="userId")]
    pub user_id:String,
    #[field(name="id")]
    pub pro_id:String,
    #[field(name="star")]
    pub star:bool
}

pub enum ObjectTypes{
    PROJECT,
    TEST,
    WIKI,
    TOPIC
}
