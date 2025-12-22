//! Rust language support.

use super::Language;
use tree_sitter::Language as TsLanguage;

/// Rust programming language.
pub struct Rust;

impl Language for Rust {
    fn name(&self) -> &'static str {
        "rust"
    }

    fn extensions(&self) -> &[&'static str] {
        &["rs"]
    }

    fn grammar(&self) -> TsLanguage {
        tree_sitter_rust::LANGUAGE.into()
    }
}
