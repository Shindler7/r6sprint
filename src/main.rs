//! Парсинг лог-файлов.

mod cli;
mod parse;

use crate::cli::GoCliArgs;
use analysis::{MyReader, ReadModeLog};
use anyhow::{Result as AnyhowResult, anyhow};

fn main() -> AnyhowResult<()> {
    println!("Placeholder для экспериментов с cli");

    parsing_demo()?;
    parse_log_file()?;

    Ok(())
}

/// Функция-демонстратор работы парсинга.
fn parsing_demo() -> AnyhowResult<()> {
    let parsing_demo =
        r#"[UserBackets{"user_id":"Bob","backets":[Backet{"asset_id":"milk","count":3,},],},]"#
            .to_string();

    let announcements = parse::just_parse_anouncements(parsing_demo)
        .map_err(|_| anyhow!("An error occurred while processing log data in the demo mode."))?;

    println!("demo-parsed: {:?}", announcements);

    Ok(())
}

/// Парсинг log-файла, переданного через командную строку.
fn parse_log_file() -> AnyhowResult<()> {
    let log_file = GoCliArgs::new().path_to_log_file()?;

    println!("Trying opening file '{}'", log_file.to_string_lossy());
    let file = std::fs::File::open(&log_file)?;
    println!("Successfully opened file.");

    let file: std::rc::Rc<std::cell::RefCell<Box<dyn MyReader>>> =
        std::rc::Rc::new(std::cell::RefCell::new(Box::new(file)));

    let logs = analysis::read_log(file.clone(), ReadModeLog::All, vec![]);

    println!("got logs:");
    logs.iter().for_each(|parsed| println!("  {:?}", parsed));

    Ok(())
}
