use std::env;
use std::str::FromStr;

use chrono::NaiveDate;
use futures::TryStreamExt;
use mongodb::{Client, Collection};
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::error::Error as MongoError;
use mongodb::results::DeleteResult;

use crate::dto::create_task::CreateTask;
use crate::model::task_model::{Task, TaskStatus};

pub struct TaskRepository {
    col: Collection<Task>,
}

impl TaskRepository {
    pub async fn init() -> Self {
        let uri = match env::var("MONGO_URI") {
            Ok(variable) => variable.to_string(),
            Err(_) => format!("Error loading env variable"),
        };

        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database("rust-actix");
        let col: Collection<Task> = db.collection("Task");
        TaskRepository { col }
    }

    pub async fn create_task(&self, new_task: &CreateTask, user_id: &ObjectId) -> Result<Vec<Task>, MongoError> {
        let new_doc = Task {
            id: None,
            user_id: user_id.clone(),
            title: new_task.title.clone(),
            description: new_task.description.to_string(),
            status: TaskStatus::ToDo,
            due_date: new_task.due_date,
        };
        match self.col.insert_one(new_doc, None).await {
            Ok(_) => match self.find_all_for_user(user_id).await {
                Ok(tasks) => Ok(tasks),
                Err(e) => Err(e)
            },
            Err(e) => Err(e)
        }
    }

    pub async fn find_by_id(&self, task_id: &String, user_id: &ObjectId) -> Result<Option<Task>, MongoError> {
        let task_object_id = match ObjectId::from_str(task_id) {
            Ok(id) => id,
            Err(e) => return Err(MongoError::custom(format!("Error parsing ObjectId: {}", e)))
        };

        let filter = doc! {
            "_id": task_object_id,
            "user_id": user_id
        };

        match self.col.find_one(filter, None).await {
            Ok(task) => Ok(task),
            Err(e) => Err(e)
        }
    }

    pub async fn find_all_for_user(&self, user_id: &ObjectId) -> Result<Vec<Task>, MongoError> {
        let filter = doc! { "user_id": user_id };
        let cursor = self.col.find(filter, None).await?;
        let tasks: Vec<Task> = cursor.try_collect().await.map_err(MongoError::from)?;

        Ok(tasks)
    }

    pub async fn delete_all_for_user(&self, user_id: &ObjectId) -> Result<DeleteResult, MongoError> {
        let filter = doc! { "user_id": user_id };
        match self.col.delete_many(filter, None).await {
            Ok(result) => Ok(result),
            Err(e) => Err(e)
        }
    }

    pub async fn delete_by_id(&self, task_id: &String, user_id: &ObjectId) -> Result<Option<Vec<Task>>, MongoError> {
        let task_object_id = match ObjectId::from_str(task_id) {
            Ok(id) => id,
            Err(e) => return Err(MongoError::custom(format!("Error parsing ObjectId: {}", e)))
        };

        let filter = doc! {
            "_id": task_object_id,
            "user_id": user_id
        };

        let delete_result = match self.col.delete_one(filter, None).await {
            Ok(result) => result,
            Err(e) => return Err(e)
        };

        if delete_result.deleted_count == 0 {
            return Ok(None);
        }

        match self.find_all_for_user(user_id).await {
            Ok(tasks) => Ok(Some(tasks)),
            Err(e) => Err(e)
        }
    }

    pub async fn update_status(&self, task_id: &String, user_id: &ObjectId,
                               new_status: &TaskStatus) -> Result<Option<Vec<Task>>, MongoError> {
        let task_object_id = match ObjectId::from_str(task_id) {
            Ok(id) => id,
            Err(e) => return Err(MongoError::custom(format!("Error parsing ObjectId: {}", e)))
        };

        let filter = doc! {
            "_id": task_object_id,
            "user_id": user_id
        };

        let new_doc = doc! {
            "$set": {
                "status": new_status.to_string()
            }
        };

        let update_result = match self.col.update_one(filter, new_doc, None).await {
            Ok(result) => result,
            Err(e) => return Err(e)
        };

        if update_result.modified_count == 0 {
            return Ok(None);
        }

        match self.find_all_for_user(user_id).await {
            Ok(tasks) => Ok(Some(tasks)),
            Err(e) => Err(e)
        }
    }

    pub async fn find_by_due_date(&self, user_id: &ObjectId, due_date: NaiveDate) -> Result<Vec<Task>, MongoError> {
        let filter = doc! {
            "user_id": user_id,
            "due_date": due_date.to_string(),
            "status": {
                "$ne": TaskStatus::Done.to_string()
            }
        };
        let cursor = self.col.find(filter, None).await?;
        let tasks: Vec<Task> = cursor.try_collect().await.map_err(MongoError::from)?;

        Ok(tasks)
    }
}
