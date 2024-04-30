use rocket::form::Form;
use rocket::http::Status;
use rocket::post;
use crate::db::GdDBC;
use crate::types::{ProjectSetter, RtData, SingleEditResult};
use crate::utils;
use crate::wiki::types::try_set_wiki;

#[post("/set_wiki",data="<project_data>")]
pub async fn set_wiki(
    db:GdDBC,
    project_data:Form<ProjectSetter>
) -> Result<RtData<SingleEditResult>,Status>{
    let res = try_set_wiki(db, project_data.into_inner()).await;

    utils::match_insert_res(res, "create project success".to_string())
}