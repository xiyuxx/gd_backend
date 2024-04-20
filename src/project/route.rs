use rocket::{get, post};
use rocket::form::Form;
use rocket::http::Status;
use sqlx::FromRow;
use crate::db::{GdDBC, SqlxError};
use crate::project::db_service::{select_project, try_delete_project, try_insert_project};
use crate::project::types::{Project, ProjectCollector, ProjectSetter};
use crate::types::{DeleteResult, InsertResult, RtData, RtStatus};

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

#[get("/get_project?<user_id>")]
pub async fn get_project(
    db:GdDBC,
    user_id:String,
) -> Result<RtData<ProjectCollector>,Status> {

    let res = select_project(db,user_id).await;
    dbg!("开始执行查询所有项目");
    match res {
        Ok(v) => {
            dbg!("总共行数：",v.len());
            // do not make sure the type in here
            let projects:Vec<_>;
            projects = v.iter().map(|row| Project::from_row(row).unwrap()).collect::<Vec<Project>>();
            dbg!("查询到{}个项目",projects.len());
            Ok(RtData{
                success:true,
                msg:String::from("get all projects success"),
                data: ProjectCollector{
                    projects
                },
                status:RtStatus::Success,
            })
        }
        Err(err) => {
            dbg!("查询所有项目出错了",err);
            Err(Status::InternalServerError)
        }
    }
}

#[get("/delete_project?<project_id>")]
pub async fn delete_project(
    db:GdDBC,
    project_id:String
) -> Result<RtData<DeleteResult>,Status> {
    let res = try_delete_project(db,project_id).await;
    match res {
        Ok(data) => {
            Ok(RtData{
                data,
                msg:"delete project success".to_string(),
                status:RtStatus::Success,
                success:true
            })
        }
        Err(err) => {
            dbg!(err);
            Err(Status::InternalServerError)
        }
    }
}