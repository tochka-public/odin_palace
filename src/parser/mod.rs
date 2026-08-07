//! Парсер банковских выписок формата 1CClientBankExchange.
//!
//! Точка входа — [`Parser::parse`]; настройка хуков — через [`ParserBuilder`].
//!
//! Внутренняя структура:
//! - [`model`](self) — типы данных выписки ([`Statement`], [`Document`], [`Account`], [`Interval`]);
//! - ошибки ([`Error`], [`ParserError`], [`ParserErrorKind`], [`SectionContext`]);
//! - определение кодировки ([`Encoding`]);
//! - лексический разбор строк ([`Section`]);
//! - стейт-машина ([`Parser`], [`ParserBuilder`]);
//! - [`hooks`] — пользовательские хуки секций.

pub mod hooks;

mod attrs;
mod de;
mod encoding;
mod error;
mod line;
mod machine;
mod model;

pub use encoding::Encoding;
pub use error::{Error, ParserError, ParserErrorKind, SectionContext};
pub use line::Section;
pub use machine::{Parser, ParserBuilder};
pub use model::{Account, Document, Interval, Statement};
