#![allow(dead_code)]
use chrono::NaiveDate;
use encoding_rs::WINDOWS_1251;
use hooks::{HookError, SectionHook, SectionType};
use indexmap::IndexMap;
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};
use std::borrow::Cow;
use std::ops::ControlFlow;

pub mod hooks;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Document {
    #[serde(rename = "Номер")]
    pub doc_number: String,

    #[serde(rename = "СекцияДокумент")]
    pub doc_type: String,

    #[serde(rename = "ВидОплаты")]
    pub payment_type: Option<String>,

    #[serde(rename = "Дата", deserialize_with = "deserialize_dt")]
    pub doc_date: NaiveDate,

    #[serde(rename = "НазначениеПлатежа")]
    pub purpose: String,

    #[serde(rename = "Очередность")]
    pub ordering: Option<String>, // optional?

    #[serde(rename = "Сумма")]
    pub amount: Decimal,

    #[serde(rename = "ПлательщикИНН")]
    pub counterparty_inn: String,

    #[serde(rename = "ПлательщикКПП")]
    pub counterparty_kpp: Option<String>,

    #[serde(rename = "ПлательщикБИК")]
    pub counterparty_bic: String,

    #[serde(rename = "ПлательщикБанк1")]
    pub counterparty_bank1: String,

    #[serde(rename = "ПлательщикСчет")]
    pub counterparty_account: String,

    #[serde(rename = "Плательщик")]
    pub counterparty: Option<String>,

    #[serde(rename = "Плательщик1")]
    pub counterparty_1: Option<String>,

    #[serde(
        rename = "ДатаСписано",
        default,
        deserialize_with = "try_deserialize_dt"
    )]
    pub outcome_date: Option<NaiveDate>,

    #[serde(rename = "ПлательщикРасчСчет")]
    pub counterparty_ras_account: Option<String>,

    #[serde(rename = "ПлательщикКорсчет")]
    pub counterparty_cor_account: Option<String>,

    #[serde(rename = "ПолучательИНН")]
    pub payee_inn: String,

    #[serde(rename = "Получатель")]
    pub payee: Option<String>,

    #[serde(rename = "ПолучательСчет")]
    pub payee_account: String,

    #[serde(rename = "ПолучательКПП")]
    pub payee_kpp: Option<String>,

    #[serde(rename = "ПолучательБИК")]
    pub payee_bic: String,

    #[serde(rename = "ПолучательБанк1")]
    pub payee_bank1: String,

    #[serde(rename = "ПолучательРасчСчет")]
    pub payee_ras_account: Option<String>,

    #[serde(rename = "ПолучательКорсчет")]
    pub payee_cor_account: Option<String>,

    #[serde(
        rename = "ДатаПоступило",
        default,
        deserialize_with = "try_deserialize_dt"
    )]
    pub income_date: Option<NaiveDate>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Interval {
    #[serde(rename = "ДатаНачала", deserialize_with = "deserialize_dt")]
    pub date_start: NaiveDate,
    #[serde(rename = "ДатаКонца", default, deserialize_with = "try_deserialize_dt")]
    pub date_end: Option<NaiveDate>,
    #[serde(rename = "ВсегоПоступило")]
    pub total_income: Option<Decimal>,
    #[serde(rename = "ВсегоСписано")]
    pub total_expense: Option<Decimal>,
    #[serde(rename = "НачальныйОстаток")]
    pub start_amount: Decimal,
    #[serde(rename = "КонечныйОстаток")]
    pub end_amount: Option<Decimal>,
}

#[derive(Clone, Debug)]
pub struct Account {
    pub number: String,
    pub intervals: Vec<Interval>,
}

#[derive(Clone, Debug)]
pub struct Statement {
    pub encoding: Encoding,
    pub header: IndexMap<String, String>,
    pub accounts: IndexMap<String, Account>,
    pub documents: Vec<Document>,
    pub warnings: Vec<(usize, String)>,
}

