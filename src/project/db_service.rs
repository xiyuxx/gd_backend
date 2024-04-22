use std::str::FromStr;
use jsonwebtoken::get_current_timestamp;
use sqlx::postgres::PgRow;
use uuid::Uuid;
use crate::db::{DbQueryResult, GdDBC, SqlxError};

use crate::project::types::{AddPartners, ProjectSetter};
use crate::types::{DeleteResult, InsertResult};
use crate::utils::timestamp_to_date;

pub async fn try_insert_project(
    mut db:GdDBC,
    mut project_creator: ProjectSetter
) -> DbQueryResult<InsertResult> {
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
                Ok(InsertResult::Exist("project adjust successfully".to_string()))
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
) -> DbQueryResult<InsertResult> {
    let user_id = project_setter.user_id;
    let pro_id = project_setter.id.unwrap();

    let sql = format!(
        "insert into public.participation (user_id, project_id, role, star) \
                values('{user_id}','{pro_id}','0','0')"
    );
    match sqlx::query(&sql)
        .fetch_one(&mut *db).await {
        // check whether the insert on participation success
        Ok(_) => {
            dbg!("用户{}参与项目{}成功！",user_id,pro_id);
            Ok(InsertResult::Success("participate project successfully".to_string()))
        }
        Err(err) => {
            if let SqlxError::RowNotFound = err {
                dbg!("用户{}参与项目{}成功！",user_id,pro_id);
                return Ok(InsertResult::Exist("participate project successfully".to_string()))
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
    LEFT JOIN public.user admin ON pa.user_id = admin.id AND pa.role = 0
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
) -> DbQueryResult<InsertResult> {
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
        Ok(_) => { Ok(InsertResult::Success("add partners success".to_string())) }
        Err(err) => {
            if let SqlxError::RowNotFound = err {
                return Ok(InsertResult::Success("add partners success".to_string()))
            }
            Err(err)
        }
    }
}