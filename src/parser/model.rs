//! Типизированные модели данных для парсеров.

use crate::parser::{parse, stdp};
use crate::parser::parse::{Announcements, AuthData, Delimited, KeyValue, Map, Permutation, Preceded, StripWhitespace, Tag, Unquote, UserBacket, UserCash};
use crate::parser::traits::Parsable;
use crate::parser::constructors::*;


/// Комбинатор, который вернёт тот результат, который будет успешно
/// получен первым из дочерних комбинаторов
/// (аналог `alt` из `nom`)
#[derive(Debug, Clone)]
pub(crate) struct Alt<T> {
    pub(crate) parser: T,
}

/// Статус, которые можно парсить
pub(crate) enum Status {
    Ok,
    Err(String),
}

impl Parsable for Status {
    type Parser = Alt<(
        Map<Tag, fn(()) -> Self>,
        Map<Delimited<Tag, Unquote, Tag>, fn(String) -> Self>,
    )>;
    fn parser() -> Self::Parser {
        fn to_ok(_: ()) -> Status {
            Status::Ok
        }
        fn to_err(error: String) -> Status {
            Status::Err(error)
        }
        alt2(
            parse::map(parse::tag("Ok"), to_ok),
            parse::map(parse::delimited(parse::tag("Err("), parse::unquote(), parse::tag(")")), to_err),
        )
    }
}

/// Все виды логов
#[derive(Debug, Clone, PartialEq)]
pub enum LogKind {
    System(SystemLogKind),
    App(AppLogKind),
}

/// Все виды [системных](LogKind) логов
#[derive(Debug, Clone, PartialEq)]
pub enum SystemLogKind {
    Error(SystemLogErrorKind),
    Trace(SystemLogTraceKind),
}

/// Trace [системы](SystemLogKind)
#[derive(Debug, Clone, PartialEq)]
pub enum SystemLogTraceKind {
    SendRequest(String),
    GetResponse(String),
}

/// Error [системы](SystemLogKind)
#[derive(Debug, Clone, PartialEq)]
pub enum SystemLogErrorKind {
    NetworkError(String),
    AccessDenied(String),
}

/// Все виды [логов приложения](LogKind) логов
#[derive(Debug, Clone, PartialEq)]
pub enum AppLogKind {
    Error(AppLogErrorKind),
    Trace(AppLogTraceKind),
    Journal(AppLogJournalKind),
}

/// Error [приложения](AppLogKind)
#[derive(Debug, Clone, PartialEq)]
pub enum AppLogErrorKind {
    LackOf(String),
    SystemError(String),
}

// подсказка: а поля не слишком много места на стэке занимают?
/// Trace [приложения](AppLogKind)
#[derive(Debug, Clone, PartialEq)]
pub enum AppLogTraceKind {
    Connect(AuthData),
    SendRequest(String),
    Check(Announcements),
    GetResponse(String),
}

/// Журнал [приложения](AppLogKind), самые высокоуровневые события
#[derive(Debug, Clone, PartialEq)]
pub enum AppLogJournalKind {
    CreateUser {
        user_id: String,
        authorized_capital: u32,
    },
    DeleteUser {
        user_id: String,
    },
    RegisterAsset {
        asset_id: String,
        user_id: String,
        liquidity: u32,
    },
    UnregisterAsset {
        asset_id: String,
        user_id: String,
    },
    DepositCash(UserCash),
    WithdrawCash(UserCash),
    BuyAsset(UserBacket),
    SellAsset(UserBacket),
}

impl Parsable for SystemLogErrorKind {
    type Parser = Preceded<
        Tag,
        Alt<(
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> SystemLogErrorKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> SystemLogErrorKind,
            >,
        )>,
    >;
    fn parser() -> Self::Parser {
        parse::preceded(
            parse::tag("Error"),
            parse::alt2(
                parse::map(
                    parse::preceded(
                        parse::strip_whitespace(parse::tag("NetworkError")),
                        parse::strip_whitespace(parse::unquote()),
                    ),
                    |error| SystemLogErrorKind::NetworkError(error),
                ),
                parse::map(
                    parse::preceded(
                        parse::strip_whitespace(parse::tag("AccessDenied")),
                        parse::strip_whitespace(parse::unquote()),
                    ),
                    |error| SystemLogErrorKind::AccessDenied(error),
                ),
            ),
        )
    }
}

impl Parsable for SystemLogTraceKind {
    type Parser = Preceded<
        Tag,
        Alt<(
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> SystemLogTraceKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> SystemLogTraceKind,
            >,
        )>,
    >;
    fn parser() -> Self::Parser {
        parse::preceded(
            parse::tag("Trace"),
            parse::alt2(
                parse::map(
                    parse::preceded(
                        parse::strip_whitespace(parse::tag("SendRequest")),
                        parse::strip_whitespace(parse::unquote()),
                    ),
                    |request| SystemLogTraceKind::SendRequest(request),
                ),
                parse::map(
                    parse::preceded(
                        parse::strip_whitespace(parse::tag("GetResponse")),
                        parse::strip_whitespace(parse::unquote()),
                    ),
                    |response| SystemLogTraceKind::GetResponse(response),
                ),
            ),
        )
    }
}

