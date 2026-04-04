//! Инструментарий парсинга лог-файлов.

use crate::parser::traits::Parser;

/// Обернуть строку в кавычки, экранировав кавычки, которые в строке уже есть.
#[cfg(debug_assertions)]
#[allow(dead_code)]
pub(crate) fn quote(input: &str) -> String {
    let mut result = String::from("\"");
    result.extend(input.chars().flat_map(|c| match c {
        '\\' | '"' => ['\\', c].into_iter().take(2),
        _ => [c, ' '].into_iter().take(1),
    }));
    result.push('"');
    result
}

/// Распарсить строку, которую ранее [обернули в кавычки](quote).
// `"abc\"def\\ghi"nice` -> (`abcd"def\ghi`, `nice`)
pub(crate) fn do_unquote(input: &str) -> Result<(&str, String), ()> {
    let mut out = String::with_capacity(input.len().saturating_sub(2));
    let mut escaped = false;
    let mut chars = input.strip_prefix('"').ok_or(())?.chars();

    for c in chars.by_ref() {
        if escaped {
            out.push(c);
            escaped = false;
            continue;
        }

        match c {
            '\\' => escaped = true,
            '"' => return Ok((chars.as_str(), out)),
            _ => out.push(c),
        }
    }

    Err(()) // строка кончилась, не закрыв кавычку
}

/// Распарсить строку, обёрнутую в кавычки.
///
/// Сокращённая реализация [`do_unquote`], в которой вложенные кавычки
/// не предусмотрены.
pub(crate) fn do_unquote_non_escaped(input: &str) -> Result<(&str, &str), ()> {
    let input = input.strip_prefix("\"").ok_or(())?;
    let quote_byteidx = input.find('"').ok_or(())?;
    if 0 == quote_byteidx || Some("\\") == input.get(quote_byteidx - 1..quote_byteidx) {
        return Err(());
    }
    Ok((&input[1 + quote_byteidx..], &input[..quote_byteidx]))
}

/// Парсер кавычек.
#[derive(Debug, Clone)]
pub struct Unquote;

impl Parser for Unquote {
    type Dest = String;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        do_unquote(input)
    }
}

/// Парсер константных строк
///
/// (аналог `nom::bytes::complete::tag`).
#[derive(Debug, Clone)]
pub struct Tag {
    pub(crate) tag: &'static str,
}

impl Parser for Tag {
    type Dest = ();
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        Ok((input.strip_prefix(self.tag).ok_or(())?, ()))
    }
}

/// Парсер [тэга](Tag), обёрнутого в кавычки.
#[derive(Debug, Clone)]
pub struct QuotedTag(Tag);

impl QuotedTag {
    pub(crate) fn new(tag: &'static str) -> Self {
        Self(Tag { tag })
    }
}

impl Parser for QuotedTag {
    type Dest = ();
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let (remaining, candidate) = do_unquote_non_escaped(input)?;
        if !self.0.parse(candidate)?.0.is_empty() {
            return Err(());
        }
        Ok((remaining, ()))
    }
}

/// Комбинатор, пробрасывающий строку без лидирующих пробелов
#[derive(Debug, Clone)]
pub struct StripWhitespace<T> {
    pub(crate) parser: T,
}
impl<T: Parser> Parser for StripWhitespace<T> {
    type Dest = T::Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        self.parser
            .parse(input.trim_start())
            .map(|(remaining, parsed)| (remaining.trim_start(), parsed))
    }
}

/// Комбинатор, чтобы распарсить нужное, окружённое в начале и в конце чем-то
/// обязательным, не участвующем в результате.
/// Пробрасывает строку в парсер1, оставшуюся строку после первого
/// парсинга - в парсер2, оставшуюся строку после второго парсинга - в парсер3.
/// Результат парсера2 будет результатом этого комбинатора, а оставшейся
/// строкой - строка, оставшаяся после парсера3.
/// (аналог `delimited` из `nom`)
#[derive(Debug, Clone)]
pub struct Delimited<Prefix, T, Suffix> {
    pub(crate) prefix_to_ignore: Prefix,
    pub(crate) dest_parser: T,
    pub(crate) suffix_to_ignore: Suffix,
}
impl<Prefix, T, Suffix> Parser for Delimited<Prefix, T, Suffix>
where
    Prefix: Parser,
    T: Parser,
    Suffix: Parser,
{
    type Dest = T::Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let (remaining, _) = self.prefix_to_ignore.parse(input)?;
        let (remaining, result) = self.dest_parser.parse(remaining)?;
        self.suffix_to_ignore
            .parse(remaining)
            .map(|(remaining, _)| (remaining, result))
    }
}

