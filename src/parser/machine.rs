//! Стейт-машина разбора выписки.

use std::ops::ControlFlow;

use indexmap::IndexMap;

use super::encoding::{Encoding, parse_text};
use super::error::{Error, ParserError, ParserErrorKind, SectionContext};
use super::hooks::{HookError, SectionHook, SectionType};
use super::line::{Line, Section, numbered_lines};
use super::model::{AddDocError, Statement};

/// Билдер [`Parser`] с настройкой хуков секций.
#[derive(Default)]
pub struct ParserBuilder {
    section_hooks: Vec<Box<SectionHook>>,
}

impl ParserBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_hooks(mut self, hooks: Vec<Box<SectionHook>>) -> Self {
        self.section_hooks = hooks;
        self
    }

    #[must_use]
    pub fn build(self) -> Parser {
        Parser {
            section_hooks: self.section_hooks,
        }
    }
}

impl std::fmt::Debug for ParserBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParserBuilder")
            .field("section_hooks", &self.section_hooks.len())
            .finish()
    }
}

/// Парсер выписок 1CClientBankExchange.
pub struct Parser {
    section_hooks: Vec<Box<SectionHook>>,
}

impl Default for Parser {
    fn default() -> Self {
        ParserBuilder::new().build()
    }
}

impl std::fmt::Debug for Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Parser")
            .field("section_hooks", &self.section_hooks.len())
            .finish()
    }
}

impl Parser {
    /// Разбирает выписку из байтов (кодировки UTF-8 и CP1251, UTF-8 BOM допускается).
    ///
    /// # Errors
    ///
    /// Возвращает [`Error::Empty`] для пустого ввода, [`Error::Not1CStatement`],
    /// если данные не являются выпиской, [`Error::Unfinished`], если файл
    /// оборван, и [`Error::Syntax`] при синтаксической ошибке.
    pub fn parse(&self, content: &[u8]) -> Result<Statement, Error> {
        if content.iter().all(u8::is_ascii_whitespace) {
            return Err(Error::Empty);
        }
        let (raw, encoding) = parse_text(content).ok_or(Error::Not1CStatement)?;
        match self.parse_internal(&raw, encoding) {
            ControlFlow::Continue(State::Finished(statement)) => Ok(statement),
            ControlFlow::Break(err) => Err(Error::Syntax(err)),
            ControlFlow::Continue(_) => Err(Error::Unfinished),
        }
    }

    fn call_hooks(
        &self,
        section: SectionType,
        attrs: &mut IndexMap<String, String>,
        statement: &Statement,
    ) -> Result<(), HookError> {
        for hook in &self.section_hooks {
            hook(section, attrs, statement)?;
        }
        Ok(())
    }