impl Parsable for SystemLogKind {
    type Parser = StripWhitespace<
        Preceded<
            Tag,
            Alt<(
                Map<
                    <SystemLogTraceKind as Parsable>::Parser,
                    fn(SystemLogTraceKind) -> SystemLogKind,
                >,
                Map<
                    <SystemLogErrorKind as Parsable>::Parser,
                    fn(SystemLogErrorKind) -> SystemLogKind,
                >,
            )>,
        >,
    >;
    fn parser() -> Self::Parser {
        parse::strip_whitespace(parse::preceded(
            parse::tag("System::"),
            parse::alt2(
                parse::map(SystemLogTraceKind::parser(), |trace| {
                    SystemLogKind::Trace(trace)
                }),
                parse::map(SystemLogErrorKind::parser(), |error| {
                    SystemLogKind::Error(error)
                }),
            ),
        ))
    }
}

impl Parsable for AppLogErrorKind {
    type Parser = Preceded<
        Tag,
        Alt<(
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> AppLogErrorKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> AppLogErrorKind,
            >,
        )>,
    >;
    fn parser() -> Self::Parser {
        parse::preceded(
            parse::tag("Error"),
            parse::alt2(
                parse::map(
                    parse::preceded(parse::strip_whitespace(parse::tag("LackOf")), parse::strip_whitespace(parse::unquote())),
                    |error| AppLogErrorKind::LackOf(error),
                ),
                parse::map(
                    parse::preceded(
                        parse::strip_whitespace(parse::tag("SystemError")),
                        parse::strip_whitespace(parse::unquote()),
                    ),
                    |error| AppLogErrorKind::SystemError(error),
                ),
            ),
        )
    }
}

impl Parsable for AppLogTraceKind {
    type Parser = Preceded<
        Tag,
        Alt<(
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<<AuthData as Parsable>::Parser>>,
                fn(AuthData) -> AppLogTraceKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> AppLogTraceKind,
            >,
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    StripWhitespace<<Announcements as Parsable>::Parser>,
                >,
                fn(Announcements) -> AppLogTraceKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> AppLogTraceKind,
            >,
        )>,
    >;
    fn parser() -> Self::Parser {
        parse::preceded(
            parse::tag("Trace"),
            parse::alt4(
                parse::map(
                    parse::preceded(
                        parse::strip_whitespace(parse::tag("Connect")),
                        parse::strip_whitespace(AuthData::parser()),
                    ),
                    |authdata| AppLogTraceKind::Connect(authdata),
                ),
                parse::map(
                    parse::preceded(
                        parse::strip_whitespace(parse::tag("SendRequest")),
                        parse::strip_whitespace(parse::unquote()),
                    ),
                    |trace| AppLogTraceKind::SendRequest(trace),
                ),
                parse::map(
                    parse::preceded(
                        parse::strip_whitespace(parse::tag("Check")),
                        parse::strip_whitespace(Announcements::parser()),
                    ),
                    |announcements| AppLogTraceKind::Check(announcements),
                ),
                parse::map(
                    parse::preceded(
                        parse::strip_whitespace(parse::tag("GetResponse")),
                        parse::strip_whitespace(parse::unquote()),
                    ),
                    |trace| AppLogTraceKind::GetResponse(trace),
                ),
            ),
        )
    }
}

