use rocket::form::Form;
use rocket::http::Status;
use rocket::post;
use crate::db::{GdDBC};
use crate::project::match_insert_res;
use crate::project::work_item::db_service::try_set_work_item;
use crate::project::work_item::types::WorkItem;
use crate::types::{SingleEditResult, RtData};

#[post("/add_work_item",data="<work_item>")]
pub async fn set_item(
    gd:GdDBC,
    work_item:Form<WorkItem>
)-> Result<RtData<SingleEditResult>,Status> {
    let res = try_set_work_item(gd, work_item.into_inner()).await;

    match_insert_res(res,"insert work item success".to_string())

}