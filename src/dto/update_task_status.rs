use serde::Deserialize;
use validator::Validate;
use crate::model::task_model::TaskStatus;

#[derive(Deserialize, Validate)]
pub struct UpdateTaskStatus {
    pub new_status: TaskStatus
}