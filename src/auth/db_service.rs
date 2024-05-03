
use std::str::FromStr;
use jsonwebtoken::get_current_timestamp;
use rocket::State;
use sqlx::postgres::{PgRow};
use sqlx::{ FromRow, Row};
use uuid::Uuid;
use crate::auth::{MoreUser, RegisterResult, User, UserCollector, UserGetter};
use crate::auth::validate::ValidateData;
use crate::db::{DbQueryResult, GdDBC, SqlxError};

use crate::types::{SignData, SingleEditResult};
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
                        dbg!(&create_time);
                        let phone = phone.as_ref().map(String::as_str);
                        let email = email.as_ref().map(String::as_str);
                        let gender = gender.as_ref().map(String::as_str);
                        let work_id = work_id.as_ref().map(String::as_str);
                        let role = 1;
                        dbg!("尝试插入用户");
                        let query = format!("
                        INSERT INTO public.user (id, name, pwd, phone,
                        gender, email,organization, work_id, create_time, role)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, '{create_time}',$9)
                        ");
                        match sqlx::query(&query)
                            .bind(id).bind(name.clone()).bind(pwd).bind(phone).bind(gender)
                            .bind(email).bind(test).bind(work_id).bind(role)
                            .execute(&mut *db).await {
                            Ok(_) => {
                                let mut org_id="".to_string();
                                let mut org_name="".to_string();
                                if let Ok(v) = sqlx::query(
                                    "select u.organization org_id, org.name org_name from \
                                    public.user u left join public.organization org \
                                    on u.organization = org.id where u.id = $1 "
                                ).bind(id).fetch_one(&mut *db).await {
                                    org_id = v.get::<Uuid,usize>(0).to_string();
                                    org_name = v.get(1);
                                }
                                RegisterResult::Success(SignData {
                                    name,
                                    id: id.to_string(),
                                    org_id,
                                    org_name
                                })
                            },
                            Err(err) => {
                                dbg!(&err);
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
                        let org_c = org.clone();
                        match sqlx::query(
                            "insert into public.organization values($1,$2)"
                        ).bind(org_id).bind(org).execute(&mut *db).await {
                            Ok(_) => {
                                dbg!("组织添加成功！开始注册账号");
                                let org_id_str =  org_id.to_string();
                                let pwd = format!("{:x}",md5::compute(pwd));
                                let role = 0;
                                let create_time = timestamp_to_date(get_current_timestamp());
                                let insert_key = "id,name,pwd,phone,create_time,organization,role".to_string();

                                let insert_values =
                                    format!("'{id}','{name}','{pwd}','{phone}','{create_time}','{org_id_str}','{role}'");
                                sql = format!("insert into public.user ({insert_key}) values ({insert_values})");
                                register_user(db,sql,id,name,org_id,org_c).await?
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
    name:String,
    org_id:Uuid,
    org_name:String
) -> DbQueryResult<RegisterResult> {
    dbg!(&sql);

    return match sqlx::query(&sql).fetch_one(&mut *db).await {
        Ok(_) => {
            dbg!("insert user success");
            Ok(RegisterResult::Success(SignData {
                name,
                id: id.to_string(),
                org_id: org_id.to_string(),
                org_name,
            }))
        },
        Err(err) => {
            if let SqlxError::RowNotFound = err {
                return Ok(RegisterResult::Success(SignData{
                    name,
                    id: id.to_string(),
                    org_id:org_id.to_string(),
                    org_name
                }))
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
        "select u.*,org.name org_name from public.user u left join public.organization org \
         on u.organization = org.id where pwd = '{pwd}' and {condition}"
    );
    
    dbg!(&sql);
    
    sqlx::query(&sql).fetch_one(&mut *db).await
}

pub async fn edit_user(
    user_data:User,
    mut db:GdDBC,
    validator:&State<ValidateData>
)-> DbQueryResult<SingleEditResult> {
    let id = Uuid::from_str(user_data.id.as_str()).unwrap();
    let (name,mut pwd,phone,gender,
        email,avatar,background,work_id)
        = user_data.into();


    let mut sql = "update public.user set ".to_string();

    if name.is_some() && validator.validate_name(&name.clone().unwrap()){
        sql += "name = $1,";
    }
    if pwd.is_some(){
        let pwd_md5 = format!("{:x}",md5::compute(pwd.clone().unwrap()));
        pwd.replace(pwd_md5);
        sql += "pwd = $2,";
    }
    if phone.is_some() && validator.validate_phone(&phone.clone().unwrap()){
        sql += "phone = $3,";
    }
    if gender.is_some(){sql += "gender = $4,"}
    if email.is_some() && validator.validate_email(&email.clone().unwrap()){
        sql += "email = $5,";
    }
    if avatar.is_some(){sql += "avatar = $6,"}
    if background.is_some(){sql += "background = $7,"}
    if work_id.is_some(){sql += "work_id = $8,"}

    let name = name.as_ref().map(String::as_str);
    let pwd = pwd.as_ref().map(String::as_str);
    let phone = phone.as_ref().map(String::as_str);
    let gender = gender.as_ref().map(String::as_str);
    let email = email.as_ref().map(String::as_str);
    let avatar = avatar.as_ref().map(String::as_str);
    let background = background.as_ref().map(String::as_str);
    let work_id = work_id.as_ref().map(String::as_str);

    if sql.ends_with(',') {sql = sql[..sql.len()-1].to_string()}
    sql = sql + " where id = $9";
    dbg!(&sql);
    return match sqlx::query(&sql).bind(name).bind(pwd).bind(phone).bind(gender)
        .bind(email).bind(avatar).bind(background).bind(work_id).bind(id).execute(&mut *db).await {
        Ok(_) => {
            Ok(SingleEditResult::Success("更新用户信息成功".to_string()))
        }
        Err(err) => {
            dbg!(&err);
            Err(err)
        }
    }
}

pub async fn select_partners(org_id:String,mut db:GdDBC) -> DbQueryResult<UserCollector> {
    let org_id = Uuid::from_str(org_id.as_str()).unwrap();
    let sql = "select * from public.user where organization = $1";
    let partners:Vec<_>;
    return match sqlx::query(sql).bind(org_id).fetch_all(&mut *db).await {
        Ok(v) => {
            partners = v.iter().map(|user| {
                UserGetter::from_row(user).unwrap()
            }).collect();

            Ok(UserCollector{
                collector: partners,
            })
        }
        Err(err) => {
            dbg!(&err);
            Err(err)
        }
    }
}

