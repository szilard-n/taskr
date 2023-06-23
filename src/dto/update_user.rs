use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct UpdateUser {
    #[validate(email)]
    pub email: String
}