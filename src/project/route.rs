use std::str::FromStr;
use rocket::{get, post};
use rocket::form::Form;
use rocket::http::Status;
use sqlx::{Error, FromRow};
use sqlx::postgres::PgQueryResult;
use uuid::Uuid;
use crate::db::{GdDBC, SqlxError};
use crate::project;
use crate::project::db_service::{get_partners, select_project, try_add_partners_to_project, try_delete_project, try_insert_project};
use crate::project::types::{AddPartners, Project, ProjectCollector, ProjectSetter, ProjectStar, WorkMateCollector};
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

#[post("/set_star",data="<pro_star>")]
pub async fn set_pro_star(
    mut db:GdDBC,
    pro_star:Form<ProjectStar>
)-> Result<RtData<SingleEditResult>,Status>{
    let pro_star = pro_star.into_inner();
    let user_id = Uuid::from_str(pro_star.user_id.as_str()).unwrap();
    let pro_id = Uuid::from_str(pro_star.pro_id.as_str()).unwrap();
    let star = pro_star.star;

    let if_exist = "select * from public.participation \
    where user_id = $1 and project_id = $2";
    if let Err(e) = sqlx::query(if_exist).bind(user_id.clone()).bind(pro_id.clone())
        .fetch_one(&mut *db).await{
        return match e{
            SqlxError::RowNotFound => {
                Ok(RtData{
                    data:SingleEditResult::Fail("no this project".to_string()),
                    msg: "".to_string(),
                    success: false,
                    status: RtStatus::Fail,
                })
            },
            _ => {
                Err(Status::InternalServerError)
            }
        }
    }

    let sql = "update public.participation set star = $1 \
    where user_id = $2 and project_id = $3";
    match sqlx::query(sql).bind(star).bind(user_id).bind(pro_id).execute(&mut *db).await {
        Ok(_) => {
            Ok(RtData {
                data: SingleEditResult::Success("更新成功".to_string()),
                msg: "".to_string(),
                success: true,
                status: RtStatus::Success,
            })
        }
        Err(err) => {
            dbg!(err);
            Err(Status::InternalServerError)
        }
    }

}