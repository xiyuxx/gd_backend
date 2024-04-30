use rocket::form::Form;
use rocket::http::Status;
use rocket::{get, post};
use crate::db::{ GdDBC};
use crate::utils::match_insert_res;
use crate::project::work_item::db_service::{try_get_all_items, try_set_work_item};
use crate::project::work_item::types::{ WorkItemCollector, WorkItemSetter};
use crate::types::{SingleEditResult, RtData, RtStatus};

#[post("/set_work_item",data="<work_item>")]
pub async fn set_item(
    gd:GdDBC,
    work_item:Form<WorkItemSetter>
)-> Result<RtData<SingleEditResult>,Status> {
    let res = try_set_work_item(gd, work_item.into_inner()).await;

    match_insert_res(res,"insert work item success".to_string())

}

#[get("/get_items?<project_id>")]
pub async fn get_all_item(
    gd:GdDBC,
    project_id:String,
) -> Result<RtData<WorkItemCollector>,Status> {
    let res = try_get_all_items(gd,project_id).await;

    match res {
        Ok(data) => {
            Ok(RtData{
                data,
                msg: "get all items success".to_string(),
                success: true,
                status: RtStatus::Success,
            })
        }
        Err(_) => {
            Err(Status::InternalServerError)
        }
    }

}