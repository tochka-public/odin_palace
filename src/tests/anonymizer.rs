use crate::anonymizer::anonymize_str;
use rstest::rstest;

/// Одно и то же значение счёта во всех ключах должно заменяться одинаково,
/// а разные значения — по-разному.
#[test]
fn consistent_account_mapping() {
    let input = "ПлательщикСчет=40702810000000000001\n\
                 ПолучательСчет=40702810000000000002\n\
                 РасчСчет=40702810000000000001";
    let out = anonymize_str(input);
    let lines: Vec<&str> = out.lines().collect();
    let payer = lines[0].split_once('=').unwrap().1;
    let payee = lines[1].split_once('=').unwrap().1;
    let ras = lines[2].split_once('=').unwrap().1;
    assert_eq!(
        payer, ras,
        "same source account must map to same replacement"
    );
    assert_ne!(
        payer, payee,
        "different accounts must map to different replacements"
    );
    assert_ne!(payer, "40702810000000000001", "account must be replaced");
}

/// ИНН/КПП: известное значение не должно перезаписываться новым индексом.
#[test]
fn consistent_id_mapping() {
    let input = "ПлательщикИНН=7707083893\nПолучательИНН=7707083893";
    let out = anonymize_str(input);
    let lines: Vec<&str> = out.lines().collect();
    let first = lines[0].split_once('=').unwrap().1;
    let second = lines[1].split_once('=').unwrap().1;
    assert_eq!(first, second, "same INN must map to same replacement");
    assert_ne!(first, "7707083893", "INN must be replaced");
}

/// Не-ASCII значение в ключе со "Счет" не должно приводить к панике
/// (срез должен идти по границам символов, а не байтов).
#[rstest]
#[case::cyrillic("СчетКомментарий=абвгдежзик")]
#[case::short_cyrillic("Счет=абв")]
#[case::mixed("РасчСчет=аб12345678")]
fn non_ascii_value_does_not_panic(#[case] input: &str) {
    let _ = anonymize_str(input);
}

/// Строки не должны теряться, когда уникальных ФИО больше, чем шаблонов.
#[test]
fn no_lines_dropped_with_many_payers() {
    let input: String = (0..15)
        .map(|i| format!("Плательщик=Персона Номер {i}\n"))
        .collect::<String>()
        + "КонецДокумента";
    let out = anonymize_str(&input);
    assert_eq!(
        out.lines().count(),
        input.lines().count(),
        "anonymization must not drop lines"
    );
}

/// Замена назначения платежа не должна задевать другие ключи с тем же значением.
#[test]
fn purpose_replacement_respects_key() {
    let input = "НазначениеПлатежа=Аренда офиса\nПримечание=Аренда офиса";
    let out = anonymize_str(input);
    let lines: Vec<&str> = out.lines().collect();
    let purpose = lines[0].split_once('=').unwrap().1;
    let note = lines[1].split_once('=').unwrap().1;
    assert_ne!(purpose, "Аренда офиса", "purpose must be replaced");
    assert_eq!(note, "Аренда офиса", "non-purpose key must keep its value");
}

/// CRLF-переводы строк сохраняются как есть.
#[test]
fn crlf_preserved() {
    let input = "Плательщик=Иванов И И\r\nСумма=100.00\r\n";
    let out = anonymize_str(input);
    assert!(out.contains("\r\n"), "CRLF endings must be preserved");
    assert_eq!(out.matches("\r\n").count(), 2);
}

/// Снапшот полного прохода по синтетической выписке.
#[test]
fn snapshot_full_statement() {
    let input = "1CClientBankExchange\n\
                 ВерсияФормата=1.03\n\
                 СекцияРасчСчет\n\
                 РасчСчет=40702810000000000333\n\
                 КонецРасчСчет\n\
                 СекцияДокумент=Платежное поручение\n\
                 Номер=1\n\
                 Дата=01.02.2024\n\
                 Сумма=100.50\n\
                 Плательщик=ООО Ромашка\n\
                 Плательщик1=ООО Ромашка\n\
                 ПлательщикИНН=7707083893\n\
                 ПлательщикКПП=770701001\n\
                 ПлательщикСчет=40702810000000000333\n\
                 ПлательщикКорсчет=30101810400000000225\n\
                 Получатель=ИП Васильев\n\
                 ПолучательИНН=503201543211\n\
                 ПолучательСчет=40802810900000000444\n\
                 НазначениеПлатежа=Оплата аренды за январь\n\
                 КонецДокумента\n\
                 КонецФайла";
    insta::assert_snapshot!(anonymize_str(input));
}
