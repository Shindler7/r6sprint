//! Парсеры для стандартных типов.

use crate::parser::traits::Parser;
use std::num::{NonZeroI32, NonZeroU32};

/// Беззнаковые числа.
#[derive(Debug)]
pub struct U32;

impl Parser for U32 {
    type Dest = u32;

    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let (remaining, is_hex) = input
            .strip_prefix("0x")
            .map_or((input, false), |remaining| (remaining, true));

        let end_idx = remaining
            .char_indices()
            .find_map(|(idx, c)| match (is_hex, c) {
                (true, 'a'..='f' | '0'..='9' | 'A'..='F') => None,
                (false, '0'..='9') => None,
                _ => Some(idx),
            })
            .unwrap_or(remaining.len());

        let radix = if is_hex { 16 } else { 10 };
        let value =
            NonZeroU32::new(u32::from_str_radix(&remaining[..end_idx], radix).map_err(|_| ())?)
                .ok_or(())?;

        Ok((&remaining[end_idx..], value.into()))
    }
}

/// Знаковые числа.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct I32;

impl Parser for I32 {
    type Dest = i32;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let end_idx = input
            .char_indices()
            .skip(1)
            .find_map(|(idx, c)| (!c.is_ascii_digit()).then_some(idx))
            .unwrap_or(input.len());

        let value = NonZeroI32::new(input[..end_idx].parse::<i32>().map_err(|_| ())?).ok_or(())?;

        Ok((&input[end_idx..], value.into()))
    }
}

/// Шестнадцатеричные байты (пригодится при парсинге блобов).
#[derive(Debug, Clone)]
pub struct Byte;

impl Parser for Byte {
    type Dest = u8;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ()> {
        let (to_parse, remaining) = input.split_at_checked(2).ok_or(())?;
        if !to_parse.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(());
        }
        let value = u8::from_str_radix(to_parse, 16).map_err(|_| ())?;
        Ok((remaining, value))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::traits::Parser;

    #[test]
    fn test_u32() {
        assert_eq!(U32.parse("411".into()), Ok(("".into(), 411)));
        assert_eq!(U32.parse("411ab".into()), Ok(("ab".into(), 411)));
        assert_eq!(U32.parse("".into()), Err(()));
        assert_eq!(U32.parse("-3".into()), Err(()));
        assert_eq!(U32.parse("0x03".into()), Ok(("".into(), 0x3)));
        assert_eq!(U32.parse("0x03abg".into()), Ok(("g".into(), 0x3ab)));
        assert_eq!(U32.parse("0x".into()), Err(()));
    }

    #[test]
    fn test_i32() {
        assert_eq!(I32.parse("411".into()), Ok(("".into(), 411)));
        assert_eq!(I32.parse("411ab".into()), Ok(("ab".into(), 411)));
        assert_eq!(I32.parse("".into()), Err(()));
        assert_eq!(I32.parse("-3".into()), Ok(("".into(), -3)));
        assert_eq!(I32.parse("0x03".into()), Err(()));
        assert_eq!(I32.parse("-".into()), Err(()));
    }
}