/// Комбинатор-отображение. Парсит дочерним парсером, преобразует результат так,
/// как вызывающему хочется
#[derive(Debug, Clone)]
pub struct Map<T, M> {
    pub(crate) parser: T,
    pub(crate) map: M,
}
impl<T: Parser, Dest: Sized, M: Fn(T::Dest) -> Dest> Parser for Map<T, M> {
    type Dest = Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        self.parser
            .parse(input)
            .map(|(remaining, pre_result)| (remaining, (self.map)(pre_result)))
    }
}

/// Комбинатор с отбрасываемым префиксом, упрощённая версия [Delimited]
/// (аналог `preceeded` из `nom`).
#[derive(Debug, Clone)]
pub struct Preceded<Prefix, T> {
    pub(crate) prefix_to_ignore: Prefix,
    pub(crate) dest_parser: T,
}

impl<Prefix, T> Parser for Preceded<Prefix, T>
where
    Prefix: Parser,
    T: Parser,
{
    type Dest = T::Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let (remaining, _) = self.prefix_to_ignore.parse(input)?;
        self.dest_parser.parse(remaining)
    }
}

/// Комбинатор, который требует, чтобы все дочерние парсеры отработали,
/// (аналог `all` из `nom`).
#[derive(Debug, Clone)]
pub struct All<T> {
    pub(crate) parser: T,
}

impl<A0, A1> Parser for All<(A0, A1)>
where
    A0: Parser,
    A1: Parser,
{
    type Dest = (A0::Dest, A1::Dest);

    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let (remaining, a0) = self.parser.0.parse(input)?;
        self.parser
            .1
            .parse(remaining)
            .map(move |(remaining, a1)| (remaining, (a0, a1)))
    }
}

impl<A0, A1, A2> Parser for All<(A0, A1, A2)>
where
    A0: Parser,
    A1: Parser,
    A2: Parser,
{
    type Dest = (A0::Dest, A1::Dest, A2::Dest);
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let (remaining, a0) = self.parser.0.parse(input)?;
        let (remaining, a1) = self.parser.1.parse(remaining)?;
        self.parser
            .2
            .parse(remaining)
            .map(move |(remaining, a2)| (remaining, (a0, a1, a2)))
    }
}

impl<A0, A1, A2, A3> Parser for All<(A0, A1, A2, A3)>
where
    A0: Parser,
    A1: Parser,
    A2: Parser,
    A3: Parser,
{
    type Dest = (A0::Dest, A1::Dest, A2::Dest, A3::Dest);
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let (remaining, a0) = self.parser.0.parse(input)?;
        let (remaining, a1) = self.parser.1.parse(remaining)?;
        let (remaining, a2) = self.parser.2.parse(remaining)?;
        self.parser
            .3
            .parse(remaining)
            .map(move |(remaining, a3)| (remaining, (a0, a1, a2, a3)))
    }
}

/// Комбинатор, который вытаскивает значения из пары `"ключ":значение, `.
/// Для простоты реализации, запятая всегда нужна в конце пары ключ-значение,
/// простое '"ключ":значение' читаться не будет.
#[derive(Debug, Clone)]
#[allow(clippy::type_complexity)]
pub struct KeyValue<T> {
    pub(crate) parser: Delimited<
        All<(StripWhitespace<QuotedTag>, StripWhitespace<Tag>)>,
        StripWhitespace<T>,
        StripWhitespace<Tag>,
    >,
}

impl<T> Parser for KeyValue<T>
where
    T: Parser,
{
    type Dest = T::Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        self.parser.parse(input)
    }
}

/// Комбинатор, который возвращает результаты дочерних парсеров, если их
/// удалось применить друг после друга в любом порядке. Результат возвращается в
/// том порядке, в каком `Permutation` был сконструирован
/// (аналог `permutation` из `nom`).
#[derive(Debug, Clone)]
pub struct Permutation<T> {
    pub(crate) parsers: T,
}

impl<A0, A1> Parser for Permutation<(A0, A1)>
where
    A0: Parser,
    A1: Parser,
{
    type Dest = (A0::Dest, A1::Dest);
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        match self.parsers.0.parse(input) {
            Ok((remaining, a0)) => self
                .parsers
                .1
                .parse(remaining)
                .map(|(remaining, a1)| (remaining, (a0, a1))),
            Err(()) => self.parsers.1.parse(input).and_then(|(remaining, a1)| {
                self.parsers
                    .0
                    .parse(remaining)
                    .map(|(remaining, a0)| (remaining, (a0, a1)))
            }),
        }
    }
}

