//! Ошибки парсинга.

use thiserror::Error;

/// Доменные ошибки парсера.
#[derive(Error, Debug)]
pub enum ParsersError {
    /// Неопределённая ошибка.
    #[error("Unknown log data parsing error")]
    UnexpectedError,
}
