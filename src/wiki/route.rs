use std::str::FromStr;
use rocket::form::Form;
use rocket::http::Status;
use rocket::{get, post};
use sqlx::FromRow;
use uuid::Uuid;
use crate::db::{GdDBC, SqlxError};
use crate::types::{Project, ProjectCollector, ProjectSetter, RtData, RtStatus, SingleEditResult};
use crate::utils;
use crate::utils::match_insert_res;
use crate::wiki::db_service::{select_space, try_get_all_articles, try_set_article};
use crate::wiki::db_service::try_set_wiki;
use crate::wiki::types::{ArticleCollector, ArticleSetter, WikiStar};

#[post("/set_wiki",data="<project_data>")]
pub async fn set_wiki(
    db:GdDBC,
    project_data:Form<ProjectSetter>
) -> Result<RtData<SingleEditResult>,Status>{
    let res = try_set_wiki(db, project_data.into_inner()).await;

    utils::match_insert_res(res, "create space success".to_string())
}

#[get("/get_wiki?<id>")]
pub async fn get_wiki(
   db:GdDBC,
   id:String
) -> Result<RtData<ProjectCollector>,Status>{
    let res = select_space(db,id).await;
    match res {
        Ok(v) => {
            dbg!("总共行数：",v.len());
            // do not make sure the type in here
            let projects:Vec<_>;
            projects = v.iter().map(|row| Project::from_row(row).unwrap()).collect::<Vec<Project>>();
            Ok(RtData{
                success:true,
                msg:String::from("get all space success"),
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

#[post("/set_star",data="<wiki_star>")]
pub async fn set_wiki_star(
    mut db:GdDBC,
    wiki_star:Form<WikiStar>
)-> Result<RtData<SingleEditResult>,Status>{
    let wiki_star = wiki_star.into_inner();
    let user_id = Uuid::from_str(wiki_star.user_id.as_str()).unwrap();
    let wiki_id = Uuid::from_str(wiki_star.wiki_id.as_str()).unwrap();
    let star = wiki_star.star;

    let if_exist = "select * from public.wiki_participation \
    where user_id = $1 and wiki_id = $2";
    if let Err(e) = sqlx::query(if_exist).bind(user_id.clone()).bind(wiki_id.clone())
        .fetch_one(&mut *db).await{
        return match e{
            SqlxError::RowNotFound => {
                Ok(RtData{
                    data:SingleEditResult::Fail("no this wiki".to_string()),
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

    let sql = "update public.wiki_participation set star = $1 \
    where user_id = $2 and wiki_id = $3";
    match sqlx::query(sql).bind(star).bind(user_id).bind(wiki_id).execute(&mut *db).await {
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

#[get("/all?<wiki_id>")]
pub async fn get_all_articles(
    db:GdDBC,
    wiki_id:String,
) -> Result<RtData<ArticleCollector>,Status> {
    let res = try_get_all_articles(db,wiki_id).await;

    match res {
        Ok(data) => {
            Ok(RtData{
                data,
                msg: "get all articles success".to_string(),
                success: true,
                status: RtStatus::Success,
            })
        }
        Err(_) => {
            Err(Status::InternalServerError)
        }
    }
}

#[post("/set",data="<article>")]
pub async fn set_article(
    db:GdDBC,
    article:Form<ArticleSetter>
) -> Result<RtData<SingleEditResult>,Status>{
    let res = try_set_article(db, article.into_inner()).await;

    match_insert_res(res,"adjust article success".to_string())

}