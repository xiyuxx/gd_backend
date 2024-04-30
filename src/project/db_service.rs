use std::str::FromStr;
use jsonwebtoken::get_current_timestamp;
use sqlx::FromRow;
use sqlx::postgres::PgRow;
use uuid::Uuid;
use crate::db::{DbQueryResult, GdDBC, SqlxError};

use crate::types::{ObjectTypes, ProjectSetter};
use crate::types::{AddPartners, DeleteResult, SingleEditResult, WorkMate, WorkMateCollector};
use crate::utils::{get_sequence_name, timestamp_to_date};

pub async fn try_set_project(
    mut db:GdDBC,
    mut project_creator: ProjectSetter
) -> DbQueryResult<SingleEditResult> {
    let pro_c = project_creator.clone();
    let is_new = pro_c.id.is_none();

    let name = pro_c.name;
    let logo = pro_c.logo;
    let organization = Uuid::from_str(pro_c.organization.as_str()).unwrap() ;
    let desc = pro_c.description.as_ref().map(String::as_str);

    // project_id
    let pro_id;
    let update_time;
    if is_new {
        pro_id = Uuid::new_v4().to_string();
        update_time = timestamp_to_date(get_current_timestamp());
        project_creator.id = Some(pro_id.clone());

        let pro_seq_name =
            get_sequence_name(ObjectTypes::PROJECT, pro_id.clone());
        let sql = format!(
            "create SEQUENCE {}",pro_seq_name
        );
        dbg!(&sql);
        if let Err(err) = sqlx::query(&sql).execute(&mut *db).await {
            dbg!("create sequence fail");
            dbg!(err);
        }
    } else{
        pro_id = pro_c.id.unwrap();
        update_time = pro_c.last_update.unwrap();
    }

    let sql = format!(
        "insert into public.project values('{pro_id}','{name}','{logo}','{organization}',$1,'{update_time}')\
        on conflict(id) do update set name = '{name}',logo = '{logo}',organization = '{organization}',\
        description = $2, last_update = '{update_time}'"
    );
    match sqlx::query(&sql).bind(desc).bind(desc).fetch_one(&mut *db).await{
        // if insert or update success
        Ok(_) => {
            // judge insert or update by is_new
            // if is_new equals to true means is insert
            //  then execute insert on participation
            return if is_new {
                try_insert_on_participation(db,project_creator).await
            }
            // or it is update, just return
            else {
                Ok(SingleEditResult::Exist("project adjust successfully".to_string()))
            }
        }
        Err(err) => {
            if let SqlxError::RowNotFound = err{
                try_insert_on_participation(db,project_creator).await?;
            }
            Err(err)
        }
    }
}

pub async fn try_insert_on_participation(
    mut db:GdDBC,
    project_setter: ProjectSetter
) -> DbQueryResult<SingleEditResult> {
    let user_id = project_setter.user_id;
    let pro_id = project_setter.id.unwrap();

    let sql = format!(
        "insert into public.participation (user_id, project_id,  star) \
                values('{user_id}','{pro_id}','0')"
    );
    match sqlx::query(&sql)
        .fetch_one(&mut *db).await {
        // check whether the insert on participation success
        Ok(_) => {
            dbg!("用户{}参与项目{}成功！",user_id,pro_id);
            Ok(SingleEditResult::Success("participate project successfully".to_string()))
        }
        Err(err) => {
            if let SqlxError::RowNotFound = err {
                dbg!("用户{}参与项目{}成功！",user_id,pro_id);
                return Ok(SingleEditResult::Exist("participate project successfully".to_string()))
            }
            Err(err)
        }
    }
}