impl<A0, A1, A2> Parser for Permutation<(A0, A1, A2)>
where
    A0: Parser,
    A1: Parser,
    A2: Parser,
{
    type Dest = (A0::Dest, A1::Dest, A2::Dest);
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        match self.parsers.0.parse(input) {
            Ok((remaining, a0)) => match self.parsers.1.parse(remaining) {
                Ok((remaining, a1)) => self
                    .parsers
                    .2
                    .parse(remaining)
                    .map(|(remaining, a2)| (remaining, (a0, a1, a2))),
                Err(()) => self.parsers.2.parse(remaining).and_then(|(remaining, a2)| {
                    self.parsers
                        .1
                        .parse(remaining)
                        .map(|(remaining, a1)| (remaining, (a0, a1, a2)))
                }),
            },
            Err(()) => match self.parsers.1.parse(input) {
                Ok((remaining, a1)) => match self.parsers.0.parse(remaining) {
                    Ok((remaining, a0)) => self
                        .parsers
                        .2
                        .parse(remaining)
                        .map(|(remaining, a2)| (remaining, (a0, a1, a2))),
                    Err(()) => self.parsers.2.parse(remaining).and_then(|(remaining, a2)| {
                        self.parsers
                            .0
                            .parse(remaining)
                            .map(|(remaining, a0)| (remaining, (a0, a1, a2)))
                    }),
                },
                Err(()) => self.parsers.2.parse(input).and_then(|(remaining, a2)| {
                    match self.parsers.0.parse(remaining) {
                        Ok((remaining, a0)) => self
                            .parsers
                            .1
                            .parse(remaining)
                            .map(|(remaining, a1)| (remaining, (a0, a1, a2))),
                        Err(()) => self.parsers.1.parse(remaining).and_then(|(remaining, a1)| {
                            self.parsers
                                .0
                                .parse(remaining)
                                .map(|(remaining, a0)| (remaining, (a0, a1, a2)))
                        }),
                    }
                }),
            },
        }
    }
}

/// Комбинатор списка из любого числа элементов, которые надо читать
/// вложенным парсером. Граница списка определяется квадратными (`[`&`]`)
/// скобками.
/// Для простоты реализации, после каждого элемента списка должна быть запятая.
#[derive(Debug, Clone)]
pub struct List<T> {
    pub(crate) parser: T,
}

impl<T: Parser> Parser for List<T> {
    type Dest = Vec<T::Dest>;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let mut remaining = input.trim_start().strip_prefix('[').ok_or(())?.trim_start();
        let mut result = Vec::new();
        while !remaining.is_empty() {
            match remaining.strip_prefix(']') {
                Some(remaining) => return Ok((remaining.trim_start(), result)),
                None => {
                    let (new_remaining, item) = self.parser.parse(remaining)?;
                    let new_remaining = new_remaining
                        .trim_start()
                        .strip_prefix(',')
                        .ok_or(())?
                        .trim_start();
                    result.push(item);
                    remaining = new_remaining;
                }
            }
        }
        Err(()) // строка кончилась, не закрыв скобку
    }
}

/// Комбинатор вернёт результат, который будет успешно
/// получен первым из дочерних комбинаторов (аналог `alt` из `nom`).
#[derive(Debug, Clone)]
pub struct Alt<T> {
    pub(crate) parser: T,
}

impl<A0, A1, Dest> Parser for Alt<(A0, A1)>
where
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
{
    type Dest = Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        if let Ok(ok) = self.parser.0.parse(input) {
            return Ok(ok);
        }
        self.parser.1.parse(input)
    }
}

impl<A0, A1, A2, Dest> Parser for Alt<(A0, A1, A2)>
where
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
    A2: Parser<Dest = Dest>,
{
    type Dest = Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        self.parser
            .0
            .parse(input)
            .or_else(|_| self.parser.1.parse(input))
            .or_else(|_| self.parser.2.parse(input))
    }
}

impl<A0, A1, A2, A3, Dest> Parser for Alt<(A0, A1, A2, A3)>
where
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
    A2: Parser<Dest = Dest>,
    A3: Parser<Dest = Dest>,
{
    type Dest = Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        if let Ok(ok) = self.parser.0.parse(input) {
            return Ok(ok);
        }
        if let Ok(ok) = self.parser.1.parse(input) {
            return Ok(ok);
        }
        if let Ok(ok) = self.parser.2.parse(input) {
            return Ok(ok);
        }
        self.parser.3.parse(input)
    }
}

impl<A0, A1, A2, A3, A4, A5, A6, A7, Dest> Parser for Alt<(A0, A1, A2, A3, A4, A5, A6, A7)>
where
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
    A2: Parser<Dest = Dest>,
    A3: Parser<Dest = Dest>,
    A4: Parser<Dest = Dest>,
    A5: Parser<Dest = Dest>,
    A6: Parser<Dest = Dest>,
    A7: Parser<Dest = Dest>,
{
    type Dest = Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        self.parser
            .0
            .parse(input)
            .or_else(|_| self.parser.1.parse(input))
            .or_else(|_| self.parser.2.parse(input))
            .or_else(|_| self.parser.3.parse(input))
            .or_else(|_| self.parser.4.parse(input))
            .or_else(|_| self.parser.5.parse(input))
            .or_else(|_| self.parser.6.parse(input))
            .or_else(|_| self.parser.7.parse(input))
    }
}

