
use rocket::{get, post};
use rocket::form::Form;
use rocket::http::Status;
use sqlx::FromRow;
use crate::db::{GdDBC};
use crate::project;
use crate::project::db_service::{get_partners, select_project, try_add_partners_to_project, try_delete_project, try_insert_project};
use crate::project::types::{AddPartners, Project, ProjectCollector, ProjectSetter, WorkMateCollector};
use crate::types::{DeleteResult, SingleEditResult, RtData, RtStatus};

#[post("/set_project",data="<project_data>")]
pub async fn set_project(
    db:GdDBC,
    project_data:Form<ProjectSetter>
) -> Result<RtData<SingleEditResult>,Status>{
    let res = try_insert_project(db,project_data.into_inner()).await;

    project::match_insert_res(res, "create project success".to_string())
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
                    collector: projects
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

#[get("/work_mate?<project_id>")]
pub async fn get_participants(
    db: GdDBC,
    project_id:String,
) -> Result<RtData<WorkMateCollector>,Status> {
    let res = get_partners(db,project_id).await;

    match res {
        Ok(data) => {
            let success = true;
            let status = RtStatus::Success;
            let msg = String::from("get all partners success!");

            Ok(RtData{
                success,status,msg,data
            })
        }
        Err(_) => {
            Err(Status::InternalServerError)
        }
    }
}

#[post("/add_workmate",data="<partners>")]
pub async fn add_workmate_to_project(
    db:GdDBC,
    partners: Form<AddPartners>
)-> Result<RtData<SingleEditResult>,Status>{

    let res = try_add_partners_to_project(db,partners.into_inner()).await;
    project::match_insert_res(res, "add partners success".to_string())

}