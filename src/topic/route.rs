use rocket::form::Form;
use rocket::http::Status;
use rocket::post;
use crate::db::GdDBC;
use crate::topic::db_service::try_set_topic;
use crate::types::{ProjectSetter, RtData, SingleEditResult};
use crate::utils;

#[post("/set_project",data="<topic_data>")]
pub async fn set_topic(
    db:GdDBC,
    topic_data:Form<ProjectSetter>
) -> Result<RtData<SingleEditResult>,Status>{
    let res = try_set_topic(db, topic_data.into_inner()).await;

    utils::match_insert_res(res, "create topic success".to_string())
}
