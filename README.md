# Чертоги Одина aka odin_palace

[![crates.io](https://img.shields.io/crates/v/odin_palace.svg)](https://crates.io/crates/odin_palace)
[![docs.rs](https://img.shields.io/docsrs/odin_palace)](https://docs.rs/odin_palace)
[![CI](https://github.com/tochka-public/odin_palace/actions/workflows/checks.yml/badge.svg)](https://github.com/tochka-public/odin_palace/actions/workflows/checks.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

odin_palace — это инструмент для парсинга и анализа банковских выписок в формате 1CClientBankExchange.

Возможности:

- разбор выписок версий формата 1.01-1.03 в типизированную структуру
  (`Statement` -> счета с интервалами, документы, заголовок, предупреждения);
- автоматическое определение кодировки (UTF-8 и CP1251, UTF-8 BOM допускается);
- hooks для модификации атрибутов секций до десериализации;
- анонимизация выписок (CLI-команда `anon`);
- Python-биндинги: [odin_palace_py](https://github.com/tochka-public/odin_palace_py).

[Спецификация 1CClientBankExchange v1.03](https://v8.1c.ru/tekhnologii/obmen-dannymi-i-integratsiya/standarty-i-formaty/standart-obmena-s-sistemami-klient-banka/formaty-obmena/)

## Установка

Установка с crates.io:

```bash
cargo install odin_palace
```

Установка последней версии из репозитория:

```bash
cargo install --git https://github.com/tochka-public/odin_palace
```

## Использование CLI

Для разбора банковской выписки предусмотрена команда:

```bash
odin_palace parse путь/к/файлу.txt
```

где `путь/к/файлу.txt` — путь к файлу выписки в формате 1CClientBankExchange. Результатом выполнения является структурированное представление выписки, выводимое в стандартный вывод (stdout).

Для анонимизации выписки (ФИО, счета, ИНН/КПП и назначения платежей заменяются
согласованными шаблонными значениями):

```bash
odin_palace anon путь/к/файлу.txt > анонимная_выписка.txt
```

## Пример входного файла

```
1CClientBankExchange
ВерсияФормата=1.02
Кодировка=Windows
Отправитель=Tinkoff
Получатель=Иванов Иван Иванович
ДатаСоздания=16.04.2024
ВремяСоздания=15:36:40
ДатаНачала=01.04.2024
ДатаКонца=16.04.2024
РасчСчет=40802000000000000007
СекцияРасчСчет
ДатаНачала=01.04.2024
ДатаКонца=16.04.2024
РасчСчет=40802000000000000007
НачальныйОстаток=0
ВсегоПоступило=67770
ВсегоСписано=0
КонечныйОстаток=67770
КонецРасчСчет
СекцияДокумент=Банковский ордер
Номер=481554
Дата=13.04.2024
Сумма=1000
ДатаСписано=13.04.2024
Плательщик=Петров Петр Петрович
Плательщик1=Петров Петр Петрович
ПлательщикСчет=30233000000000000004
ПлательщикИНН=7700000001
ПлательщикРасчСчет=30233000000000000004
ПлательщикКорсчет=30101000000000000005
ПлательщикБИК=044525593
ПлательщикБанк1=АО "АЛЬФА-БАНК"
Получатель=Сидоров Сидор Сидорович
Получатель1=Сидоров Сидор Сидорович
ПолучательСчет=40802000000000000007
ПолучательИНН=260000000002
ПолучательРасчСчет=40802000000000000007
ПолучательКорсчет=30101000000000000008
ПолучательБИК=044525974
ПолучательБанк1=АО "ТИНЬКОФФ БАНК"
ПолучательКПП=
ВидОплаты=17
НазначениеПлатежа=Оплата по договору
Очередность=5
КонецДокумента
КонецФайла
```

## Использование как библиотеки

Для использования в качестве библиотеки добавьте зависимость:

```bash
cargo add odin_palace
```

Либо последнюю версию из репозитория в `Cargo.toml`:

```toml
odin_palace = { git = "https://github.com/tochka-public/odin_palace" }
```

Пример использования:

```rust
use odin_palace::parser::Parser;

let content = std::fs::read("input.txt")?;
let statement = Parser::default().parse(&content)?;
for doc in &statement.documents {
    println!("#{} {} {}: {}", doc.doc_number, doc.doc_date, doc.amount, doc.purpose);
}
```

С hooks (модификация атрибутов секции до десериализации):

```rust
use odin_palace::parser::ParserBuilder;
use odin_palace::parser::hooks::SectionType;

let parser = ParserBuilder::new()
    .with_hooks(vec![Box::new(|section, attrs, _statement| {
        if section == SectionType::Document {
            attrs.shift_remove("НеизвестноеПоле");
        }
        Ok(())
    })])
    .build();
```

## Разработка

```bash
cargo test                                  # тесты (insta + rstest, сьюты в src/tests/suites)
cargo clippy --all-targets --all-features   # линтер
cargo bench                                 # criterion-бенчмарк
cargo run --release --example alloc_stats   # количество/объём аллокаций на разбор
```
