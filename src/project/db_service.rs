use std::str::FromStr;
use jsonwebtoken::get_current_timestamp;
use uuid::Uuid;
use crate::db::{DbQueryResult, GdDBC, SqlxError};
use crate::project::types::{ProjectSetter};
use crate::types::InsertResult;
use crate::utils::timestamp_to_date;

pub async fn try_insert_project(
    mut db:GdDBC,
    project_creator: ProjectSetter
) -> DbQueryResult<InsertResult> {
    let pro_c = project_creator.clone();
    // project_id
    let id = if let Some(pro_id) = pro_c.id{
        pro_id
    }else{
        Uuid::new_v4().to_string()
    };
    let update_time = if let Some(last) = pro_c.last_update{
        last
    }else{
        timestamp_to_date(get_current_timestamp())
    };
    let name = pro_c.name;
    let logo = pro_c.logo;
    let organization = Uuid::from_str(pro_c.organization.as_str()).unwrap() ;
    let desc = pro_c.description.as_ref().map(String::as_str);

    let sql = format!(
        "insert into public.project values('{id}','{name}','{logo}','{organization}',$1,'{update_time}')\
        on conflict(id) do update set name = '{name}',logo = '{logo}',organization = '{organization}',\
        description = $2, last_update = '{update_time}'"
    );

    return match sqlx::query(&sql).bind(desc).bind(desc)
        .fetch_one(&mut *db).await {
        Ok(_) => Ok(InsertResult::Success("project adjust successfully".to_string())),
        Err(err) => {
            if let SqlxError::RowNotFound = err {
                return Ok(InsertResult::Success("project adjust successfully".to_string()))
            }
            Err(err)
        }
    }

}