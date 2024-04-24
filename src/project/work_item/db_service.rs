use std::str::FromStr;
use jsonwebtoken::get_current_timestamp;
use sqlx::{FromRow};
use uuid::Uuid;
use crate::db::{DbQueryResult, GdDBC, SqlxError};
use crate::project::work_item::types::{WorkItemCollector, WorkItemGetter, WorkItemSetter};
use crate::types::SingleEditResult;
use crate::utils::{get_work_item_seq, timestamp_to_date};

pub async fn try_set_work_item(
    mut db: GdDBC,
    work_item:WorkItemSetter
) -> DbQueryResult<SingleEditResult> {
    let is_new = work_item.id.is_none();

    let pro_id = Uuid::from_str(work_item.project_id.clone().as_str()).unwrap() ;
    let name = work_item.name;
    let item_type = work_item.item_type;
    let status = work_item.status;
    let principal = work_item.principal.as_ref().map(|s| Uuid::parse_str(&s).unwrap());
    let father_item = work_item.father_item;
    let priority = work_item.priority;
    let desc_final;
    if let Some(desc) = work_item.description {
        desc_final = desc;
    }else {
        desc_final = "-".to_string();
    }
    if is_new {
        // if is insert operation, change the sequence
        let seq = get_work_item_seq(work_item.project_id);
        let use_seq_sql = format!("alter table public.work_item alter column id \
        set default NEXTVAL('{seq}')");

        match sqlx::query(&use_seq_sql).execute(&mut *db).await {
            Ok(_) => {
                let create_time = timestamp_to_date(get_current_timestamp());
                let sql = format!(
                    "insert into public.work_item (name, type, status, principal, create_time, \
                    father_item, priority, project_id, description) values ($1,$2,$3,$4,'{create_time}',$5,$6,$7,$8)"
                );
                match sqlx::query(&sql).bind(name).bind(item_type).bind(status).bind(principal)
                    .bind(father_item).bind(priority).bind(pro_id).bind(desc_final).execute(&mut *db).await {
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
            father_item = $4, priority = $5, description = '{desc_final}' where id = '{id}' and project_id = '{pro_id}'"
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

pub async fn try_get_all_items(
    mut db:GdDBC,
    project_id: String
) -> DbQueryResult<WorkItemCollector> {
    let pro_id = Uuid::from_str(project_id.as_str()).unwrap();

    let item_collector:Vec<_>;
    let sql = "select wi.id, wi.name, wi.type, \
    wi.status, wi.create_time, wi.father_item, wi.priority, wi.desc \
    u.name principal_name, u.avatar principal_avatar \
    from public.work_item wi \
    left join public.user u on wi.principal = u.id \
    where wi.project_id = $1".to_string();

    match sqlx::query(&sql).bind(pro_id)
        .fetch_all(&mut *db).await {
        Ok(v) => {
            dbg!("开始类型转化");

            item_collector = v.iter().map(|row| {
                dbg!("转换中");
                let work_item: WorkItemGetter = WorkItemGetter::from_row(row).unwrap();
                dbg!(&work_item);
                work_item
            }).collect::<Vec<WorkItemGetter>>();
            dbg!("whats wrong " ,item_collector.len());
            Ok(WorkItemCollector{
                collector:item_collector
            })
        }
        Err(err) => {
            dbg!(&err);
            Err(err)
        }
    }
}