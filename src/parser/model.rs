//! Модель данных выписки: заголовок, счета с интервалами и документы.

use chrono::NaiveDate;
use indexmap::IndexMap;
use rust_decimal::Decimal;
use serde::Deserialize;

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

    pub fn add_warning(&mut self, e: (usize, String)) {
        self.warnings.push(e);
    }

    pub(super) fn add_document(
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

    pub(super) fn add_account(
        &mut self,
        mut attrs: IndexMap<String, String>,
        lineno: usize,
    ) -> Result<(), ParserError> {
        let interval: Interval = de::from_borrowed_attrs(
            attrs.iter().map(|(k, v)| (k.as_str(), v.as_str())),
        )
        .map_err(|e| ParserError {
            lineno,
            kind: ParserErrorKind::AccountParseError(e.to_string()),
        })?;
        let number = attrs.shift_remove("РасчСчет").ok_or_else(|| ParserError {
            lineno,
            kind: ParserErrorKind::MissingField {
                field: "РасчСчет".to_string(),
                context: SectionContext::Account,
            },
        })?;
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
        Ok(())
    }
}

pub(super) enum AddDocError {
    Warning(String),
}
