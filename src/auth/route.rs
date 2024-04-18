use rocket::{post, State};
use rocket::form::Form;
use rocket::http::Status;
use uuid::Uuid;
use crate::auth::{AddUser, MoreUser, RegisterResult, RegisterUser};
use crate::auth::db_service::{ try_register_user};
use crate::auth::validate::{validate_register_data, ValidateData};
use crate::db::{DbQueryResult, GdDBC};
use crate::types::{RtData, RtStatus};

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