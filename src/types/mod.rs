use std::io::Cursor;
use rocket::{Request, Response};
use rocket::http::ContentType;
use rocket::response::Responder;
use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize,Debug,Eq, PartialEq,Clone)]
pub struct RtData<T> {
    pub data: T,
    pub msg: String,
    pub success: bool,
    pub status: RtStatus
}

impl<T:Serialize> RtData<T>{
    pub fn to_string(mut self) -> String{
        serde_json::to_string(&mut self).unwrap()
    }
}

impl<'r> Responder<'r,'static> for RtData<DefaultData> {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let data = self.to_string();

        Response::build()
            .header(ContentType::JSON)
            .sized_body(data.len(),Cursor::new(data))
            .ok()
    }
}

#[derive(Debug,Serialize,Deserialize,Eq, PartialEq,Copy, Clone)]
pub enum RtStatus {
    AuthFail,
    AuthSuccess,
    Success,
    Fail,
    Error,
}


#[derive(Debug,Serialize,Deserialize,Eq, PartialEq, Copy, Clone)]
pub enum DefaultData{
    Success(()),
    Failure(())
}