impl Statement {
    pub fn new(encoding: Encoding, header: IndexMap<String, String>) -> Self {
        Self {
            encoding,
            header,
            accounts: Default::default(),
            documents: Default::default(),
            warnings: Default::default(),
        }
    }
    fn add_document(
        &mut self,
        typ: &str,
        attrs: IndexMap<String, String>,
    ) -> Result<(), AddDocError> {
        let value_map_json: IndexMap<String, serde_json::Value> = attrs
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect();
        let mut value_map_json = value_map_json;
        value_map_json.insert(
            "СекцияДокумент".into(),
            serde_json::Value::String(typ.into()),
        );
        let value = serde_json::Value::Object(value_map_json.into_iter().collect());
        let doc: Document =
            serde_json::from_value(value).map_err(|e| AddDocError::Warning(e.to_string()))?;
        self.documents.push(doc);
        Ok(())
    }
    fn add_account(&mut self, attrs: IndexMap<String, String>) -> Result<(), String> {
        let value_map = &attrs;
        let value = serde_json::Value::Object(
            value_map
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect(),
        );
        let interval: Interval = serde_json::from_value(value)
            .map_err(|e| format!("Ошибка разбора интервала счёта: {e}"))?;
        let number = value_map
            .get("РасчСчет")
            .ok_or_else(|| "Нет поля РасчСчет в секции счёта".to_string())?
            .to_string();
        let key = number.clone();
        match self.accounts.get_mut(&key) {
            Some(account) => {
                if account.intervals.contains(&interval) {
                    return Ok(());
                }
                let pos = account
                    .intervals
                    .binary_search_by_key(&interval.date_start, |i| i.date_start)
                    .unwrap_or_else(|e| e);
                account.intervals.insert(pos, interval);
            }
            None => {
                self.accounts.insert(
                    key,
                    Account {
                        number,
                        intervals: vec![interval],
                    },
                );
            }
        }
        Ok(())
    }
    pub fn add_warning(&mut self, e: (usize, String)) {
        self.warnings.push(e);
    }
}

#[derive(Debug)]
pub enum Error {
    Syntax { lineno: usize, text: String },
    InvalidDocument,
    Unfinished,
    NotText,
    Empty,
}

#[derive(Clone, Copy, Debug)]
pub enum Encoding {
    Cp1251,
    Utf8,
}

#[derive(Default)]
pub struct ParserBuilder {
    section_hooks: Vec<Box<SectionHook>>,
}

impl ParserBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_hooks(mut self, hooks: Vec<Box<SectionHook>>) -> Self {
        self.section_hooks = hooks;
        self
    }
    pub fn build(self) -> Parser {
        Parser {
            section_hooks: self.section_hooks,
        }
    }
}

pub struct Parser {
    section_hooks: Vec<Box<SectionHook>>,
}

impl Default for Parser {
    fn default() -> Self {
        ParserBuilder::new().build()
    }
}

impl Parser {
    pub fn parse(&self, content: &[u8]) -> Result<Statement, Error> {
        let (raw, encoding) = parse_text(content).ok_or(Error::NotText)?;
        match self.parse_internal(&raw, encoding)? {
            ControlFlow::Continue(State::Finished(statement)) => Ok(statement),
            ControlFlow::Break((lineno, text)) => Err(Error::Syntax { lineno, text }),
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
            hook(section, attrs, statement)?
        }
        Ok(())
    }

