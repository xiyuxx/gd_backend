use std::io::Cursor;
use uuid::Uuid;
use chrono::NaiveDateTime;
use rocket::{FromForm, Request, Response};
use rocket::http::ContentType;
use rocket::response::Responder;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::auth::AuthCheck;
use crate::types::RtData;

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
    pub last_update:Option<String>
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


#[derive(Debug,FromForm,Serialize,Deserialize,Eq, PartialEq,Clone)]
pub struct AddPartners {
    #[field(name="partners")]
    pub partners:Vec<String>,
    #[field(name="project_id")]
    pub project_id:String
}
