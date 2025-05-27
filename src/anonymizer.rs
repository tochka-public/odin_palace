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

pub fn anonymize_str(input: &str) -> String {
    let mut fio_map = std::collections::HashMap::new();
    let mut purpose_map = std::collections::HashMap::new();
    let mut account_map = std::collections::HashMap::new();
    let mut id_map = std::collections::HashMap::new();
    let mut payer_num = 1;
    let mut payee_num = 1;
    let mut fio_idx = 0;
    let mut purpose_idx = 0;
    let mut account_idx = 1;
    let mut id_idx = 1;
    let mut lines = Vec::new();

    let payer_keys = ["Плательщик", "Плательщик1"];
    let payee_keys = ["Получатель", "Получатель1"];

    for line in input.split('\n') {
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim();
            if payer_keys.iter().any(|k| key.trim() == *k) && !fio_map.contains_key(value) {
                let templ = if fio_idx < FIO_TEMPLATES.len() {
                    FIO_TEMPLATES[fio_idx]
                } else {
                    let t = format!("Плательщик_{payer_num}");
                    payer_num += 1;
                    fio_map.insert(value.to_string(), t.clone());
                    fio_idx += 1;
                    continue;
                };
                fio_map.insert(value.to_string(), templ.to_string());
                fio_idx += 1;
            } else if payee_keys.iter().any(|k| key.trim() == *k) && !fio_map.contains_key(value) {
                let templ = if fio_idx < FIO_TEMPLATES.len() {
                    FIO_TEMPLATES[fio_idx]
                } else {
                    let t = format!("Получатель_{payee_num}");
                    payee_num += 1;
                    fio_map.insert(value.to_string(), t.clone());
                    fio_idx += 1;
                    continue;
                };
                fio_map.insert(value.to_string(), templ.to_string());
                fio_idx += 1;
            } else if key.contains("Счет")
                || key.contains("Корсчет") && !account_map.contains_key(value)
            {
                let prefix = &value[..std::cmp::min(5, value.len())];
                let num_len = value.len().saturating_sub(5);
                let new_num = format!("{account_idx:0num_len$}");
                let new_value = format!("{prefix}{new_num}");
                account_map.insert(value.to_string(), new_value);
                account_idx += 1;
            } else if key.contains("ИНН") || key.contains("КПП") && !id_map.contains_key(value)
            {
                let prefix = &value[..std::cmp::min(2, value.len())];
                let num_len = value.len().saturating_sub(2);
                let new_num = format!("{id_idx:0num_len$}");
                let new_value = format!("{prefix}{new_num}");
                id_map.insert(value.to_string(), new_value);
                id_idx += 1;
            } else if (key.to_lowercase().contains("purpose")
                || key.to_lowercase().contains("назначение")
                || key.to_lowercase().contains("description"))
                && !purpose_map.contains_key(value)
            {
                let templ = PURPOSE_TEMPLATES
                    .get(purpose_idx % PURPOSE_TEMPLATES.len())
                    .unwrap();
                purpose_map.insert(value.to_string(), templ.to_string());
                purpose_idx += 1;
            }
        }
        lines.push(line);
    }

    let mut out = String::with_capacity(input.len());
    for (i, line) in lines.iter().enumerate() {
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim();
            let new_value = if payer_keys.iter().any(|k| key.trim() == *k)
                || payee_keys.iter().any(|k| key.trim() == *k)
            {
                fio_map.get(value).map(|s| s.as_str()).unwrap_or(value)
            } else if key.contains("Счет") || key.contains("Корсчет") {
                account_map.get(value).map(|s| s.as_str()).unwrap_or(value)
            } else if key.contains("ИНН") || key.contains("КПП") {
                id_map.get(value).map(|s| s.as_str()).unwrap_or(value)
            } else if let Some(new) = purpose_map.get(value) {
                new
            } else {
                value
            };
            out.push_str(key);
            out.push('=');
            out.push_str(new_value);
        } else {
            out.push_str(line);
        }
        if i + 1 < lines.len() {
            out.push('\n');
        }
    }
    out
}
