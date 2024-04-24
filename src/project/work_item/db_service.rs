use std::str::FromStr;
use jsonwebtoken::get_current_timestamp;
use uuid::Uuid;
use crate::db::{DbQueryResult, GdDBC, SqlxError};
use crate::project::work_item::types::WorkItem;
use crate::types::SingleEditResult;
use crate::utils::{get_work_item_seq, timestamp_to_date};

pub async fn try_set_work_item(
    mut db: GdDBC,
    work_item:WorkItem
) -> DbQueryResult<SingleEditResult> {
    let is_new = work_item.id.is_none();

    let pro_id = Uuid::from_str(work_item.project_id.clone().as_str()).unwrap() ;
    let name = work_item.name;
    let item_type = work_item.item_type;
    let status = work_item.status;
    let principal = work_item.principal.as_ref().map(|s| Uuid::parse_str(&s).unwrap());
    let father_item = work_item.father_item;
    let priority = work_item.priority;

    if is_new {
        // if is insert operation, change the sequence
        let seq = get_work_item_seq(work_item.project_id);
        dbg!(&seq);
        let use_seq_sql = format!("alter table public.work_item alter column id \
        set default NEXTVAL('{seq}')");

        match sqlx::query(&use_seq_sql).execute(&mut *db).await {
            Ok(_) => {
                let create_time = timestamp_to_date(get_current_timestamp());
                let sql = format!(
                    "insert into public.work_item (name, type, status, principal, create_time, \
                    father_item, priority, project_id) values ($1,$2,$3,$4,'{create_time}',$5,$6,$7)"
                );
                match sqlx::query(&sql).bind(name).bind(item_type).bind(status).bind(principal)
                    .bind(father_item).bind(priority).bind(pro_id).execute(&mut *db).await {
                    Ok(_) => { Ok(SingleEditResult::Success("insert work item success".to_string())) }
                    Err(err) => {
                        if let SqlxError::RowNotFound = err {
                            return Ok(SingleEditResult::Success("insert work item success".to_string()))
                        }
                        dbg!(&err);
                        Err(err)
                    }
                }
            }
            Err(err) => {
                dbg!(&err);
                return Err(err);
            }
        }
    } else {
        // if is update operation
        let id = work_item.id.unwrap();
        let sql = format!(
            "update public.work_item set name = '{name}', type = $1, status = $2, principal = $3, \
            father_item = $4, priority = $5 where id = '{id}' and project_id = '{pro_id}'"
        );
        return match sqlx::query(&sql).bind(item_type).bind(status).bind(principal).bind(father_item)
            .bind(priority).execute(&mut *db).await {
            Ok(_) => { Ok(SingleEditResult::Success("update work item success".to_string())) }
            Err(err) => {
                dbg!(&err);
                if let SqlxError::RowNotFound = err {
                    return Ok(SingleEditResult::Success("update work item success".to_string()));
                }
                Err(err)
            }
        }
    }


}