    /// Прогоняет стейт-машину по строкам без промежуточной материализации:
    /// каждая строка лексируется и сразу подаётся в [`Parser::step`].
    fn parse_internal<'a>(
        &self,
        raw: &'a str,
        encoding: Encoding,
    ) -> ControlFlow<ParserError, State<'a>> {
        let mut state = State::Init;
        for (lineno, raw_line) in numbered_lines(raw) {
            let line = match Line::try_from(raw_line) {
                Ok(line) => line,
                Err(unrecognized) => {
                    return ControlFlow::Break(ParserError {
                        lineno,
                        kind: ParserErrorKind::UnrecognizedLine {
                            line: unrecognized.to_string(),
                        },
                    });
                }
            };
            state = self.step(state, lineno, line, encoding)?;
        }
        ControlFlow::Continue(state)
    }

    fn step<'a>(
        &self,
        state: State<'a>,
        lineno: usize,
        line: Line<'a>,
        encoding: Encoding,
    ) -> ControlFlow<ParserError, State<'a>> {
        match (state, line) {
            // Начальное состояние, каждая выписка в первой строке имеет заголовок "1CClientBankExchange"
            (State::Init, Line::Section(Section::StartOfFile)) => {
                ControlFlow::Continue(State::Header(IndexMap::new()))
            }
            // После заголовка — заголовочные атрибуты без границ секции
            (State::Header(mut attrs), Line::Attr(k, v)) => {
                attrs.insert(k.to_string(), v.to_string());
                ControlFlow::Continue(State::Header(attrs))
            }
            // После заголовка — либо СекцияДокумент
            (State::Header(attrs), Line::Section(Section::Document(typ))) => {
                ControlFlow::Continue(State::Document {
                    statement: Statement::new(encoding, attrs),
                    typ,
                    attrs: IndexMap::new(),
                })
            }
            // Либо СекцияРасчСчет
            (State::Header(attrs), Line::Section(Section::Account)) => {
                ControlFlow::Continue(State::Account {
                    statement: Statement::new(encoding, attrs),
                    attrs: IndexMap::new(),
                })
            }
            // Чтение документа
            (
                State::Document {
                    mut attrs,
                    statement,
                    typ,
                },
                Line::Attr(k, v),
            ) => {
                attrs.insert(k.to_string(), v.to_string());
                ControlFlow::Continue(State::Document {
                    statement,
                    typ,
                    attrs,
                })
            }
            (
                State::Document {
                    statement,
                    typ,
                    attrs,
                },
                Line::Section(Section::EndOfDocument),
            ) => self.finish_document(statement, typ, attrs, lineno),
            // Чтение счёта
            (
                State::Account {
                    mut attrs,
                    statement,
                },
                Line::Attr(k, v),
            ) => {
                attrs.insert(k.to_string(), v.to_string());
                ControlFlow::Continue(State::Account { statement, attrs })
            }
            (State::Account { attrs, statement }, Line::Section(Section::EndOfAccount)) => {
                self.finish_account(statement, attrs, lineno)
            }
            // Секции документа и счёта заканчиваются соответствующими секциями:
            // КонецДокумента и КонецРасчСчет, после чего парсер ищет следующую секцию
            (State::ReadNextSection { statement }, Line::Section(Section::Account)) => {
                ControlFlow::Continue(State::Account {
                    statement,
                    attrs: IndexMap::new(),
                })
            }
            (State::ReadNextSection { statement }, Line::Section(Section::Document(typ))) => {
                ControlFlow::Continue(State::Document {
                    statement,
                    typ,
                    attrs: IndexMap::new(),
                })
            }
            (State::ReadNextSection { statement }, Line::Section(Section::EndOfFile)) => {
                ControlFlow::Continue(State::Finished(statement))
            }
            (State::ReadNextSection { .. }, Line::Section(s)) => ControlFlow::Break(
                unexpected_section(lineno, &s, SectionContext::ReadNextSection),
            ),
            (State::Init, Line::Section(s)) => {
                ControlFlow::Break(unexpected_section(lineno, &s, SectionContext::Init))
            }
            (State::Header(_), Line::Section(s)) => {
                ControlFlow::Break(unexpected_section(lineno, &s, SectionContext::Header))
            }
            (State::Document { .. }, Line::Section(s)) => {
                ControlFlow::Break(unexpected_section(lineno, &s, SectionContext::Document))
            }
            (State::Account { .. }, Line::Section(s)) => {
                ControlFlow::Break(unexpected_section(lineno, &s, SectionContext::Account))
            }
            (State::Finished(_), Line::Section(s)) => {
                ControlFlow::Break(unexpected_section(lineno, &s, SectionContext::Finished))
            }
            // Атрибут вне секции недопустим
            (
                State::Init | State::ReadNextSection { .. } | State::Finished(_),
                Line::Attr(k, v),
            ) => ControlFlow::Break(unexpected_attribute(lineno, k, v)),
        }
    }

    fn finish_document<'a>(
        &self,
        mut statement: Statement,
        typ: &'a str,
        mut attrs: IndexMap<String, String>,
        lineno: usize,
    ) -> ControlFlow<ParserError, State<'a>> {
        match self.call_hooks(SectionType::Document, &mut attrs, &statement) {
            Ok(()) => match statement.add_document(typ, attrs) {
                Ok(()) => ControlFlow::Continue(State::ReadNextSection { statement }),
                Err(AddDocError::Warning(e)) => {
                    statement.add_warning((lineno, e));
                    ControlFlow::Continue(State::ReadNextSection { statement })
                }
            },
            Err(HookError::Warning(warn)) => {
                statement.add_warning((lineno, warn));
                ControlFlow::Continue(State::ReadNextSection { statement })
            }
            Err(HookError::Error(err)) => ControlFlow::Break(ParserError {
                lineno,
                kind: ParserErrorKind::HookError(err),
            }),
        }
    }

    fn finish_account<'a>(
        &self,
        mut statement: Statement,
        mut attrs: IndexMap<String, String>,
        lineno: usize,
    ) -> ControlFlow<ParserError, State<'a>> {
        match self.call_hooks(SectionType::Account, &mut attrs, &statement) {
            Ok(()) => match statement.add_account(attrs, lineno) {
                Ok(()) => ControlFlow::Continue(State::ReadNextSection { statement }),
                Err(err) => ControlFlow::Break(err),
            },
            Err(HookError::Warning(warn)) => {
                statement.add_warning((lineno, warn));
                ControlFlow::Continue(State::ReadNextSection { statement })
            }
            Err(HookError::Error(err)) => ControlFlow::Break(ParserError {
                lineno,
                kind: ParserErrorKind::HookError(err),
            }),
        }
    }
}

fn unexpected_section(
    lineno: usize,
    section: &Section<'_>,
    context: SectionContext,
) -> ParserError {
    ParserError {
        lineno,
        kind: ParserErrorKind::UnexpectedSection {
            found: section.to_string(),
            context,
        },
    }
}

fn unexpected_attribute(lineno: usize, key: &str, value: &str) -> ParserError {
    ParserError {
        lineno,
        kind: ParserErrorKind::UnexpectedAttribute {
            key: key.to_string(),
            value: value.to_string(),
        },
    }
}

#[derive(Debug)]
enum State<'a> {
    Init,
    Header(IndexMap<String, String>),
    Document {
        statement: Statement,
        typ: &'a str,
        attrs: IndexMap<String, String>,
    },
    Account {
        statement: Statement,
        attrs: IndexMap<String, String>,
    },
    ReadNextSection {
        statement: Statement,
    },
    Finished(Statement),
}
