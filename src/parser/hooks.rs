use crate::parser::Statement;
use indexmap::IndexMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionType {
    Document,
    Account,
}

#[derive(Debug)]
pub enum HookError {
    Warning(String),
    Error(String),
}

pub type SectionHook =
    dyn Fn(SectionType, &mut IndexMap<String, String>, &Statement) -> Result<(), HookError>;
