use crate::core::lang::Language;

#[derive(Debug, PartialEq)]
pub struct CrossIdentifier {
    language: Language,
    raw: String,
}
