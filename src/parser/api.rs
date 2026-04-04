//! Публичный API парсинга.

use crate::parser::traits::{Parsable, Parser};

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
