#[macro_use] extern crate rocket;



use gd_backend::auth::route::{add_user, create_user, edit, get_all_partners, login};
use gd_backend::config::{get_custom_figment, init_my_config};
use gd_backend::db::init_gd_data;
use gd_backend::project::route::{add_workmate_to_project, delete_project, get_participants, get_project, set_pro_star, set_project};
use gd_backend::project::work_item::route::{get_all_item, set_item};
use gd_backend::state::{get_default_user_token, init_validate_instance};
use gd_backend::test_hub::route::set_test_hub;
use gd_backend::topic::route::set_topic;
use gd_backend::wiki::route::{get_wiki, set_wiki, set_wiki_star};
use crate::catcher::{bad_request_catcher, error_catcher, not_found_catcher};

mod catcher;

#[launch]
fn rocket()-> _ {
    rocket::custom(get_custom_figment())
        .attach(init_my_config())
        .attach(init_gd_data())
        .attach(get_default_user_token())
        .manage(init_validate_instance().unwrap())
        .register("/",catchers![error_catcher,not_found_catcher,bad_request_catcher])
        .mount("/user",routes![add_user,create_user,login,edit,get_all_partners])
        .mount("/project",routes![set_project,get_project,delete_project,
            add_workmate_to_project,get_participants,set_pro_star])
        .mount("/item",routes![set_item,get_all_item])
        .mount("/test",routes![set_test_hub])
        .mount("/wiki",routes![set_wiki,get_wiki,set_wiki_star])
        .mount("/topic",routes![set_topic])

}
