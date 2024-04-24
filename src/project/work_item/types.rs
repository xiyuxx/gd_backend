
use std::io::Cursor;
use chrono::NaiveDateTime;
use rocket::{FromForm, Request, Response};
use rocket::http::ContentType;
use rocket::response::Responder;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::auth::AuthCheck;
use crate::types::RtData;

#[derive(Debug,FromForm,Eq, PartialEq,Clone,Serialize,Deserialize)]
pub struct WorkItemSetter {
    pub id:Option<i32>,
    pub project_id:String,
    pub name:String,
    #[field(name="type")]
    pub item_type:Option<i32>,
    pub status:Option<i32>,
    pub principal:Option<String>,
    // shall be converted to NaiveDateTime
    pub create_time:Option<String>,
    #[field(name="father")]
    pub father_item: Option<i32>,
    pub priority: Option<i32>,
}

#[derive(Debug,Eq, PartialEq,Clone,Serialize,Deserialize,FromRow)]
pub struct WorkItemGetter {
    pub id:i32,
    pub name:String,
    #[sqlx(default)]
    #[sqlx(rename="type")]
    pub item_type:Option<i32>,
    #[sqlx(default)]
    pub status:Option<i32>,
    #[sqlx(default)]
    pub create_time:NaiveDateTime,
    #[sqlx(default)]
    pub father_item: Option<i32>,
    #[sqlx(default)]
    pub priority: Option<i32>,
    #[sqlx(default)]
    pub principal_name:Option<String>,
    #[sqlx(default)]
    pub principal_avatar:Option<String>,
}


#[derive(Debug,Serialize,Deserialize,Eq, PartialEq,Clone)]
pub struct WorkItemCollector{
    pub collector: Vec<WorkItemGetter>
}
impl<'r> Responder<'r,'static> for RtData<WorkItemCollector> {
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