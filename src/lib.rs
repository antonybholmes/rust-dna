use std::fmt::{Display, Formatter, Result};

#[derive(Debug)] // derive std::fmt::Debug on AppError
pub struct AppError {
    message: String,
}

impl AppError {
    pub fn new(message: String) -> AppError {
        AppError { message }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.message)
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        AppError {
            message: error.to_string(),
        }
    }
}

impl From<String> for AppError {
    fn from(error: String) -> Self {
        AppError {
            message: error.to_string(),
        }
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;
