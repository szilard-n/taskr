extern crate dotenv;

use std::env;
use futures::TryStreamExt;

use mongodb::{
    bson::doc,
    Client,
    Collection, results::InsertOneResult,
};
use mongodb::bson::oid::ObjectId;
use mongodb::error::Error as MongoError;
use mongodb::results::UpdateResult;

use crate::dto::update_user::UpdateUser;
use crate::model::user_model::User;

pub struct UserRepository {
    col: Collection<User>,
}

impl UserRepository {
    pub async fn init() -> Self {
        let uri = match env::var("MONGO_URI") {
            Ok(variable) => variable.to_string(),
            Err(_) => format!("Error loading env variable"),
        };

        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database("rust-actix");
        let col: Collection<User> = db.collection("User");
        UserRepository { col }
    }

    pub async fn create_user(&self, email: String, password: String) -> Result<InsertOneResult, MongoError> {
        let new_doc = User {
            id: None,
            email,
            password,
        };
        let result = self.col.insert_one(new_doc, None).await?;

        Ok(result)
    }

    pub async fn find_by_email(&self, email: &String) -> Result<Option<User>, MongoError> {
        let filter = doc! { "email": email };
        match self.col.find_one(filter, None).await {
            Ok(user) => Ok(user),
            Err(e) => Err(e)
        }
    }

    pub async fn update_user(&self, id: ObjectId, new_user: UpdateUser) -> Result<UpdateResult, MongoError> {
        let new_doc = doc! {
            "$set": {
                "email": new_user.email
            }
        };
        let filter = doc! { "_id": id };
        let result = self.col.update_one(filter, new_doc, None).await?;

        Ok(result)
    }

    pub async fn delete_user(&self, id: ObjectId) -> Result<Option<User>, MongoError> {
        let filter = doc! { "_id": id };
        match self.col.find_one_and_delete(filter, None).await {
            Ok(result) => Ok(result),
            Err(e) => Err(e)
        }
    }

    pub async fn find_all(&self) -> Result<Vec<User>, MongoError> {
        let cursor = self.col.find(None, None).await?;
        let users: Vec<User> = cursor.try_collect().await.map_err(MongoError::from)?;

        Ok(users)
    }
}
