use actix_web::{App, HttpServer, web};
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web_httpauth::middleware::HttpAuthentication;
use dotenv::dotenv;

use repository::user_repository::UserRepository;

use crate::api::auth_api::{sign_in, sign_up};
use crate::api::task_api::{create_task, delete_task, get_all_tasks_for_user, get_task, update_task_status};
use crate::api::user_api::{delete_user, update_user};
use crate::repository::task_repository::TaskRepository;
use crate::service::email_service::morning_email_scheduler;
use crate::validator::request_validators::jwt_validator;

mod api;
mod model;
mod repository;
mod dto;
mod validator;
mod service;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let user_repo = UserRepository::init().await;
    let user_data = Data::new(user_repo);

    let task_repo = TaskRepository::init().await;
    let task_data = Data::new(task_repo);

    // start scheduler on a different thread
    tokio::spawn(morning_email_scheduler(user_data.clone(), task_data.clone()));

    HttpServer::new(move || {
        let bearer_middleware = HttpAuthentication::bearer(jwt_validator);
        App::new()
            .wrap(Logger::default())
            .app_data(user_data.clone())
            .app_data(task_data.clone())
            .service(sign_up)
            .service(sign_in)
            .service(
                web::scope("")
                    .wrap(bearer_middleware)
                    .service(update_user)
                    .service(delete_user)
                    .service(create_task)
                    .service(get_task)
                    .service(get_all_tasks_for_user)
                    .service(delete_task)
                    .service(update_task_status)
            )
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