pub async fn select_project(
    mut db:GdDBC,
    user_id: String
) -> DbQueryResult<Vec<PgRow>> {
    // pro_id,name,logo,desc,last_update,admin_name,if_star
    // participation: pro_id,if_star
    // user: admin_name(user_id)
    // project: name,logo,desc,last_update
    let user_id = Uuid::from_str(user_id.as_str()).unwrap();
    let sql = r#"
    SELECT
        p.id,
        p.name,
        p.logo,
        p.description,
        p.last_update,
        admin.name AS admin_name,
        pa.star
    FROM public.project p
    JOIN public.participation pa ON p.id = pa.project_id
    LEFT JOIN public.user admin ON pa.user_id = admin.id
    WHERE pa.user_id = $1;
    "#;
    dbg!(&sql);
    sqlx::query(sql).bind(user_id)
        .fetch_all(&mut *db).await
}

pub async fn try_delete_project(
    mut db:GdDBC,
    project_id:String,
) -> DbQueryResult<DeleteResult> {
    let pro_id = Uuid::from_str(project_id.as_str()).unwrap();

    let delete_part =
        "delete from public.participation part \
        where part.project_id = $1".to_string();

    match sqlx::query(&delete_part).bind(pro_id.clone()).fetch_one(&mut *db).await {
        Ok(_) => {
            delete_from_project(db, pro_id).await
        }
        Err(err) => {
            if let SqlxError::RowNotFound = err {
                return Ok(delete_from_project(db, pro_id).await?);
            }
            dbg!(&err);
            Err(err)
        }
    }

}

async fn delete_from_project(
    mut db:GdDBC,
    pro_id:Uuid
)-> DbQueryResult<DeleteResult>{
    let delete_project = "delete from public.project pro where pro.id = $1".to_string();
    match sqlx::query(&delete_project).bind(pro_id)
        .fetch_one(&mut *db).await{
        Ok(_) => {
            Ok(DeleteResult::Success("delete project success".to_string()))
        }
        Err(err) => {
            if let SqlxError::RowNotFound = err {
                return Ok(DeleteResult::Success("delete project success".to_string()));
            }
            Err(err)
        }
    }
}

pub async fn try_add_partners_to_project(
    mut db:GdDBC,
    partners:AddPartners
) -> DbQueryResult<SingleEditResult> {
    let pro_id = Uuid::from_str(partners.project_id.as_str()).unwrap() ;
    let user_ids = partners.partners;
    let user_ids:Vec<_> = user_ids.into_iter().map(|user_id|{
        Uuid::from_str(user_id.as_str()).unwrap()
    }).collect::<Vec<Uuid>>();

    let sql = "INSERT INTO participation (user_id, project_id, role, star) \
        SELECT user_id, $1 AS project_id, 1 AS role, 'f' AS star
        FROM unnest($2::uuid[]) AS user_id".to_string();

    match sqlx::query(&sql).bind(pro_id).bind(&user_ids)
        .fetch_one(&mut *db).await {
        Ok(_) => { Ok(SingleEditResult::Success("add partners success".to_string())) }
        Err(err) => {
            if let SqlxError::RowNotFound = err {
                return Ok(SingleEditResult::Success("add partners success".to_string()))
            }
            Err(err)
        }
    }
}

// get all partners in particular project
pub async fn get_partners(
    mut db:GdDBC,
    project_id:String
) -> DbQueryResult<WorkMateCollector> {

    let pro_id = Uuid::from_str(project_id.as_str()).unwrap();
    let sql = "
    SELECT
        u.name,
        CASE
            WHEN p.role = 0 THEN '管理员'
            WHEN p.role = 1 THEN '普通成员'
            ELSE '未知角色'
        END AS role,
        pos.name AS position
    FROM
        public.participation p
    JOIN
        public.user u ON p.user_id = u.id
    LEFT JOIN
        public.position pos ON p.position = pos.id
    WHERE
        p.project_id = $1;
    ".to_string();
    let work_mates:Vec<_>;
    match sqlx::query(&sql).bind(pro_id)
        .fetch_all(&mut *db).await {
        Ok(v) => {
            work_mates = v.iter().map(|row| {
                WorkMate::from_row(row).unwrap()
            }).collect::<Vec<WorkMate>>();
            Ok(WorkMateCollector{
                collector:work_mates
            })
        }
        Err(err) => {
            dbg!(&err);
            Err(err)
        }
    }
}