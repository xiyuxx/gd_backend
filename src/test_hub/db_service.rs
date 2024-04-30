use std::str::FromStr;
use jsonwebtoken::get_current_timestamp;
use uuid::Uuid;
use crate::db::{DbQueryResult, GdDBC, SqlxError};
use crate::types::{ObjectTypes, ProjectSetter, SingleEditResult};
use crate::utils::{get_sequence_name, timestamp_to_date};

pub async fn try_set_hub(
    mut gd:GdDBC,
    mut hub_setter:ProjectSetter
) -> DbQueryResult<SingleEditResult> {
    let pro_c = hub_setter.clone();
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
        hub_setter.id = Some(pro_id.clone());

        let pro_seq_name = get_sequence_name(ObjectTypes::TEST, pro_id.clone());
        let sql = format!(
            "create SEQUENCE {}",pro_seq_name
        );
        dbg!(&sql);
        if let Err(err) = sqlx::query(&sql).execute(&mut *gd).await {
            dbg!("create sequence fail");
            dbg!(err);
        }
    } else{
        pro_id = pro_c.id.unwrap();
        update_time = pro_c.last_update.unwrap();
    }

    let sql = format!(
        "insert into public.test_hub values('{pro_id}','{name}','{logo}','{organization}',$1,'{update_time}')\
        on conflict(id) do update set name = '{name}',logo = '{logo}',organization = '{organization}',\
        description = $2, last_update = '{update_time}'"
    );
    dbg!(&sql);
    match sqlx::query(&sql).bind(desc).bind(desc).fetch_one(&mut *gd).await{
        Ok(_) => {

            return if is_new {
                try_insert_on_hub_participation(gd, hub_setter).await
            }
            // or it is update, just return
            else {
                Ok(SingleEditResult::Exist("hub adjust successfully".to_string()))
            }
        }
        Err(err) => {
            if let SqlxError::RowNotFound = err{
                try_insert_on_hub_participation(gd, hub_setter).await?;
            }
            Err(err)
        }
    }
}

pub async fn try_insert_on_hub_participation(
    mut gd:GdDBC,
    project_setter: ProjectSetter
) -> DbQueryResult<SingleEditResult> {
    let user_id = project_setter.user_id;
    let pro_id = project_setter.id.unwrap();

    let sql = format!(
        "insert into public.test_participation (user_id, hub_id,  star) \
                values('{user_id}','{pro_id}','0')"
    );
    match sqlx::query(&sql)
        .fetch_one(&mut *gd).await {
        // check whether the insert on participation success
        Ok(_) => {
            dbg!("用户{}参与测试库{}成功！",user_id,pro_id);
            Ok(SingleEditResult::Success("participate test hub successfully".to_string()))
        }
        Err(err) => {
            if let SqlxError::RowNotFound = err {
                dbg!("用户{}参与测试库{}成功！",user_id,pro_id);
                return Ok(SingleEditResult::Exist("participate test hub successfully".to_string()))
            }
            Err(err)
        }
    }
}