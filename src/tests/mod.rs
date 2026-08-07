// why: тестовому коду разрешены unwrap/expect/индексация и пр. — паника в тесте
// и есть сигнал провала, а не скрытая ошибка продакшен-кода.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::format_collect,
    clippy::similar_names
)]

mod anonymizer;
mod hooks;
mod run_suites;
