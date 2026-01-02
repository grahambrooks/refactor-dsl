//! Ruby language support.

use super::Language;
use tree_sitter::Language as TsLanguage;

/// Ruby programming language.
pub struct Ruby;

impl Language for Ruby {
    fn name(&self) -> &'static str {
        "ruby"
    }

    fn extensions(&self) -> &[&'static str] {
        &["rb", "rake", "gemspec"]
    }

    fn grammar(&self) -> TsLanguage {
        tree_sitter_ruby::LANGUAGE.into()
    }
}