    fn parse_internal<'a>(
        &self,
        raw: &'a Cow<'a, str>,
        encoding: Encoding,
    ) -> Result<ControlFlow<(usize, String), State<'a>>, Error> {
        Ok(parse_lines(raw)?
            .into_iter()
            .try_fold(State::Init, |state, (lineno, line)| {
                match (state, line) {
                    // Начальное состояние, каждая выписка в первой строке имеет заголовок "1CClientBankExchange"
                    (State::Init, Line::Section(Section::StartOfFile)) => {
                        ControlFlow::Continue(State::Header(Default::default()))
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
                            attrs: Default::default(),
                        })
                    }
                    // Либо СекцияРасчСчет
                    (State::Header(attrs), Line::Section(Section::Account)) => {
                        ControlFlow::Continue(State::Account {
                            statement: Statement::new(encoding, attrs),
                            attrs: Default::default(),
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
                            mut statement,
                            typ,
                            mut attrs,
                        },
                        Line::Section(prev_section @ Section::EndOfDocument),
                    ) => {
                        let section_type = SectionType::Document;
                        match self.call_hooks(section_type, &mut attrs, &statement) {
                            Ok(()) => match statement.add_document(typ, attrs) {
                                Ok(()) => ControlFlow::Continue(State::ReadNextSection {
                                    statement,
                                    prev_section,
                                }),
                                Err(AddDocError::Warning(e)) => {
                                    statement.add_warning((lineno, e));
                                    ControlFlow::Continue(State::ReadNextSection {
                                        statement,
                                        prev_section,
                                    })
                                }
                                Err(AddDocError::Critical(e)) => ControlFlow::Break((
                                    lineno,
                                    format!("Ошибка разбора документа: {e:?}"),
                                )),
                            },
                            Err(HookError::Warning(warn)) => {
                                statement.add_warning((lineno, warn));
                                ControlFlow::Continue(State::ReadNextSection {
                                    statement,
                                    prev_section,
                                })
                            }
                            Err(HookError::Error(err)) => ControlFlow::Break((lineno, err)),
                        }
                    }
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
                    (
                        State::Account {
                            mut attrs,
                            mut statement,
                        },
                        Line::Section(prev_section @ Section::EndOfAccount),
                    ) => {
                        let section_type = SectionType::Account;
                        match self.call_hooks(section_type, &mut attrs, &statement) {
                            Ok(()) => {
                                let res = statement.add_account(attrs);
                                match res {
                                    Ok(()) => ControlFlow::Continue(State::ReadNextSection {
                                        statement,
                                        prev_section,
                                    }),
                                    Err(e) => ControlFlow::Break((lineno, e)),
                                }
                            }
                            Err(HookError::Warning(warn)) => {
                                statement.add_warning((lineno, warn));
                                ControlFlow::Continue(State::ReadNextSection {
                                    statement,
                                    prev_section,
                                })
                            }
                            Err(HookError::Error(err)) => ControlFlow::Break((lineno, err)),
                        }
                    }
                    // Секции документа и счёта заканчиваются соотвествующими секциями: КонецДокумента и КонецРасчСчет
                    // После чего парсер ищет следующую секцию
                    (State::ReadNextSection { statement, .. }, Line::Section(Section::Account)) => {
                        ControlFlow::Continue(State::Account {
                            statement,
                            attrs: Default::default(),
                        })
                    }
                    (
                        State::ReadNextSection { statement, .. },
                        Line::Section(Section::Document(typ)),
                    ) => ControlFlow::Continue(State::Document {
                        statement,
                        typ,
                        attrs: Default::default(),
                    }),
                    (
                        State::ReadNextSection { statement, .. },
                        Line::Section(Section::EndOfFile),
                    ) => ControlFlow::Continue(State::Finished(statement)),
                    (State::ReadNextSection { prev_section, .. }, Line::Section(s)) => {
                        ControlFlow::Break((
                            lineno,
                            format!("Неожиданная секция '{s}' после '{prev_section}'"),
                        ))
                    }
                    (State::ReadNextSection { .. }, Line::Attr(k, v)) => ControlFlow::Break((
                        lineno,
                        format!("Атрибут '{k}={v}' не принадлежит ни одной секции"),
                    )),
                    (State::Init, Line::Attr(k, v)) => ControlFlow::Break((
                        lineno,
                        format!("Неожиданный атрибут '{k}={v}' до секции 1CClientBankExchange"),
                    )),
                    (State::Init, Line::Section(s)) => ControlFlow::Break((
                        lineno,
                        format!("Неожиданная секция '{s}' до секции 1CClientBankExchange"),
                    )),
                    (State::Header(_), Line::Section(s)) => ControlFlow::Break((
                        lineno,
                        format!("Неожиданная секция '{s}' на этапе разбора заголовков"),
                    )),
                    (State::Document { .. }, Line::Section(s)) => ControlFlow::Break((
                        lineno,
                        format!("Неожиданная секция '{s}' на этапе разбора документа"),
                    )),
                    (State::Account { .. }, Line::Section(s)) => ControlFlow::Break((
                        lineno,
                        format!("Неожиданная секция '{s}' на этапе разбора счёта"),
                    )),
                    (State::Finished(_), Line::Attr(k, v)) => ControlFlow::Break((
                        lineno,
                        format!("Неожиданный атрибут '{k}={v}' после секции КонецФайла"),
                    )),
                    (State::Finished(_), Line::Section(s)) => ControlFlow::Break((
                        lineno,
                        format!("Неожиданная секция '{s}' после секции КонецФайла"),
                    )),
                }
            }))
    }
}

