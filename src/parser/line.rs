//! Лексический разбор строк файла: границы секций и атрибуты.

/// Граница секции файла выписки.
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
            (Some((key, typ)), _) if key.trim_end() == "СекцияДокумент" => {
                Ok(Section::Document(typ.trim_start()))
            }
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

/// Одна значащая строка файла: либо граница секции, либо атрибут "Ключ=Значение".
#[derive(Debug, Clone, Copy)]
pub enum Line<'a> {
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
            Some((k, v)) => Ok(Self::Attr(k.trim(), v.trim())),
            None => Err(s),
        }
    }
}

/// Итератор по непустым строкам с номерами (нумерация с 1).
pub fn numbered_lines(raw: &str) -> impl Iterator<Item = (usize, &str)> {
    raw.lines()
        .enumerate()
        .map(|(lineno0, line)| (lineno0 + 1, line.trim()))
        .filter(|(_, line)| !line.is_empty())
}
