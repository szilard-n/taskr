use chrono::{Datelike, Local, NaiveDate};
use serde::Deserialize;
use validator::{Validate, ValidationError};

#[derive(Deserialize, Validate)]
pub struct CreateTask {
    #[validate(length(min = 1))]
    pub title: String,

    #[validate(length(min = 1))]
    pub description: String,

    #[validate(custom = "validate_date")]
    pub due_date: NaiveDate
}

pub fn validate_date(value: &NaiveDate) -> Result<(), ValidationError> {
    let today = NaiveDate::from_ymd_opt(Local::now().year(),
                                        Local::now().month(), Local::now().day()).unwrap();

    if *value >= today {
        Ok(())
    } else {
        Err(ValidationError::new("Date must not be in the past"))
    }
}