/// Комбинатор для применения дочернего парсера N раз
/// (аналог `take` из `nom`).
pub struct Take<T> {
    pub(crate) count: usize,
    pub(crate) parser: T,
}

impl<T: Parser> Parser for Take<T> {
    type Dest = Vec<T::Dest>;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let mut remaining = input;
        let mut result = Vec::new();
        for _ in 0..self.count {
            let (new_remaining, new_result) = self.parser.parse(remaining)?;
            result.push(new_result);
            remaining = new_remaining;
        }
        Ok((remaining, result))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::{constructors::*, stdp, traits::Parser};

    #[test]
    fn test_quote() {
        assert_eq!(quote(r#"411"#), r#""411""#.to_string());
        assert_eq!(quote(r#"4\11""#), r#""4\\11\"""#.to_string());
    }

    #[test]
    fn test_do_unquote_non_escaped() {
        assert_eq!(
            do_unquote_non_escaped(r#""411""#.into()),
            Ok(("".into(), "411".into()))
        );
        assert_eq!(do_unquote_non_escaped(r#" "411""#.into()), Err(()));
        assert_eq!(do_unquote_non_escaped(r#"411"#.into()), Err(()));
    }

    #[test]
    fn test_unquote() {
        assert_eq!(
            Unquote.parse(r#""411""#.into()),
            Ok(("".into(), "411".into()))
        );
        assert_eq!(Unquote.parse(r#" "411""#.into()), Err(()));
        assert_eq!(Unquote.parse(r#"411"#.into()), Err(()));

        assert_eq!(
            Unquote.parse(r#""ni\\c\"e""#.into()),
            Ok(("".into(), r#"ni\c"e"#.into()))
        );
    }

    #[test]
    fn test_tag() {
        assert_eq!(
            tag("key=").parse("key=value".into()),
            Ok(("value".into(), ()))
        );
        assert_eq!(tag("key=").parse("key:value".into()), Err(()));
    }

    #[test]
    fn test_quoted_tag() {
        assert_eq!(
            quoted_tag("key").parse(r#""key"=value"#.into()),
            Ok(("=value".into(), ()))
        );
        assert_eq!(quoted_tag("key").parse(r#""key:"value"#.into()), Err(()));
        assert_eq!(quoted_tag("key").parse(r#"key=value"#.into()), Err(()));
    }

    #[test]
    fn test_strip_whitespace() {
        assert_eq!(
            strip_whitespace(tag("hello")).parse(" hello world".into()),
            Ok(("world".into(), ()))
        );
        assert_eq!(
            strip_whitespace(tag("hello")).parse("hello".into()),
            Ok(("".into(), ()))
        );
        assert_eq!(
            strip_whitespace(stdp::U32).parse(" 42 answer".into()),
            Ok(("answer".into(), 42))
        );
    }

    #[test]
    fn test_delimited() {
        assert_eq!(
            delimited(tag("["), stdp::U32, tag("]")).parse("[0x32]".into()),
            Ok(("".into(), 0x32))
        );
        assert_eq!(
            delimited(tag("["), stdp::U32, tag("]")).parse("[0x32] nice".into()),
            Ok((" nice".into(), 0x32))
        );
        assert_eq!(
            delimited(tag("["), stdp::U32, tag("]")).parse("0x32]".into()),
            Err(())
        );
        assert_eq!(
            delimited(tag("["), stdp::U32, tag("]")).parse("[0x32".into()),
            Err(())
        );
    }

    #[test]
    fn test_key_value() {
        assert_eq!(
            key_value("key", stdp::U32).parse(r#""key":32,"#.into()),
            Ok(("".into(), 32))
        );
        assert_eq!(
            key_value("key", stdp::U32).parse(r#"key:32,"#.into()),
            Err(())
        );
        assert_eq!(
            key_value("key", stdp::U32).parse(r#""key":32"#.into()),
            Err(())
        );
        assert_eq!(
            key_value("key", stdp::U32).parse(r#" "key" : 32 , nice"#.into()),
            Ok(("nice".into(), 32))
        );
    }

    #[test]
    fn test_list() {
        assert_eq!(
            list(stdp::U32).parse("[1,2,3,4,]".into()),
            Ok(("".into(), vec![1, 2, 3, 4,]))
        );
        assert_eq!(
            list(stdp::U32).parse(" [ 1 , 2 , 3 , 4 , ] nice".into()),
            Ok(("nice".into(), vec![1, 2, 3, 4,]))
        );
        assert_eq!(list(stdp::U32).parse("1,2,3,4,".into()), Err(()));
        assert_eq!(list(stdp::U32).parse("[]".into()), Ok(("".into(), vec![])));
    }
}
