use std::str::FromStr;
use sqlx::postgres::{ PgRow};
use uuid::Uuid;
use jsonwebtoken::get_current_timestamp;
use sqlx::{FromRow};
use crate::db::{DbQueryResult, GdDBC, SqlxError};
use crate::types::{ObjectTypes, ProjectSetter, SingleEditResult};
use crate::utils::{get_sequence_name, timestamp_to_date};
use crate::wiki::types::{ArticleCollector, ArticleGetter, ArticleSetter};

pub async fn select_space(
    mut db:GdDBC,
    user_id:String,
) -> DbQueryResult<Vec<PgRow>> {
    let user_id = Uuid::from_str(user_id.as_str()).unwrap();
    let sql = r#"
    SELECT
        w.id,
        w.name,
        w.logo,
        w.description,
        w.last_update,
        admin.name AS admin_name,
        wp.star
    FROM public.wiki w
    JOIN public.wiki_participation wp ON w.id = wp.wiki_id
    LEFT JOIN public.user admin ON wp.user_id = admin.id
    WHERE wp.user_id = $1;
    "#;
    dbg!(&sql);
    sqlx::query(sql).bind(user_id)
        .fetch_all(&mut *db).await
}

pub async fn try_set_wiki(
    mut db:GdDBC,
    mut wiki_setter:ProjectSetter
) -> DbQueryResult<SingleEditResult> {
    let pro_c = wiki_setter.clone();
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
        wiki_setter.id = Some(pro_id.clone());

        let pro_seq_name =
            get_sequence_name(ObjectTypes::WIKI, pro_id.clone());
        let sql = format!(
            "create SEQUENCE {}",pro_seq_name
        );
        if let Err(err) = sqlx::query(&sql).execute(&mut *db).await {
            dbg!("create sequence fail");
            dbg!(&err);
            return Err(err);
        }


    } else{
        pro_id = pro_c.id.unwrap();
        update_time = pro_c.last_update.unwrap();
    }

    let private = wiki_setter.private.unwrap_or_else(|| false);
    let sql = format!(
        "insert into public.wiki values('{pro_id}','{name}','{logo}','{organization}',$1,'{update_time}','{private}')\
        on conflict(id) do update set name = '{name}',logo = '{logo}',organization = '{organization}',\
        description = $2, last_update = '{update_time}',private = '{private}'"
    );
    match sqlx::query(&sql).bind(desc).bind(desc).fetch_one(&mut *db).await{
        Ok(_) => {
            return if is_new {
                try_insert_on_wiki_participation(db, wiki_setter).await
            }
            else {
                Ok(SingleEditResult::Exist("space adjust successfully".to_string()))
            }
        }
        Err(err) => {
            if let SqlxError::RowNotFound = err{
                try_insert_on_wiki_participation(db, wiki_setter).await?;
            }
            Err(err)
        }
    }
}

pub async fn try_insert_on_wiki_participation(
    mut db:GdDBC,
    wiki_setter:ProjectSetter
) -> DbQueryResult<SingleEditResult> {
    let user_id = wiki_setter.user_id;
    let pro_id = wiki_setter.id.unwrap();

    let sql = format!(
        "insert into public.wiki_participation (user_id, wiki_id,  star) \
                values('{user_id}','{pro_id}','0')"
    );
    match sqlx::query(&sql)
        .fetch_one(&mut *db).await {
        Ok(_) => {
            dbg!("用户{}参与{}空间成功！",user_id,pro_id);
            Ok(SingleEditResult::Success("participate wiki successfully".to_string()))
        }
        Err(err) => {
            if let SqlxError::RowNotFound = err {
                dbg!("用户{}参与{}空间成功！",user_id,pro_id);
                return Ok(SingleEditResult::Exist("participate wiki successfully".to_string()))
            }
            Err(err)
        }
    }
}

pub async fn try_get_all_articles(
    mut db:GdDBC,
    wiki_id: String
) -> DbQueryResult<ArticleCollector> {
    let pro_id = Uuid::from_str(wiki_id.as_str()).unwrap();

    let item_collector:Vec<_>;
    let sql = "select a.id, a.title,  \
    a.content, a.last_update, a.father_id, a.last_update \
    u.name update_name, u.avatar update_avatar \
    from public.article a \
    left join public.user u on a.update_id = u.id \
    where a.wiki_id = $1".to_string();
    match sqlx::query(&sql).bind(pro_id)
        .fetch_all(&mut *db).await {
        Ok(v) => {
            dbg!("开始类型转化");
            item_collector = v.iter().map(|row| {
                dbg!("转换中");
                let work_item: ArticleGetter = ArticleGetter::from_row(row).unwrap();
                dbg!(&work_item);
                work_item
            }).collect::<Vec<ArticleGetter>>();
            dbg!("whats wrong " ,item_collector.len());
            Ok(ArticleCollector {
                collector:item_collector
            })
        }
        Err(err) => {
            dbg!(&err);
            Err(err)
        }
    }
}


pub async fn try_set_article(
    mut db:GdDBC,
    article:ArticleSetter
) -> DbQueryResult<SingleEditResult> {
    let is_new = article.id.is_none();

    let wiki_id = Uuid::from_str(article.wiki_id.clone().as_str()).unwrap();
    let title = article.title;
    let content = article.content;
    let update_id = Uuid::from_str(article.update_id.clone().as_str()).unwrap();
    let father_id = article.father_id;
    let last_update = article.last_update
        .map_or(timestamp_to_date(get_current_timestamp()),|v|v.replace("T"," "));
    if is_new {
        let seq = get_sequence_name(ObjectTypes::WIKI,article.wiki_id);
        let use_seq_sql = format!("alter table public.article alter column id \
        set default NEXTVAL('{seq}')");

        match sqlx::query(&use_seq_sql).execute(&mut *db).await {
            Ok(_) => {
                let sql = format!(
                    "insert into public.article (wiki_id, title, content, \
                    update_id, last_update, father_id ) values ($1,$2,$3,$4,'{last_update}',$5)"
                );
                match sqlx::query(&sql).bind(wiki_id).bind(title).bind(content).bind(update_id)
                    .bind(father_id).execute(&mut *db).await {
                    Ok(_) => { Ok(SingleEditResult::Success("insert article success".to_string())) }
                    Err(err) => {
                        if let SqlxError::RowNotFound = err {
                            return Ok(SingleEditResult::Success("insert article success".to_string()))
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
        let id = article.id.unwrap();
        let sql = format!("\
        update public.article set title = $1, content = $2, update_id = $3,\
         last_update = '{last_update}',father_id = $4 where id = $5 and wiki_id = $6\
        ");
        return match sqlx::query(&sql).bind(title).bind(content).bind(update_id)
            .bind(father_id).bind(id).bind(wiki_id).execute(&mut *db).await {
            Ok(_) => { Ok(SingleEditResult::Success("update article success".to_string())) }
            Err(err) => {
                dbg!(&err);
                if let SqlxError::RowNotFound = err {
                    return Ok(SingleEditResult::Success("update article success".to_string()));
                }
                Err(err)
            }
        }

    }
}