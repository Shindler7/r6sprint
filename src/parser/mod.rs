//! Механизмы парсинга лог-файлов.

pub mod api;
pub(crate) mod combinators;
pub(crate) mod constructors;
pub(crate) mod domain;
pub mod errors;
pub(crate) mod logs;
pub(crate) mod stdp;
pub(crate) mod traits;

// Реэкспорт доменных и log-типов, чтобы не ломать внешние use.
pub use domain::{Announcements, AssetDsc, AuthData, Backet, UserBacket, UserBackets, UserCash};
pub use logs::{
    AppLogErrorKind, AppLogJournalKind, AppLogKind, AppLogTraceKind, LogKind, LogLine, Status,
    SystemLogErrorKind, SystemLogKind, SystemLogTraceKind,
};
