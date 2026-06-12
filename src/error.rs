use std::fmt;
use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;

#[derive(Debug)]
pub enum AppError {
    Usage(String),
    MissingField(String),
    UnexpectedField(String),
    InvalidHeader(String),
    InvalidId(ParseIntError),
    InvalidActive(ParseBoolError),
    InvalidAmount(ParseFloatError),
    IoError(std::io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::Usage(msg) => write!(f, "Usage error: {}", msg),
            AppError::MissingField(field) => write!(f, "Format error: Missing field: {}", field),
            AppError::UnexpectedField(line) => {
                write!(f, "Format error: Unexpected extra field in line: {}", line)
            }
            AppError::InvalidHeader(header) => {
                write!(f, "Format error: Invalid header: {}", header)
            }
            AppError::InvalidId(e) => write!(f, "ID error: {}", e),
            AppError::InvalidActive(e) => write!(f, "Active state error: {}", e),
            AppError::InvalidAmount(e) => write!(f, "Amount error: {}", e),
            AppError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for AppError {}

impl From<ParseIntError> for AppError {
    fn from(err: ParseIntError) -> Self {
        AppError::InvalidId(err)
    }
}

impl From<ParseFloatError> for AppError {
    fn from(err: ParseFloatError) -> Self {
        AppError::InvalidAmount(err)
    }
}

impl From<ParseBoolError> for AppError {
    fn from(err: ParseBoolError) -> Self {
        AppError::InvalidActive(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err)
    }
}
