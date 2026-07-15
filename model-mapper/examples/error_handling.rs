#![allow(dead_code)]

use std::convert::TryFrom;

use model_mapper::Mapper;

// The raw input DTO
pub struct RawUser {
    pub age: i64,
    pub status: String,
    pub email: String,
}

// Short-circuiting try_from conversion
#[derive(Debug, Mapper)]
#[mapper(try_from(err = AppError), ty = RawUser)]
pub struct UserShortCircuit {
    // Implicit conversion: maps TryFromIntError automatically using AppError's From implementation
    pub age: u8,
    // Error erasure: maps StatusError to AppError::InvalidStatus, discarding the source error
    #[mapper(err = AppError::InvalidStatus)]
    pub status: Status,
    // Error mapping: maps EmailError to AppError::InvalidEmail using a custom mapping closure
    #[mapper(err_with = |e: EmailError| AppError::InvalidEmail(e.to_string()))]
    pub email: Email,
}

// Accumulating try_from conversion
#[derive(Debug, Mapper)]
#[mapper(try_from(err = AppError, accumulate), ty = RawUser)]
pub struct UserAccumulated {
    pub age: u8,
    #[mapper(err = AppError::InvalidStatus)]
    pub status: Status,
    #[mapper(err_with = |e: EmailError| AppError::InvalidEmail(e.to_string()))]
    pub email: Email,
}

// Custom error enum styled with thiserror
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AppError {
    #[error("Invalid age format: {0}")]
    InvalidAge(#[from] std::num::TryFromIntError),

    #[error("Invalid email format: {0}")]
    InvalidEmail(String),

    #[error("Invalid status value")]
    InvalidStatus,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Email(String);

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
#[error("email must contain @")]
pub struct EmailError;

impl TryFrom<String> for Email {
    type Error = EmailError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.contains('@') {
            Ok(Email(value))
        } else {
            Err(EmailError)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Active,
    Inactive,
}

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
#[error("unknown status")]
pub struct StatusError;

impl TryFrom<String> for Status {
    type Error = StatusError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "active" => Ok(Status::Active),
            "inactive" => Ok(Status::Inactive),
            _ => Err(StatusError),
        }
    }
}

fn main() {
    let raw_invalid = RawUser {
        age: -5,
        status: "invalid_status".to_string(),
        email: "invalid_email".to_string(),
    };

    // Under short-circuiting error handling, the first conversion error is returned immediately
    let result = UserShortCircuit::try_from(raw_invalid);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::InvalidAge(_)));

    let raw_all_invalid = RawUser {
        age: -5,
        status: "invalid_status".to_string(),
        email: "invalid_email".to_string(),
    };

    // Under error accumulation, all field conversion errors are collected into a Vec
    let result_accum = UserAccumulated::try_from(raw_all_invalid);
    assert!(result_accum.is_err());
    let errors = result_accum.unwrap_err();
    assert_eq!(errors.len(), 3);
    assert!(matches!(errors[0], AppError::InvalidAge(_)));
    assert_eq!(errors[1], AppError::InvalidStatus);
    assert_eq!(errors[2], AppError::InvalidEmail("email must contain @".to_string()));

    // When all fields convert successfully, the target struct is returned
    let raw_happy = RawUser {
        age: 42,
        status: "active".to_string(),
        email: "bob@example.com".to_string(),
    };

    let happy_user = UserAccumulated::try_from(raw_happy).unwrap();
    assert_eq!(happy_user.age, 42);
    assert_eq!(happy_user.status, Status::Active);
    assert_eq!(happy_user.email, Email("bob@example.com".to_string()));
}
