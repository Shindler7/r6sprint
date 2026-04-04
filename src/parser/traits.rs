//! Общие трейты для парсеров.

/// Трейт, чтобы **реализовывать** и **требовать** метод 'распарсь и покажи,
/// что распарсить осталось'.
pub trait Parser {
    type Dest;

    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()>;
}

/// Вспомогательный трейт, чтобы писать собственный десериализатор
/// (по решаемой задаче — отдалённый аналог `serde::Deserialize`).
pub trait Parsable: Sized {
    type Parser: Parser<Dest = Self>;

    fn parser() -> Self::Parser;
}