impl Parsable for AppLogJournalKind {
    type Parser = Preceded<
        Tag,
        Alt<(
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    Delimited<Tag, Permutation<(KeyValue<Unquote>, KeyValue<stdp::U32>)>, Tag>,
                >,
                fn((String, u32)) -> AppLogJournalKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, Delimited<Tag, KeyValue<Unquote>, Tag>>,
                fn(String) -> AppLogJournalKind,
            >,
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    Delimited<
                        Tag,
                        Permutation<(KeyValue<Unquote>, KeyValue<Unquote>, KeyValue<stdp::U32>)>,
                        Tag,
                    >,
                >,
                fn((String, String, u32)) -> AppLogJournalKind,
            >,
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    Delimited<Tag, Permutation<(KeyValue<Unquote>, KeyValue<Unquote>)>, Tag>,
                >,
                fn((String, String)) -> AppLogJournalKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, <UserCash as Parsable>::Parser>,
                fn(UserCash) -> AppLogJournalKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, <UserCash as Parsable>::Parser>,
                fn(UserCash) -> AppLogJournalKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, <UserBacket as Parsable>::Parser>,
                fn(UserBacket) -> AppLogJournalKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, <UserBacket as Parsable>::Parser>,
                fn(UserBacket) -> AppLogJournalKind,
            >,
        )>,
    >;
    fn parser() -> Self::Parser {
        parse::preceded(
            parse::tag("Journal"),
            parse::alt8(
                parse::map(
                    parse::preceded(
                        parse::strip_whitespace(parse::tag("CreateUser")),
                        parse::delimited(
                            parse::tag("{"),
                            parse::permutation2(
                                parse::key_value("user_id", parse::unquote()),
                                parse::key_value("authorized_capital", stdp::U32),
                            ),
                            parse::tag("}"),
                        ),
                    ),
                    |(user_id, authorized_capital)| AppLogJournalKind::CreateUser {
                        user_id,
                        authorized_capital,
                    },
                ),
                parse::map(
                    parse::preceded(
                        parse::strip_whitespace(parse::tag("DeleteUser")),
                        parse::delimited(parse::tag("{"), parse::key_value("user_id", parse::unquote()), parse::tag("}")),
                    ),
                    |user_id| AppLogJournalKind::DeleteUser { user_id },
                ),
                parse::map(
                    parse::preceded(
                        parse::strip_whitespace(parse::tag("RegisterAsset")),
                        parse::delimited(
                            parse::tag("{"),
                            parse::permutation3(
                                parse::key_value("asset_id", parse::unquote()),
                                parse::key_value("user_id", parse::unquote()),
                                parse::key_value("liquidity", stdp::U32),
                            ),
                            parse::tag("}"),
                        ),
                    ),
                    |(asset_id, user_id, liquidity)| AppLogJournalKind::RegisterAsset {
                        asset_id,
                        user_id,
                        liquidity,
                    },
                ),
                parse::map(
                    parse::preceded(
                        parse::strip_whitespace(parse::tag("UnregisterAsset")),
                        parse::delimited(
                            parse::tag("{"),
                            parse::permutation2(
                                parse::key_value("asset_id", parse::unquote()),
                                parse::key_value("user_id", parse::unquote()),
                            ),
                            parse::tag("}"),
                        ),
                    ),
                    |(asset_id, user_id)| AppLogJournalKind::UnregisterAsset { asset_id, user_id },
                ),
                parse::map(
                    parse::preceded(parse::strip_whitespace(parse::tag("DepositCash")), UserCash::parser()),
                    |user_cash| AppLogJournalKind::DepositCash(user_cash),
                ),
                parse::map(
                    parse::preceded(parse::strip_whitespace(parse::tag("WithdrawCash")), UserCash::parser()),
                    |user_cash| AppLogJournalKind::DepositCash(user_cash),
                ),
                parse::map(
                    parse::preceded(parse::strip_whitespace(parse::tag("BuyAsset")), UserBacket::parser()),
                    |user_backet| AppLogJournalKind::BuyAsset(user_backet),
                ),
                parse::map(
                    parse::preceded(parse::strip_whitespace(parse::tag("SellAsset")), UserBacket::parser()),
                    |user_backet| AppLogJournalKind::SellAsset(user_backet),
                ),
            ),
        )
    }
}

impl Parsable for AppLogKind {
    type Parser = StripWhitespace<
        Preceded<
            Tag,
            Alt<(
                Map<<AppLogErrorKind as Parsable>::Parser, fn(AppLogErrorKind) -> AppLogKind>,
                Map<<AppLogTraceKind as Parsable>::Parser, fn(AppLogTraceKind) -> AppLogKind>,
                Map<<AppLogJournalKind as Parsable>::Parser, fn(AppLogJournalKind) -> AppLogKind>,
            )>,
        >,
    >;
    fn parser() -> Self::Parser {
        parse::strip_whitespace(parse::preceded(
            parse::tag("App::"),
            parse::alt3(
                parse::map(AppLogErrorKind::parser(), |error| AppLogKind::Error(error)),
                parse::map(AppLogTraceKind::parser(), |trace| AppLogKind::Trace(trace)),
                parse::map(AppLogJournalKind::parser(), |journal| {
                    AppLogKind::Journal(journal)
                }),
            ),
        ))
    }
}

impl Parsable for LogKind {
    type Parser = StripWhitespace<
        Alt<(
            Map<<SystemLogKind as Parsable>::Parser, fn(SystemLogKind) -> LogKind>,
            Map<<AppLogKind as Parsable>::Parser, fn(AppLogKind) -> LogKind>,
        )>,
    >;
    fn parser() -> Self::Parser {
        parse::strip_whitespace(parse::alt2(
            parse::map(SystemLogKind::parser(), |system| LogKind::System(system)),
            parse::map(AppLogKind::parser(), |app| LogKind::App(app)),
        ))
    }
}