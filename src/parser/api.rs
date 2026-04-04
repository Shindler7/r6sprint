//! Публичный API парсинга.

use crate::parser::{
    domain::{Announcements, AssetDsc, Backet, UserBacket, UserBackets, UserCash},
    logs::LogLine,
    traits::{Parsable, Parser},
};

// просто обёртки
// подсказка: почему бы не заменить на один дженерик?
/// Обёртка для парсинга [AssetDsc]
pub fn just_parse_asset_dsc(input: &str) -> Result<(&str, AssetDsc), ()> {
    <AssetDsc as Parsable>::parser().parse(input)
}

/// Обёртка для парсинга [Backet]
pub fn just_parse_backet(input: &str) -> Result<(&str, Backet), ()> {
    <Backet as Parsable>::parser().parse(input)
}

/// Обёртка для парсинга [UserCash]
pub fn just_user_cash(input: &str) -> Result<(&str, UserCash), ()> {
    <UserCash as Parsable>::parser().parse(input)
}

/// Обёртка для парсинга [UserBacket]
pub fn just_user_backet(input: &str) -> Result<(&str, UserBacket), ()> {
    <UserBacket as Parsable>::parser().parse(input)
}

/// Обёртка для парсинга [UserBackets]
pub fn just_user_backets(input: &str) -> Result<(&str, UserBackets), ()> {
    <UserBackets as Parsable>::parser().parse(input)
}

/// Обёртка для парсинга [Announcements]
pub fn just_parse_announcements(input: &str) -> Result<(&str, Announcements), ()> {
    <Announcements as Parsable>::parser().parse(input)
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
