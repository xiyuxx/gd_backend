use rocket::form::Form;
use rocket::http::Status;
use rocket::post;
use crate::db::GdDBC;
use crate::test_hub::db_service::try_set_hub;
use crate::types::{ProjectSetter, RtData, SingleEditResult};
use crate::utils;

#[post("/set_test_hub",data="<hub_data>")]
pub async fn set_test_hub(
    gd:GdDBC,
    hub_data:Form<ProjectSetter>
) -> Result<RtData<SingleEditResult>,Status> {
    let res = try_set_hub(gd, hub_data.into_inner()).await;

    utils::match_insert_res(res, "create hub success".to_string())
}
