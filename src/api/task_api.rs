use std::error::Error;

use actix_web::{delete, get, HttpResponse, post, put};
use actix_web::web::{Data, Json, Path, ReqData};

use crate::dto::create_task::CreateTask;
use crate::dto::task_preview::TaskPreview;
use crate::dto::update_task_status::UpdateTaskStatus;
use crate::model::user_model::User;
use crate::repository::task_repository::TaskRepository;
use crate::repository::user_repository::UserRepository;
use crate::validator::request_validators::validate_request_body;

#[get("/task/{id}")]
pub async fn get_task(task_repo: Data<TaskRepository>, logged_user_data: Option<ReqData<User>>,
                      task_id: Path<String>) -> HttpResponse {
    let logged_user = match logged_user_data {
        Some(claims) => claims,
        _ => return HttpResponse::Unauthorized().finish()
    };

    match task_repo.find_by_id(&task_id, &logged_user.id.unwrap()).await {
        Ok(task_option) => match task_option {
            Some(task) => HttpResponse::Ok().json(TaskPreview {
                id: task.id.unwrap().to_string(),
                title: task.title,
                description: task.description,
                status: task.status,
                due_date: task.due_date,
            }),
            None => HttpResponse::NotFound().json("Task not found")
        }
        Err(e) => {
            let error_message = e.source().unwrap_or(&e);
            HttpResponse::InternalServerError().body(error_message.to_string())
        }
    }
}

#[get("/task")]
pub async fn get_all_tasks_for_user(task_repo: Data<TaskRepository>,
                                    logged_user_data: Option<ReqData<User>>) -> HttpResponse {
    let logged_user = match logged_user_data {
        Some(claims) => claims,
        _ => return HttpResponse::Unauthorized().finish()
    };

    match task_repo.find_all_for_user(&logged_user.id.unwrap()).await {
        Ok(tasks) => {
            let tasks_review: Vec<TaskPreview> = tasks.into_iter()
                .map(|task| TaskPreview {
                    id: task.id.unwrap().to_string(),
                    title: task.title,
                    description: task.description,
                    status: task.status,
                    due_date: task.due_date,
                }).collect();

            HttpResponse::Ok().json(tasks_review)
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string())
    }
}

#[post("/task")]
pub async fn create_task(task_repo: Data<TaskRepository>, user_repo: Data<UserRepository>,
                         logged_user_data: Option<ReqData<User>>, body: Json<CreateTask>) -> HttpResponse {
    let logged_user = match logged_user_data {
        Some(claims) => claims,
        _ => return HttpResponse::Unauthorized().finish()
    };

    let new_task = match validate_request_body(body).await {
        Ok(new_task) => new_task,
        Err(bad_request) => return bad_request,
    };

    let user = match user_repo.find_by_email(&logged_user.email).await {
        Ok(user_option) => {
            match user_option {
                Some(user) => user,
                None => return HttpResponse::InternalServerError().json("User not found")
            }
        }
        Err(e) => return HttpResponse::InternalServerError().json(format!("Something went wrong while creating task: {}", e)),
    };

    let tasks_result = match task_repo.create_task(&new_task, &user.id.unwrap()).await {
        Ok(tasks) => tasks,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };

    let tasks: Vec<TaskPreview> = tasks_result.into_iter()
        .map(|t| TaskPreview {
            id: t.id.unwrap().to_string(),
            title: t.title.to_string(),
            description: t.description.to_string(),
            due_date: t.due_date,
            status: t.status.clone(),
        })
        .collect();

    HttpResponse::Created().json(tasks)
}

#[delete("/task/{id}")]
pub async fn delete_task(task_repo: Data<TaskRepository>,
                         logged_user_data: Option<ReqData<User>>, task_id: Path<String>) -> HttpResponse {
    let logged_user = match logged_user_data {
        Some(claims) => claims,
        _ => return HttpResponse::Unauthorized().finish()
    };

    let tasks_result = match task_repo.delete_by_id(&task_id, &logged_user.id.unwrap()).await {
        Ok(Some(tasks)) => tasks,
        Ok(None) => return HttpResponse::NotFound().json("Task not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };

    let tasks: Vec<TaskPreview> = tasks_result.into_iter()
        .map(|t| TaskPreview {
            id: t.id.unwrap().to_string(),
            title: t.title.to_string(),
            description: t.description.to_string(),
            due_date: t.due_date,
            status: t.status.clone(),
        })
        .collect();

    HttpResponse::Ok().json(tasks)
}

#[put("/task/{task_id}")]
pub async fn update_task_status(task_repo: Data<TaskRepository>,
                                logged_user_data: Option<ReqData<User>>,
                                task_id: Path<String>,
                                new_task: Json<UpdateTaskStatus>) -> HttpResponse {

    let logged_user = match logged_user_data {
        Some(claims) => claims,
        _ => return HttpResponse::Unauthorized().finish()
    };

    let tasks_result = match task_repo.update_status(&task_id,
                                                &logged_user.id.unwrap(), &new_task.new_status).await {
        Ok(Some(tasks)) => tasks,
        Ok(None) => return HttpResponse::NotFound().json("Task not found"),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };

    let tasks: Vec<TaskPreview> = tasks_result.into_iter()
        .map(|t| TaskPreview {
            id: t.id.unwrap().to_string(),
            title: t.title.to_string(),
            description: t.description.to_string(),
            due_date: t.due_date,
            status: t.status.clone(),
        })
        .collect();

    HttpResponse::Ok().json(tasks)
}