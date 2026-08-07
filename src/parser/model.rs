//! Модель данных выписки: заголовок, счета с интервалами и документы.

use chrono::NaiveDate;
use indexmap::IndexMap;
use rust_decimal::Decimal;
use serde::Deserialize;

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

    pub(super) fn add_account(
        &mut self,
        attrs: IndexMap<String, String>,
        lineno: usize,
    ) -> Result<(), ParserError> {
        let value_map = &attrs;
        let value = serde_json::Value::Object(
            value_map
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect(),
        );
        let interval: Interval = serde_json::from_value(value).map_err(|e| ParserError {
            lineno,
            kind: ParserErrorKind::AccountParseError(e.to_string()),
        })?;
        let number = value_map
            .get("РасчСчет")
            .ok_or_else(|| ParserError {
                lineno,
                kind: ParserErrorKind::MissingField {
                    field: "РасчСчет".to_string(),
                    context: SectionContext::Account,
                },
            })?
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
}

pub(super) enum AddDocError {
    Warning(String),
}
