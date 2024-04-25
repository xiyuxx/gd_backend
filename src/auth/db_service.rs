
use std::str::FromStr;
use jsonwebtoken::get_current_timestamp;
use sqlx::postgres::{ PgRow};
use uuid::Uuid;
use crate::auth::{MoreUser, RegisterResult};
use crate::db::{DbQueryResult, GdDBC, SqlxError};

use crate::types::SignData;
use crate::utils::timestamp_to_date;

pub async fn try_register_user(
    mut db: GdDBC,
    user_data: MoreUser
) -> DbQueryResult<RegisterResult> {
    let id = Uuid::new_v4();
    let mut sql = String::from("");
    match user_data {
        MoreUser::Add(worker) => {
            let (name,pwd,organization,
                 phone, email,
                gender,work_id) = worker.into();
            let test = Uuid::from_str(organization.as_str()).unwrap();
            if let Some(phone_c) = &phone {
                sql = format!("select * from public.user where phone = '{phone_c}' limit 1");
            } else if let Some(email_c) = &email {
                sql = format!("select * from public.user where email = '{email_c}' limit 1");
            }
            dbg!("用户检测不存在");
            let res = if let Err(db_err) = sqlx::query(&sql).fetch_one(&mut *db).await{
                match db_err {
                    SqlxError::RowNotFound =>{
                        let pwd = format!("{:x}",md5::compute(pwd));
                        let create_time = timestamp_to_date(get_current_timestamp());
                        dbg!("time",&create_time);
                        let phone = phone.as_ref().map(String::as_str);
                        let email = email.as_ref().map(String::as_str);
                        let gender = gender.as_ref().map(String::as_str);
                        let work_id = work_id.as_ref().map(String::as_str);
                        dbg!("尝试插入用户");
                        let query = format!("
                        INSERT INTO public.user (id, name, pwd, phone,
                        gender, email,organization, work_id, create_time)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, '{create_time}')
                        ");
                        match sqlx::query(&query)
                            .bind(id).bind(name.clone()).bind(pwd).bind(phone).bind(gender)
                            .bind(email).bind(test).bind(work_id)
                            .fetch_one(&mut *db).await {
                            Ok(_) => RegisterResult::Success(SignData { name, id: id.to_string()}),
                            Err(err) => {
                                if let SqlxError::RowNotFound = err {
                                    return Ok(RegisterResult::Success(SignData { name, id: id.to_string() }));
                                }
                                return Err(err);
                            }
                        }
                    }

                    _ => return Err(db_err)
                }

            } else {
                return Ok(RegisterResult::Exist(String::from("\
                email or phone had been registered!")));
            };

            Ok(res)
        }
        MoreUser::Create(creator) => {
            let (name,pwd,phone,_organization) = creator.clone().into();
            let phone_c = phone.clone();
            sql = format!("select * from public.user where phone = '{phone_c}' limit 1");
            dbg!("检测用户是否存在");
            let res = if let Err(db_err) = sqlx::query(&sql).fetch_one(&mut *db).await{
                match db_err {
                    SqlxError::RowNotFound =>{
                        dbg!("用户未注册");

                        dbg!("开始注册公司");
                        let org_id = Uuid::new_v4();
                        let org = creator.organization.clone();
                        match sqlx::query(
                            "insert into public.organization values($1,$2)"
                        ).bind(org_id).bind(org).execute(&mut *db).await {
                            Ok(_) => {
                                dbg!("组织添加成功！开始注册账号");
                                let org_id_str =  org_id.to_string();
                                let pwd = format!("{:x}",md5::compute(pwd));
                                let create_time = timestamp_to_date(get_current_timestamp());
                                let insert_key = "id,name,pwd,phone,create_time,organization".to_string();

                                let insert_values =
                                    format!("'{id}','{name}','{pwd}','{phone}','{create_time}','{org_id_str}'");
                                sql = format!("insert into public.user ({insert_key}) values ({insert_values})");
                                register_user(db,sql,id,name).await?
                            }
                            Err(err) => {dbg!(&err); return Err(err)}
                        }
                    }
                    _ => {
                        dbg!("weird problem happened");
                        return Err(db_err);
                    }
                }
            } else {
                return Ok(RegisterResult::Exist(String::from("\
                email or phone had been registered!")));
            };

            Ok(res)
        }
    }
}

async fn register_user(
    mut db:GdDBC,
    sql:String,
    id:Uuid,
    name:String
) -> DbQueryResult<RegisterResult> {
    dbg!(&sql);

    return match sqlx::query(&sql).fetch_one(&mut *db).await {
        Ok(_) => {
            dbg!("insert user success");
            Ok(RegisterResult::Success(SignData { name, id: id.to_string()}))
        },
        Err(err) => {
            dbg!("执行插入后报错");
            if let SqlxError::RowNotFound = err {
                return Ok(RegisterResult::Success(SignData{name, id: id.to_string()}))
            }
            Err(err)
        }
    }
}

pub async fn get_user_msg(
    (login_key,pwd,is_email):(String,String,bool),
    mut db:GdDBC
) -> DbQueryResult<PgRow> {
    let condition = if is_email {
        format!("email = '{login_key}'")
    } else {
        format!("phone = '{login_key}'")
    };
    
    let pwd = format!("{:x}",md5::compute(pwd));
    let sql = format!(
        "select * from public.user where pwd = '{pwd}' and {condition}"
    );
    
    dbg!(&sql);
    
    sqlx::query(&sql).fetch_one(&mut *db).await
}

