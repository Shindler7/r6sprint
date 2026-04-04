//! Ошибки парсинга.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParsersError {
    #[error("Unknown log data parsing error")]
    UnexpectedError,
}
