use chrono::NaiveDate;
use serde::Serialize;
use crate::model::task_model::TaskStatus;

#[derive(Serialize)]
pub struct TaskPreview {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub due_date: NaiveDate,
}