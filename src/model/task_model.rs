use chrono::NaiveDate;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use strum_macros::{Display};

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub due_date: NaiveDate,
}

#[derive(Serialize, Deserialize, Debug, Clone, Display)]
pub enum TaskStatus {
    ToDo,
    InProgress,
    Done,
}
