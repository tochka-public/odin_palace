//! Анонимизация выписок 1CClientBankExchange.
//!
//! Заменяет персональные данные (ФИО, счета, ИНН/КПП, назначения платежей)
//! на шаблонные значения, сохраняя структуру файла и согласованность замен:
//! одинаковые исходные значения всегда получают одинаковую замену.

use std::collections::HashMap;

const FIO_TEMPLATES: &[&str] = &[
    "Иванов Иван Иванович",
    "Петров Петр Петрович",
    "Сидоров Сидор Сидорович",
    "Петрова Галина Леонидовна",
    "Кузнецова Мария Сергеевна",
    "Смирнов Алексей Алексеевич",
    "Васильева Ольга Николаевна",
    "Морозов Дмитрий Сергеевич",
    "Попова Наталья Владимировна",
    "Волков Андрей Владимирович",
];

const PURPOSE_TEMPLATES: &[&str] = &[
    "Оплата по договору",
    "Перевод средств",
    "Тестовая операция",
    "Услуги",
    "Платеж за услуги",
    "Погашение задолженности",
    "Возврат средств",
    "Тестовый платеж",
    "Авансовый платеж",
    "Прочие операции",
];

const PAYER_KEYS: &[&str] = &["Плательщик", "Плательщик1"];
const PAYEE_KEYS: &[&str] = &["Получатель", "Получатель1"];

/// Сохраняемый префикс номера счёта: балансовая часть, например "40702".
const ACCOUNT_PREFIX_CHARS: usize = 5;
/// Сохраняемый префикс ИНН/КПП: код региона.
const ID_PREFIX_CHARS: usize = 2;

#[derive(Clone, Copy)]
enum FioRole {
    Payer,
    Payee,
}

fn fio_role(key: &str) -> Option<FioRole> {
    if PAYER_KEYS.contains(&key) {
        Some(FioRole::Payer)
    } else if PAYEE_KEYS.contains(&key) {
        Some(FioRole::Payee)
    } else {
        None
    }
}

fn is_account_key(key: &str) -> bool {
    key.contains("Счет") || key.contains("Корсчет")
}

fn is_id_key(key: &str) -> bool {
    key.contains("ИНН") || key.contains("КПП")
}

fn is_purpose_key(key: &str) -> bool {
    let lower = key.to_lowercase();
    lower.contains("назначение") || lower.contains("purpose") || lower.contains("description")
}

/// Заменяет хвост значения порядковым номером, сохраняя первые `prefix_chars`
/// символов. Срез идёт по границам символов, поэтому не-ASCII значения безопасны.
fn mask_with_prefix(value: &str, prefix_chars: usize, index: usize) -> String {
    match value.char_indices().nth(prefix_chars) {
        Some((byte_idx, _)) => {
            let masked_chars = value.chars().count() - prefix_chars;
            let prefix = value.get(..byte_idx).unwrap_or(value);
            format!("{prefix}{index:0masked_chars$}")
        }
        None => format!("{value}{index}"),
    }
}

#[derive(Default)]
struct Mappings {
    fio: HashMap<String, String>,
    purpose: HashMap<String, String>,
    account: HashMap<String, String>,
    id: HashMap<String, String>,
}

impl Mappings {
    fn collect(input: &str) -> Self {
        let mut mappings = Self::default();
        let mut payer_num = 1usize;
        let mut payee_num = 1usize;
        let mut fio_idx = 0usize;
        let mut purpose_idx = 0usize;
        let mut account_idx = 1usize;
        let mut id_idx = 1usize;

        for line in input.split('\n') {
            let line = line.strip_suffix('\r').unwrap_or(line);
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let (key, value) = (key.trim(), value.trim());
            if value.is_empty() {
                continue;
            }

            if let Some(role) = fio_role(key) {
                if !mappings.fio.contains_key(value) {
                    let replacement = match FIO_TEMPLATES.get(fio_idx) {
                        Some(template) => (*template).to_string(),
                        None => match role {
                            FioRole::Payer => {
                                let t = format!("Плательщик_{payer_num}");
                                payer_num += 1;
                                t
                            }
                            FioRole::Payee => {
                                let t = format!("Получатель_{payee_num}");
                                payee_num += 1;
                                t
                            }
                        },
                    };
                    fio_idx += 1;
                    mappings.fio.insert(value.to_string(), replacement);
                }
            } else if is_account_key(key) {
                if !mappings.account.contains_key(value) {
                    let masked = mask_with_prefix(value, ACCOUNT_PREFIX_CHARS, account_idx);
                    mappings.account.insert(value.to_string(), masked);
                    account_idx += 1;
                }
            } else if is_id_key(key) {
                if !mappings.id.contains_key(value) {
                    let masked = mask_with_prefix(value, ID_PREFIX_CHARS, id_idx);
                    mappings.id.insert(value.to_string(), masked);
                    id_idx += 1;
                }
            } else if is_purpose_key(key) && !mappings.purpose.contains_key(value) {
                let template = PURPOSE_TEMPLATES
                    .get(purpose_idx % PURPOSE_TEMPLATES.len())
                    .copied()
                    .unwrap_or_default();
                mappings
                    .purpose
                    .insert(value.to_string(), template.to_string());
                purpose_idx += 1;
            }
        }
        mappings
    }

    /// Возвращает замену для пары ключ-значение, если ключ относится
    /// к анонимизируемой категории и значение известно.
    fn replacement(&self, key: &str, value: &str) -> Option<&str> {
        let map = if fio_role(key).is_some() {
            &self.fio
        } else if is_account_key(key) {
            &self.account
        } else if is_id_key(key) {
            &self.id
        } else if is_purpose_key(key) {
            &self.purpose
        } else {
            return None;
        };
        map.get(value).map(String::as_str)
    }
}

pub fn anonymize_str(input: &str) -> String {
    let mappings = Mappings::collect(input);

    let mut out = String::with_capacity(input.len());
    let mut first = true;
    for line in input.split('\n') {
        if !first {
            out.push('\n');
        }
        first = false;

        let (body, has_cr) = match line.strip_suffix('\r') {
            Some(body) => (body, true),
            None => (line, false),
        };

        match body.split_once('=') {
            Some((key, value)) => match mappings.replacement(key.trim(), value.trim()) {
                Some(new_value) => {
                    out.push_str(key);
                    out.push('=');
                    out.push_str(new_value);
                }
                None => out.push_str(body),
            },
            None => out.push_str(body),
        }

        if has_cr {
            out.push('\r');
        }
    }
    out
}
