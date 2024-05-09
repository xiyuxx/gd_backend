use rocket::FromForm;

use serde::{Deserialize, Serialize};

#[derive(Debug,Serialize,Deserialize,FromForm,Eq, PartialEq)]
pub struct WikiStar{
    #[field(name="userId")]
    pub user_id:String,
    #[field(name="id")]
    pub wiki_id:String,
    #[field(name="star")]
    pub star:bool
}