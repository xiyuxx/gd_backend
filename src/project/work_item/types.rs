use std::io::Cursor;
use rocket::{FromForm, Request, Response};
use rocket::http::ContentType;
use rocket::response::Responder;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::auth::AuthCheck;
use crate::types::RtData;

#[derive(Debug,FromForm,Eq, PartialEq,Clone,Serialize,Deserialize,FromRow)]
pub struct WorkItem{
    #[sqlx(default)]
    pub id:Option<i32>,
    pub project_id:String,
    pub name:String,
    #[sqlx(default)]
    #[field(name="type")]
    pub item_type:Option<i32>,
    #[sqlx(default)]
    pub status:Option<i32>,
    #[sqlx(default)]
    pub principal:Option<String>,
    // shall be converted to NaiveDateTime
    #[sqlx(default)]
    pub create_time:Option<String>,
    #[sqlx(default)]
    #[field(name="father")]
    pub father_item: Option<i32>,
    #[sqlx(default)]
    pub priority: Option<i32>,
}


#[derive(Debug,Serialize,Deserialize,Eq, PartialEq,Clone)]
pub struct WorkItemCollector{
    pub collector: Vec<WorkItem>
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