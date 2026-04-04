//! Механизмы парсинга лог-файлов.

pub(crate) mod traits;
pub(crate) mod stdp;
pub(crate) mod combinators;
pub(crate) mod constructors;
pub(crate) mod domain;
pub(crate) mod logs;
pub mod api;

// Реэкспорт доменных и log-типов, чтобы не ломать внешние use.
pub use domain::{
    Announcements, AssetDsc, AuthData, Backet, UserBacket, UserBackets, UserCash,
};
pub use logs::{
    AppLogErrorKind, AppLogJournalKind, AppLogKind, AppLogTraceKind, LogKind, LogLine,
    SystemLogErrorKind, SystemLogKind, SystemLogTraceKind,
};
