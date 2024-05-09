use std::io::Cursor;
use chrono::NaiveDateTime;
use rocket::{FromForm, Request, Response};
use rocket::http::ContentType;
use rocket::response::Responder;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::auth::AuthCheck;
use crate::types::RtData;

#[derive(Debug,Serialize,Deserialize,FromForm,Eq, PartialEq)]
pub struct WikiStar{
    #[field(name="userId")]
    pub user_id:String,
    #[field(name="id")]
    pub wiki_id:String,
    #[field(name="star")]
    pub star:bool
}

#[derive(Debug,Serialize,Deserialize,Eq, PartialEq,FromForm,Clone)]
pub struct ArticleSetter{
    // 存在=》更新| 不存在=》新建
    #[field(name="id")]
    pub id:Option<i32>,
    #[field(name="wiki_id")]
    pub wiki_id:String,
    #[field(name="title")]
    pub title:String,
    #[field(name="content")]
    pub content:String,
    #[field(name="update_id")]
    pub update_id:String,
    #[field(name="father")]
    pub father_id:Option<i32>,
    pub last_update:Option<String>,
}

#[derive(Debug,Serialize,Deserialize,Eq, PartialEq,FromRow,Clone)]
pub struct ArticleGetter{
    pub id:i32,
    pub title:String,
    pub content:String,
    pub update_name:String,
    pub update_avatar:String,
    pub last_update:NaiveDateTime,
    #[sqlx(default)]
    pub father_id:Option<i32>,
}

#[derive(Debug,Serialize,Deserialize,Eq, PartialEq,Clone)]
pub struct ArticleCollector{
    pub collector: Vec<ArticleGetter>
}

impl<'r> Responder<'r,'static> for RtData<ArticleCollector> {
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
