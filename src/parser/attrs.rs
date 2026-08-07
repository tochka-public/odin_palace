//! Накопитель атрибутов секции.
//!
//! И ключи, и значения — срезы входного текста: до конца секции ничего не
//! аллоцируется. Владеющие `String` появляются только там, где они нужны:
//! внутри десериализации для строковых полей модели (одна аллокация на
//! потреблённое поле) либо при конверсии в `IndexMap` для хуков.
//!
//! Семантика вставки повторяет `IndexMap::insert`: у повторного ключа
//! сохраняется позиция первого вхождения, а значение берётся последнее.
//! Атрибутов в секции десятки, поэтому линейный поиск по `Vec` быстрее
//! хеширования.

use indexmap::IndexMap;

#[derive(Debug)]
pub(super) struct SectionAttrs<'a> {
    entries: Vec<(&'a str, &'a str)>,
}

impl<'a> SectionAttrs<'a> {
    /// Типичный документ содержит около трёх десятков атрибутов.
    const TYPICAL_SECTION_ATTRS: usize = 32;

    pub(super) fn new() -> Self {
        Self {
            entries: Vec::with_capacity(Self::TYPICAL_SECTION_ATTRS),
        }
    }

    pub(super) fn insert(&mut self, key: &'a str, value: &'a str) {
        match self.entries.iter_mut().find(|(k, _)| *k == key) {
            Some((_, existing)) => *existing = value,
            None => self.entries.push((key, value)),
        }
    }

    /// Забирает значение по ключу, сохраняя порядок остальных атрибутов.
    pub(super) fn remove(&mut self, key: &str) -> Option<&'a str> {
        let pos = self.entries.iter().position(|(k, _)| *k == key)?;
        Some(self.entries.remove(pos).1)
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = (&'a str, &'a str)> {
        self.entries.iter().copied()
    }

    /// Конверсия для пути с хуками: их публичный контракт —
    /// `&mut IndexMap<String, String>`.
    pub(super) fn into_index_map(self) -> IndexMap<String, String> {
        self.entries
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }
}
