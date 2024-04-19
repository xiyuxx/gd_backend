use uuid::Uuid;
use chrono::NaiveDateTime;
use rocket::FromForm;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug,Serialize,Deserialize,Clone,FromRow,Eq, PartialEq)]
pub struct Project{
    #[sqlx(try_from="Uuid")]
    pub id:String,
    pub name:String,
    pub logo:String,
    #[sqlx(default)]
    pub description:Option<String>,
    pub last_update_time:NaiveDateTime
}

#[derive(Debug,Serialize,Deserialize,Eq, PartialEq,Clone)]
pub struct ProjectCollector{
    pub projects:Vec<Project>
}




#[derive(Debug,Clone,Eq, PartialEq,Serialize,Deserialize,FromForm)]
pub struct ProjectSetter {
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

