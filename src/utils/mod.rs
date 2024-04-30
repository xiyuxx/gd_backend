mod db_service;

use rocket::time::OffsetDateTime;
use rocket::http::Status;
use crate::db::{DbQueryResult, SqlxError};
use crate::types::{ObjectTypes, RtData, RtStatus, SingleEditResult};

pub fn timestamp_to_date(timestamp:u64) -> String{

    let date =
        OffsetDateTime::from_unix_timestamp(timestamp as i64).unwrap();
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        date.year(),date.month() as u32,date.day(),
        (date.hour()+8)%24,date.minute(),date.second()
    )
}


pub fn match_insert_res(res:DbQueryResult<SingleEditResult>, msg:String) -> Result<RtData<SingleEditResult>,Status>{
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


pub fn get_sequence_name(object_types: ObjectTypes, id: String) -> String{
    let mut seq = id.replace("-","_") + "_id_seq";
    match object_types {
        ObjectTypes::PROJECT => {
            seq.insert_str(0,"work_item_");
        }
        ObjectTypes::TEST => {
            seq.insert_str(0,"test_case_");
        }
        ObjectTypes::WIKI => {
            seq.insert_str(0,"wiki_doc_");
        }
        ObjectTypes::TOPIC => {
            seq.insert_str(0,"topic_");
        }
    }
    seq
}
