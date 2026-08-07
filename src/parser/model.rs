//! Модель данных выписки: заголовок, счета с интервалами и документы.

use chrono::NaiveDate;
use indexmap::IndexMap;
use rust_decimal::Decimal;
use serde::Deserialize;

use super::attrs::SectionAttrs;
use super::de;
use super::de::{deserialize_dt, try_deserialize_dt};
use super::encoding::Encoding;
use super::error::{ParserError, ParserErrorKind, SectionContext};

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

    // --- Поля спецификации 1.01-1.03, добавленные позже основного набора. ---
    // Все они опциональные строки: формат в реальных выгрузках нестрогий, и
    // ошибка в них не должна приводить к потере документа.
    /// `Плательщик2` — расчётный счёт плательщика (строка 2 реквизитов).
    #[serde(rename = "Плательщик2")]
    pub counterparty_2: Option<String>,

    /// `Плательщик3` — третья строка реквизитов плательщика.
    #[serde(rename = "Плательщик3")]
    pub counterparty_3: Option<String>,

    /// `Плательщик4` — четвёртая строка реквизитов плательщика.
    #[serde(rename = "Плательщик4")]
    pub counterparty_4: Option<String>,

    /// `ПлательщикБанк2` — город банка плательщика.
    #[serde(rename = "ПлательщикБанк2")]
    pub counterparty_bank2: Option<String>,

    /// `Получатель1` — наименование получателя.
    #[serde(rename = "Получатель1")]
    pub payee_1: Option<String>,

    /// `Получатель2` — расчётный счёт получателя (строка 2 реквизитов).
    #[serde(rename = "Получатель2")]
    pub payee_2: Option<String>,

    /// `Получатель3` — третья строка реквизитов получателя.
    #[serde(rename = "Получатель3")]
    pub payee_3: Option<String>,

    /// `Получатель4` — четвёртая строка реквизитов получателя.
    #[serde(rename = "Получатель4")]
    pub payee_4: Option<String>,

    /// `ПолучательБанк2` — город банка получателя.
    #[serde(rename = "ПолучательБанк2")]
    pub payee_bank2: Option<String>,

    /// `ВидПлатежа` — вид платежа: "Почтой", "Телеграфом", "Электронно".
    #[serde(rename = "ВидПлатежа")]
    pub payment_kind: Option<String>,

    /// `СрокПлатежа` — срок платежа (аккредитив, платёжное требование).
    #[serde(rename = "СрокПлатежа")]
    pub payment_deadline: Option<String>,

    /// `Код` — уникальный идентификатор платежа (УИН).
    #[serde(rename = "Код")]
    pub uin: Option<String>,

    /// `КодНазПлатежа` — код назначения платежа.
    #[serde(rename = "КодНазПлатежа")]
    pub purpose_code: Option<String>,

    /// `НазначениеПлатежа1` — первая строка многострочного назначения платежа.
    #[serde(rename = "НазначениеПлатежа1")]
    pub purpose_1: Option<String>,

    /// `НазначениеПлатежа2`.
    #[serde(rename = "НазначениеПлатежа2")]
    pub purpose_2: Option<String>,

    /// `НазначениеПлатежа3`.
    #[serde(rename = "НазначениеПлатежа3")]
    pub purpose_3: Option<String>,

    /// `НазначениеПлатежа4`.
    #[serde(rename = "НазначениеПлатежа4")]
    pub purpose_4: Option<String>,

    /// `НазначениеПлатежа5`.
    #[serde(rename = "НазначениеПлатежа5")]
    pub purpose_5: Option<String>,

    /// `НазначениеПлатежа6`.
    #[serde(rename = "НазначениеПлатежа6")]
    pub purpose_6: Option<String>,

    /// `СтатусСоставителя` — статус составителя расчётного документа (поле 101).
    #[serde(rename = "СтатусСоставителя")]
    pub compiler_status: Option<String>,

    /// `ПоказательКБК` — код бюджетной классификации (поле 104).
    #[serde(rename = "ПоказательКБК")]
    pub kbk: Option<String>,

    /// `ОКАТО` — код ОКАТО/ОКТМО (поле 105).
    #[serde(rename = "ОКАТО")]
    pub okato: Option<String>,

    /// `ПоказательОснования` — показатель основания налогового платежа (поле 106).
    #[serde(rename = "ПоказательОснования")]
    pub tax_basis: Option<String>,

    /// `ПоказательПериода` — показатель налогового периода (поле 107).
    #[serde(rename = "ПоказательПериода")]
    pub tax_period: Option<String>,

    /// `ПоказательНомера` — показатель номера документа (поле 108).
    #[serde(rename = "ПоказательНомера")]
    pub tax_number: Option<String>,

    /// `ПоказательДаты` — показатель даты документа (поле 109).
    #[serde(rename = "ПоказательДаты")]
    pub tax_date: Option<String>,

    /// `ПоказательТипа` — показатель типа платежа (поле 110).
    #[serde(rename = "ПоказательТипа")]
    pub tax_type: Option<String>,
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
    #[must_use]
    pub fn new(encoding: Encoding, header: IndexMap<String, String>) -> Self {
        Self {
            encoding,
            header,
            accounts: IndexMap::new(),
            documents: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_warning(&mut self, e: (usize, String)) {
        self.warnings.push(e);
    }

    /// Горячий путь без хуков: атрибуты — срезы входного текста, владеющие
    /// строки аллоцирует только десериализация потреблённых полей.
    pub(super) fn add_document(
        &mut self,
        typ: &str,
        attrs: &SectionAttrs<'_>,
    ) -> Result<(), AddDocError> {
        let doc: Document =
            de::from_borrowed_attrs(attrs.iter().chain(std::iter::once(("СекцияДокумент", typ))))
                .map_err(|e| AddDocError::Warning(e.to_string()))?;
        self.documents.push(doc);
        Ok(())
    }

    /// Путь после хуков: атрибуты уже во владеющей карте контракта хуков.
    pub(super) fn add_document_owned(
        &mut self,
        typ: &str,
        attrs: IndexMap<String, String>,
    ) -> Result<(), AddDocError> {
        let doc_type = ("СекцияДокумент".to_string(), typ.to_string());
        let doc: Document =
            de::from_owned_attrs(attrs.into_iter().chain(std::iter::once(doc_type)))
                .map_err(|e| AddDocError::Warning(e.to_string()))?;
        self.documents.push(doc);
        Ok(())
    }

    /// Горячий путь без хуков: атрибуты — срезы входного текста, владеющие
    /// строки аллоцирует только десериализация потреблённых полей.
    pub(super) fn add_account(
        &mut self,
        attrs: &mut SectionAttrs<'_>,
        lineno: usize,
    ) -> Result<(), ParserError> {
        let interval: Interval =
            de::from_borrowed_attrs(attrs.iter()).map_err(|e| account_parse_error(lineno, &e))?;
        let number = attrs
            .remove("РасчСчет")
            .ok_or_else(|| missing_account_number(lineno))?
            .to_string();
        self.insert_interval(number, interval);
        Ok(())
    }

    /// Путь после хуков: атрибуты уже во владеющей карте контракта хуков.
    pub(super) fn add_account_owned(
        &mut self,
        mut attrs: IndexMap<String, String>,
        lineno: usize,
    ) -> Result<(), ParserError> {
        let interval: Interval =
            de::from_borrowed_attrs(attrs.iter().map(|(k, v)| (k.as_str(), v.as_str())))
                .map_err(|e| account_parse_error(lineno, &e))?;
        let number = attrs
            .shift_remove("РасчСчет")
            .ok_or_else(|| missing_account_number(lineno))?;
        self.insert_interval(number, interval);
        Ok(())
    }

    fn insert_interval(&mut self, number: String, interval: Interval) {
        match self.accounts.get_mut(&number) {
            Some(account) => {
                // Интервалы отсортированы по дате начала, поэтому дубликат может
                // находиться только в непрерывном блоке с той же датой.
                let run_start = account
                    .intervals
                    .partition_point(|i| i.date_start < interval.date_start);
                let run = account
                    .intervals
                    .iter()
                    .skip(run_start)
                    .take_while(|i| i.date_start == interval.date_start);
                let mut run_len = 0;
                let mut is_duplicate = false;
                for existing in run {
                    if *existing == interval {
                        is_duplicate = true;
                        break;
                    }
                    run_len += 1;
                }
                if !is_duplicate {
                    account.intervals.insert(run_start + run_len, interval);
                }
            }
            None => {
                self.accounts.insert(
                    number.clone(),
                    Account {
                        number,
                        intervals: vec![interval],
                    },
                );
            }
        }
    }
}

fn account_parse_error(lineno: usize, e: &impl std::fmt::Display) -> ParserError {
    ParserError {
        lineno,
        kind: ParserErrorKind::AccountParseError(e.to_string()),
    }
}

fn missing_account_number(lineno: usize) -> ParserError {
    ParserError {
        lineno,
        kind: ParserErrorKind::MissingField {
            field: "РасчСчет".to_string(),
            context: SectionContext::Account,
        },
    }
}

pub(super) enum AddDocError {
    Warning(String),
}
