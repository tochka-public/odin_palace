use indexmap::IndexMap;
use odin_palace::parser::hooks::{HookError, SectionType};
use odin_palace::parser::{ParserBuilder, Statement};

const INPUT: &str = "1CClientBankExchange\n\
                     ВерсияФормата=1.03\n\
                     СекцияРасчСчет\n\
                     РасчСчет=40702810000000000333\n\
                     ДатаНачала=01.02.2024\n\
                     НачальныйОстаток=10.00\n\
                     КонецРасчСчет\n\
                     СекцияДокумент=Платежное поручение\n\
                     Номер=1\n\
                     Дата=01.02.2024\n\
                     Сумма=100.50\n\
                     ПлательщикИНН=7707083893\n\
                     ПлательщикБИК=044525225\n\
                     ПлательщикБанк1=БАНК\n\
                     ПлательщикСчет=40702810000000000333\n\
                     ПолучательИНН=7727406020\n\
                     ПолучательБИК=017003983\n\
                     ПолучательБанк1=БАНК2\n\
                     ПолучательСчет=40802810900000000444\n\
                     НазначениеПлатежа=оплата\n\
                     КонецДокумента\n\
                     КонецФайла";

type HookFn =
    dyn Fn(SectionType, &mut IndexMap<String, String>, &Statement) -> Result<(), HookError>;

fn parse_with_hook(hook: Box<HookFn>) -> Result<Statement, odin_palace::parser::Error> {
    ParserBuilder::new()
        .with_hooks(vec![hook])
        .build()
        .parse(INPUT.as_bytes())
}

/// Хук может модифицировать атрибуты секции до десериализации.
#[test]
fn hook_modifies_document_attrs() {
    let result = parse_with_hook(Box::new(|section, attrs, _statement| {
        if section == SectionType::Document {
            attrs.insert("НазначениеПлатежа".into(), "ИЗМЕНЕНО".into());
        }
        Ok(())
    }));
    let statement = result.expect("statement must parse");
    assert_eq!(
        statement.documents.len(),
        1,
        "document must survive the hook"
    );
    assert_eq!(statement.documents[0].purpose, "ИЗМЕНЕНО");
}

/// Warning из хука попадает в warnings, а секция пропускается.
#[test]
fn hook_warning_skips_section() {
    let result = parse_with_hook(Box::new(|section, _attrs, _statement| {
        if section == SectionType::Document {
            Err(HookError::Warning("подозрительный документ".into()))
        } else {
            Ok(())
        }
    }));
    let statement = result.expect("statement must parse");
    insta::assert_debug_snapshot!((statement.documents, statement.warnings));
}

/// Error из хука прерывает разбор с `Error::Syntax(HookError)`.
#[test]
fn hook_error_stops_parsing() {
    let result = parse_with_hook(Box::new(|_section, _attrs, _statement| {
        Err(HookError::Error("критическая ошибка".into()))
    }));
    insta::assert_debug_snapshot!(result);
}

/// Хук счёта видит заголовок выписки.
#[test]
fn hook_sees_header() {
    let result = parse_with_hook(Box::new(|_section, _attrs, statement| {
        match statement.header.get("ВерсияФормата").map(String::as_str) {
            Some("1.03") => Ok(()),
            other => Err(HookError::Error(format!("unexpected header: {other:?}"))),
        }
    }));
    assert!(result.is_ok(), "hook must see header attributes");
}
