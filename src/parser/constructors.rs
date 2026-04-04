//! Конструкторы для моделей данных.

use crate::parser::combinators::{
    All, Alt, Delimited, KeyValue, List, Map, Permutation, Preceded, QuotedTag, StripWhitespace,
    Tag, Take, Unquote,
};
use crate::parser::traits::Parser;

/// Конструктор [Unquote]
pub(crate) fn unquote() -> Unquote {
    Unquote
}

/// Конструктор [Tag]
pub(crate) fn tag(tag: &'static str) -> Tag {
    Tag { tag }
}

/// Конструктор [QuotedTag]
pub(crate) fn quoted_tag(tag: &'static str) -> QuotedTag {
    QuotedTag(Tag { tag })
}

/// Конструктор [StripWhitespace]
pub(crate) fn strip_whitespace<T: Parser>(parser: T) -> StripWhitespace<T> {
    StripWhitespace { parser }
}

/// Конструктор [Delimited]
pub(crate) fn delimited<Prefix, T, Suffix>(
    prefix_to_ignore: Prefix,
    dest_parser: T,
    suffix_to_ignore: Suffix,
) -> Delimited<Prefix, T, Suffix>
where
    Prefix: Parser,
    T: Parser,
    Suffix: Parser,
{
    Delimited {
        prefix_to_ignore,
        dest_parser,
        suffix_to_ignore,
    }
}

/// Конструктор [Map]
pub(crate) fn map<T: Parser, Dest: Sized, M: Fn(T::Dest) -> Dest>(parser: T, map: M) -> Map<T, M> {
    Map { parser, map }
}

/// Конструктор [Preceded]
pub(crate) fn preceded<Prefix, T>(prefix_to_ignore: Prefix, dest_parser: T) -> Preceded<Prefix, T>
where
    Prefix: Parser,
    T: Parser,
{
    Preceded {
        prefix_to_ignore,
        dest_parser,
    }
}

/// Конструктор [All] для двух парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub(crate) fn all2<A0: Parser, A1: Parser>(a0: A0, a1: A1) -> All<(A0, A1)> {
    All { parser: (a0, a1) }
}

/// Конструктор [All] для трёх парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub(crate) fn all3<A0: Parser, A1: Parser, A2: Parser>(
    a0: A0,
    a1: A1,
    a2: A2,
) -> All<(A0, A1, A2)> {
    All {
        parser: (a0, a1, a2),
    }
}

/// Конструктор [All] для четырёх парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub(crate) fn all4<A0: Parser, A1: Parser, A2: Parser, A3: Parser>(
    a0: A0,
    a1: A1,
    a2: A2,
    a3: A3,
) -> All<(A0, A1, A2, A3)> {
    All {
        parser: (a0, a1, a2, a3),
    }
}

/// Конструктор [KeyValue]
pub(crate) fn key_value<T: Parser>(key: &'static str, value_parser: T) -> KeyValue<T> {
    KeyValue {
        parser: delimited(
            all2(
                strip_whitespace(quoted_tag(key)),
                strip_whitespace(tag(":")),
            ),
            strip_whitespace(value_parser),
            strip_whitespace(tag(",")),
        ),
    }
}

/// Конструктор [Permutation] для двух парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub(crate) fn permutation2<A0: Parser, A1: Parser>(a0: A0, a1: A1) -> Permutation<(A0, A1)> {
    Permutation { parsers: (a0, a1) }
}

/// Конструктор [Permutation] для трёх парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub(crate) fn permutation3<A0: Parser, A1: Parser, A2: Parser>(
    a0: A0,
    a1: A1,
    a2: A2,
) -> Permutation<(A0, A1, A2)> {
    Permutation {
        parsers: (a0, a1, a2),
    }
}

/// Конструктор для [List]
pub(crate) fn list<T: Parser>(parser: T) -> List<T> {
    List { parser }
}

/// Конструктор [Alt] для двух парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub(crate) fn alt2<Dest, A0: Parser<Dest = Dest>, A1: Parser<Dest = Dest>>(
    a0: A0,
    a1: A1,
) -> Alt<(A0, A1)> {
    Alt { parser: (a0, a1) }
}

/// Конструктор [Alt] для трёх парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub(crate) fn alt3<
    Dest,
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
    A2: Parser<Dest = Dest>,
>(
    a0: A0,
    a1: A1,
    a2: A2,
) -> Alt<(A0, A1, A2)> {
    Alt {
        parser: (a0, a1, a2),
    }
}

/// Конструктор [Alt] для четырёх парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub(crate) fn alt4<
    Dest,
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
    A2: Parser<Dest = Dest>,
    A3: Parser<Dest = Dest>,
>(
    a0: A0,
    a1: A1,
    a2: A2,
    a3: A3,
) -> Alt<(A0, A1, A2, A3)> {
    Alt {
        parser: (a0, a1, a2, a3),
    }
}

/// Конструктор [Alt] для восьми парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub(crate) fn alt8<
    Dest,
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
    A2: Parser<Dest = Dest>,
    A3: Parser<Dest = Dest>,
    A4: Parser<Dest = Dest>,
    A5: Parser<Dest = Dest>,
    A6: Parser<Dest = Dest>,
    A7: Parser<Dest = Dest>,
>(
    a0: A0,
    a1: A1,
    a2: A2,
    a3: A3,
    a4: A4,
    a5: A5,
    a6: A6,
    a7: A7,
) -> Alt<(A0, A1, A2, A3, A4, A5, A6, A7)> {
    Alt {
        parser: (a0, a1, a2, a3, a4, a5, a6, a7),
    }
}

/// Конструктор `Take`
pub(crate) fn take<T: Parser>(count: usize, parser: T) -> Take<T> {
    Take { count, parser }
}
