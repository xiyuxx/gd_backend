use rocket::http::Status;
use crate::db::{DbQueryResult, SqlxError};
use crate::types::{SingleEditResult, RtData, RtStatus};

mod db_service;
pub mod route;
mod types;
pub mod work_item;

fn match_insert_res(res:DbQueryResult<SingleEditResult>, msg:String) -> Result<RtData<SingleEditResult>,Status>{
    match res{
        Ok(adjust_res) => {
            let success = true;
            let status = RtStatus::Success;
            let msg = String::from("");

            Ok(RtData{
                success,status,msg,data:adjust_res
            })
        }
        Err(err) => {
            match err {
                SqlxError::RowNotFound => {
                    Ok(RtData{
                        success:true,
                        status:RtStatus::Success,
                        msg:String::from(""),
                        data: SingleEditResult::Success(msg),
                    })
                }
                _ => {
                    dbg!(err);
                    Err(Status::InternalServerError)
                }
            }
        }
    }
}



