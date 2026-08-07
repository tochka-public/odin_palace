//! Определение кодировки входного файла (UTF-8 или CP1251).

use std::borrow::Cow;

use encoding_rs::WINDOWS_1251;

/// Кодировка, в которой был прочитан входной файл.
#[derive(Clone, Copy, Debug)]
pub enum Encoding {
    Cp1251,
    Utf8,
}

/// Декодирует входные байты и проверяет заголовок формата.
///
/// Возвращает `None`, если входные данные не декодируются ни одной из
/// поддерживаемых кодировок или не начинаются с "1CClientBankExchange".
pub(super) fn parse_text(content: &[u8]) -> Option<(Cow<'_, str>, Encoding)> {
    // Выгрузки из 1С часто начинаются с UTF-8 BOM — он не является частью формата.
    let content = content.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(content);
    if let Some(s) = parse_as_utf8(content) {
        let first_line = s.lines().next().unwrap_or("");
        if !is_1c_header_line(first_line) {
            return None;
        }
        return Some((Cow::Borrowed(s), Encoding::Utf8));
    }
    if let Some(cow) = parse_as_cp1251(content) {
        let first_line = cow.lines().next().unwrap_or("");
        if !is_1c_header_line(first_line) {
            return None;
        }
        return Some((cow, Encoding::Cp1251));
    }
    None
}

fn is_1c_header_line(line: &str) -> bool {
    line.trim() == "1CClientBankExchange"
}

fn parse_as_utf8(v: &[u8]) -> Option<&str> {
    std::str::from_utf8(v).ok()
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
