use std::borrow::Cow;

use chrono::{Local, NaiveDate};
use validator::ValidationError;

const VALID_PET_TYPES: [&str; 6] = ["bird", "cat", "dog", "hamster", "lizard", "snake"];

pub fn validate_not_blank(data: &str) -> Result<(), ValidationError> {
    if data.trim().is_empty() {
        return Err(create_validation_error("length", "required"));
    }

    Ok(())
}

pub fn validate_future_date(date: &str) -> Result<(), ValidationError> {
    validate_not_blank(date)?;

    let parsed_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| create_validation_error("invalid_date", "typeMismatch.birthDate"))?;

    let today = Local::now().date_naive();
    if parsed_date > today {
        return Err(create_validation_error(
            "future_date",
            "typeMismatch.birthDate",
        ));
    }

    Ok(())
}

pub fn validate_pet_type(data: &str) -> Result<(), ValidationError> {
    validate_not_blank(data)?;

    if !VALID_PET_TYPES.contains(&data) {
        return Err(create_validation_error(
            "invalid_pet_type",
            "존재하지 않는 pet type 입니다",
        ));
    }

    Ok(())
}

pub fn create_validation_error(code: &'static str, message: &'static str) -> ValidationError {
    ValidationError::new(code).with_message(Cow::Borrowed(message))
}
