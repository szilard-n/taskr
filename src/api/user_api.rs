use actix_web::{
    delete,
    HttpResponse,
    put,
    web::{Data, Json},
};
use actix_web::web::ReqData;

use crate::dto::update_user::UpdateUser;
use crate::model::user_model::User;
use crate::repository::task_repository::TaskRepository;
use crate::repository::user_repository::UserRepository;
use crate::validator::request_validators::validate_request_body;

#[put("/user")]
pub async fn update_user(db: Data<UserRepository>, logged_user_data: Option<ReqData<User>>,
                         body: Json<UpdateUser>) -> HttpResponse {

    let logged_user = match logged_user_data {
        Some(claims) => claims,
        _ => return HttpResponse::Unauthorized().finish()
    };

    let new_user = match validate_request_body(body).await {
        Ok(new_user) => new_user,
        Err(bad_request) => return bad_request
    };

    match db.update_user(logged_user.id.unwrap(), new_user).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string())
    }
}

#[delete("/user")]
pub async fn delete_user(user_db: Data<UserRepository>, task_db: Data<TaskRepository>,
                         logged_user_data: Option<ReqData<User>>) -> HttpResponse {

    let logged_user = match logged_user_data {
        Some(claims) => claims,
        _ => return HttpResponse::Unauthorized().finish()
    };

    let user = match user_db.delete_user(logged_user.id.unwrap()).await {
        Ok(user_option) => {
            match user_option {
                Some(user) => user,
                None => return HttpResponse::InternalServerError().json("User not found")
            }
        }
        Err(e) => return HttpResponse::InternalServerError().json(format!("Error while deleting the user: {}", e))
    };

    match task_db.delete_all_for_user(&user.id.unwrap()).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().json(format!("Error while deleting the user's tasks: {}", e))
    }
}


