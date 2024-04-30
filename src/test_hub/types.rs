use uuid::Uuid;

use std::io::Cursor;
use chrono::NaiveDateTime;
use rocket::{FromForm, Request, Response};
use rocket::http::ContentType;
use rocket::response::Responder;
use rocket::serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::auth::AuthCheck;
use crate::types::RtData;

#[derive(Debug,Serialize,Deserialize,Clone,FromRow,Eq, PartialEq)]
pub struct TestHub{
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
pub struct TestHubCollector{
    pub collector:Vec<TestHub>
}

impl<'r> Responder<'r,'static> for RtData<TestHubCollector> {
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

#[derive(Debug,Serialize,Deserialize,FromForm,Eq, PartialEq)]
pub struct TestHubStar{
    #[field(name="userId")]
    pub user_id:String,
    #[field(name="id")]
    pub pro_id:String,
    #[field(name="star")]
    pub star:bool
}