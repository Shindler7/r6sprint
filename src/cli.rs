//! Инфраструктура поддержки обработки параметров командной строки.
//!
//! Создано при помощи легковесного `Argh`.

use anyhow::{Context, Result as AnyhowResult};
use argh::FromArgs;
use std::path::PathBuf;

/// Архитектура параметров командной строки.
#[derive(FromArgs)]
pub(crate) struct GoCliArgs {
    /// log-file name.
    #[argh(positional)]
    log_filename: Option<PathBuf>,
}

impl GoCliArgs {
    /// Собрать данные аргументов командной строки.
    pub(crate) fn new() -> Self {
        argh::from_env()
    }

    /// Предоставить имя лог-файла.
    pub(crate) fn filename(&self) -> &Option<PathBuf> {
        &self.log_filename
    }

    /// Предоставить полный путь к log-файлу.
    ///
    /// Соединяет переданное имя файла из командной строки с `current_dir`.
    pub(crate) fn path_to_log_file(&self) -> AnyhowResult<Option<PathBuf>> {
        if let Some(log_file) = &self.filename() {
            Ok(Some(Self::get_current_dir()?.join(log_file)))
        } else {
            Ok(None)
        }
    }

    /// Предоставить текущую директорию.
    pub(crate) fn get_current_dir() -> AnyhowResult<PathBuf> {
        std::env::current_dir().with_context(|| "Failed to get current directory")
    }
}
