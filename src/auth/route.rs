use std::time::SystemTime;
use chrono::{DateTime, NaiveDateTime, Utc};
use rocket::{post, State};
use rocket::form::Form;
use rocket::http::Status;
use sqlx::{FromRow, Row};
use uuid::{Uuid};
use crate::auth::{AddUser, LoginData, MoreUser, RegisterResult, RegisterUser};
use crate::auth::db_service::{get_user_msg, try_register_user};
use crate::auth::validate::{validate_login_data, validate_register_data, ValidateData};
use crate::db::{DbQueryResult, GdDBC, SqlxError};
use crate::types::{LoginSuccessData, RtData, RtStatus};

#[post("/register/add",data="<add_data>")]
pub async fn add_user(
    db:GdDBC,
    validator:&State<ValidateData>,
    add_data: Form<AddUser>
) -> Result<RtData<RegisterResult>,Status> {
    validate_register_data(add_data.clone().into(), &validator)?;

    let res =
        try_register_user(db,MoreUser::Add(add_data.into_inner())).await;
    handle_register_res(res)
}

#[post("/register/create",data="<register_data>")]
pub async fn create_user(
    mut db:GdDBC,
    validator:&State<ValidateData>,
    mut register_data: Form<RegisterUser>
) -> Result<RtData<RegisterResult>,Status> {
    dbg!("注册相应");
    let user_c = register_data.clone();
    let org = user_c.organization;
    let (name,pwd,email,phone) =
        (user_c.name,user_c.pwd,"".to_string(),user_c.phone);
    validate_register_data((name,pwd,email,phone),&validator)?;

    

    let org_id = Uuid::new_v4();
    dbg!("添加组织！！");
    return match sqlx::query(
        "insert into public.organization values($1,$2)"
    ).bind(org_id).bind(org).execute(&mut *db).await {
        Ok(_) => {
            dbg!("组织添加成功");
            register_data.organization = org_id.to_string();
            let res =
                try_register_user(db,MoreUser::Create(register_data.into_inner())).await;
            
            handle_register_res(res)
        }
        Err(_) => Err(Status::Conflict)
    };

}

fn handle_register_res(res:DbQueryResult<RegisterResult>) -> Result<RtData<RegisterResult>,Status>{
    match res {
        Ok(register_res) => {
            let mut success = true;
            let mut status = RtStatus::Success;
            let mut msg = String::from("register success");

            let data = match register_res {
                RegisterResult::Exist(r_msg) => {
                    success = false;
                    status = RtStatus::Fail;
                    msg = r_msg;
                    RegisterResult::Exist("user already exist".to_string())
                },
                RegisterResult::Success(user_data) => {
                    RegisterResult::Success(user_data)
                }
            };

            Ok(RtData{
                success,msg,status,data
            })
        }
        Err(err) => {
            dbg!(err);
            return Err(Status::InternalServerError);
        }
    }
}

#[post("/login",data="<login_data>")]
pub async fn login(
    db:GdDBC,
    validator:&State<ValidateData>,
    mut login_data: Form<LoginData>
) -> Result<RtData<LoginSuccessData>,Status> {
    let user_login_key = login_data.login_key.to_owned();
    let pwd = login_data.pwd.to_owned();

    let is_email = validate_login_data(&mut login_data,&validator)?;
    let res = get_user_msg((user_login_key,pwd,is_email),db).await;

    let user_msg = match res {
        Ok(row) => {

            LoginSuccessData::from_row(&row)
        }
        Err(err) => {
            return match err {
                SqlxError::RowNotFound => {
                    dbg!("row not found");
                    Err(Status::BadRequest)
                }
                _ => {
                    let db_err = err.into_database_error().expect("is not db err");
                    dbg!(db_err.message());
                    Err(Status::InternalServerError)
                }

            };
        }
    };
    let user_msg = match user_msg {
        Ok(data) => data,
        Err(e) => {
            dbg!("conversion from pgRow to struct error",e);
            return Err(Status::InternalServerError);
        }
    };

    Ok(RtData{
        success: true,
        msg: String::from("login success"),
        status: RtStatus::Success,
        data:user_msg,
    })
}