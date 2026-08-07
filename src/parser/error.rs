//! Типы ошибок парсера.

/// Ошибка верхнего уровня, возвращаемая [`super::Parser::parse`].
#[derive(Debug)]
pub enum Error {
    /// Синтаксическая ошибка с привязкой к строке файла.
    Syntax(ParserError),
    /// Зарезервировано; в текущей реализации не возвращается.
    InvalidDocument,
    /// Файл закончился до секции "КонецФайла".
    Unfinished,
    /// Входные данные не являются выпиской 1CClientBankExchange.
    Not1CStatement,
    /// Входные данные пусты (или состоят только из пробельных символов).
    Empty,
}

/// Состояние парсера, в котором обнаружена ошибка.
#[derive(Debug, Clone)]
pub enum SectionContext {
    Header,
    Document,
    Account,
    Finished,
    Init,
    ReadNextSection,
}

#[derive(Debug, Clone)]
pub enum ParserErrorKind {
    UnexpectedSection {
        found: String,
        context: SectionContext,
    },
    UnexpectedAttribute {
        key: String,
        value: String,
    },
    UnrecognizedLine {
        line: String,
    },
    MissingField {
        field: String,
        context: SectionContext,
    },
    AccountParseError(String),
    DocumentParseError(String),
    HookError(String),
}

/// Синтаксическая ошибка с номером строки (нумерация с 1).
#[derive(Debug, Clone)]
pub struct ParserError {
    pub lineno: usize,
    pub kind: ParserErrorKind,
}
