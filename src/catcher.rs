use std::path::Path;
use rocket::{
    catch
};
use rocket::fs::NamedFile;
use serde_json::{json, Value};
use gd_backend::types::{DefaultData, RtData, RtStatus};
use gd_backend::types::DefaultData::Failure;

#[catch(400)]
pub fn bad_request_catcher() -> Option<RtData<DefaultData>> {
    Some( RtData {
        data: Failure(()),
        msg: String::from("get wrong params"),
        success: false,
        status: RtStatus::Fail,
    })
}

#[catch(404)]
pub async fn not_found_catcher() -> Option<NamedFile> {
    NamedFile::open(Path::new("static/404.html")).await.ok()
}

#[catch(500)]
pub fn error_catcher() -> Option<Value> {
    Some(json!({
        "success":false,
        "status": -2,
        "msg" : String::from("server internal error"),
        "data" : Failure(())
    }))
}