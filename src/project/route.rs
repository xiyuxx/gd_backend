use rocket::{ post};
use rocket::form::Form;
use rocket::http::Status;
use crate::db::{GdDBC, SqlxError};
use crate::project::db_service::try_insert_project;
use crate::project::types::{ProjectSetter};
use crate::types::{InsertResult, RtData, RtStatus};

#[post("/set_project",data="<project_data>")]
pub async fn set_project(
    db:GdDBC,
    project_data:Form<ProjectSetter>
) -> Result<RtData<InsertResult>,Status>{
    let res = try_insert_project(db,project_data.into_inner()).await;

    match res{
        Ok(adjust_res) => {
            let success = true;
            let status = RtStatus::Success;
            let msg = String::from("");

            Ok(RtData{
                success,status,msg,data:adjust_res
            })
        }
        Err(err) => {
            match err {
                SqlxError::RowNotFound => {
                    Ok(RtData{
                        success:true,
                        status:RtStatus::Success,
                        msg:String::from(""),
                        data:InsertResult::Success("create project success".to_string()),
                    })
                }
                _ => {
                    dbg!(err);
                    Err(Status::InternalServerError)
                }
            }
        }
    }
}