fn parse_lines(raw: &str) -> Result<Vec<(usize, Line<'_>)>, Error> {
    raw.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(lineno0, line)| (lineno0 + 1, line))
        .map(|(lineno, line)| {
            Line::try_from(line)
                .map(|l| (lineno, l))
                .map_err(|e| (lineno, e))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|(lineno, e)| Error::Syntax {
            lineno,
            text: format!("cannot parse line: '{e}'"),
        })
}

#[derive(Debug, Clone, Copy)]
pub enum Section<'a> {
    StartOfFile,
    Account,
    EndOfAccount,
    Document(&'a str),
    EndOfDocument,
    EndOfFile,
}

impl<'a> TryFrom<&'a str> for Section<'a> {
    type Error = ();
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match (value.split_once('='), value) {
            (None, "1CClientBankExchange") => Ok(Section::StartOfFile),
            (None, "СекцияРасчСчет") => Ok(Section::Account),
            (None, "КонецРасчСчет") => Ok(Section::EndOfAccount),
            (Some(("СекцияДокумент", typ)), _) => Ok(Section::Document(typ)),
            (None, "КонецДокумента") => Ok(Section::EndOfDocument),
            (None, "КонецФайла") => Ok(Section::EndOfFile),
            _ => Err(()),
        }
    }
}

impl<'a> std::fmt::Display for Section<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Section::StartOfFile => write!(f, "1CClientBankExchange"),
            Section::Account => write!(f, "СекцияРасчСчет"),
            Section::EndOfAccount => write!(f, "КонецРасчСчет"),
            Section::Document(typ) => write!(f, "СекцияДокумент={typ}"),
            Section::EndOfDocument => write!(f, "КонецДокумента"),
            Section::EndOfFile => write!(f, "КонецФайла"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Line<'a> {
    Section(Section<'a>),
    Attr(&'a str, &'a str),
}

impl<'a> TryFrom<&'a str> for Line<'a> {
    type Error = &'a str;
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        if let Ok(v) = Section::try_from(s) {
            return Ok(Self::Section(v));
        }
        match s.split_once('=') {
            Some((k, v)) => Ok(Self::Attr(k, v)),
            None => Err(s),
        }
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
        prev_section: Section<'a>,
    },
    Finished(Statement),
}

/// Парсинг опциональных дат формата "%d.%m.%Y"
/// Пример использования:
fn try_deserialize_dt<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    const FORMAT: &str = "%d.%m.%Y";
    match Option::<String>::deserialize(deserializer)? {
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) => NaiveDate::parse_from_str(&s, FORMAT)
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

/// Парсинг обязательных дат формата "%d.%m.%Y"
/// Пример использования:
fn deserialize_dt<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    const FORMAT: &str = "%d.%m.%Y";
    match Option::<String>::deserialize(deserializer)? {
        Some(s) if s.trim().is_empty() => {
            Err(serde::de::Error::custom("empty string is not a valid date"))
        }
        Some(s) => NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom),
        None => Err(serde::de::Error::custom("missing date field")),
    }
}

enum AddDocError {
    Warning(String),
    Critical(String),
}

fn parse_as_cp1251(v: &[u8]) -> Option<Cow<'_, str>> {
    let (cow, _, had_errors) = WINDOWS_1251.decode(v);
    if had_errors {
        return None;
    }

    let mut total = 0;
    let mut good = 0;

    for c in cow.chars().take(20 * 1024) {
        total += 1;
        if c.is_ascii_alphanumeric()
            || c.is_ascii_whitespace()
            || c.is_ascii_punctuation()
            || c >= '\u{0400}'
        {
            good += 1;
        }
    }

    (total > 0 && good * 100 / total > 95).then_some(cow)
}

fn parse_as_utf8(v: &[u8]) -> Option<&str> {
    std::str::from_utf8(v).ok()
}

fn parse_text(content: &[u8]) -> Option<(Cow<'_, str>, Encoding)> {
    if let Some(s) = parse_as_utf8(content) {
        return Some((Cow::Borrowed(s), Encoding::Utf8));
    }
    if let Some(cow) = parse_as_cp1251(content) {
        return Some((cow, Encoding::Cp1251));
    }
    None
}
