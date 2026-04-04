//! Публичный API парсинга.

use crate::parser::{
    logs::LogLine,
    traits::{Parsable, Parser},
};

/// Парсит `input` в значение типа `T`.
///
/// Поддерживается любой `T`, реализующий [`Parsable`].
///
/// ## Ошибки
///
/// Возвращает `Err`, если парсер не смог разобрать `input`.
///
/// ## Пример
///
/// ```
/// use analysis::api;
/// use analysis::Announcements;
///
/// let demo = r#"[UserBackets{"user_id":"Bob","backets":[Backet{"asset_id":"milk","count":3,},],},]"#;
/// let result = api::just_parse::<Announcements>(&demo);
/// assert!(result.is_ok());
/// ```
pub fn just_parse<T: Parsable>(input: &str) -> Result<(&str, T), ()> {
    <T as Parsable>::parser().parse(input)
}

/// Парсер строки логов
pub struct LogLineParser {
    parser: std::sync::OnceLock<<LogLine as Parsable>::Parser>,
}

impl LogLineParser {
    pub fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, LogLine), ()> {
        self.parser
            .get_or_init(|| <LogLine as Parsable>::parser())
            .parse(input)
    }
}

// подсказка: singleton, без которого можно обойтись
// парсеры не страшно вытащить в pub
/// Единожды собранный парсер логов
pub static LOG_LINE_PARSER: LogLineParser = LogLineParser {
    parser: std::sync::OnceLock::new(),
};
