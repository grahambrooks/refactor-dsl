//! Go language support.

use super::Language;
use tree_sitter::Language as TsLanguage;

/// Go programming language.
pub struct Go;

impl Language for Go {
    fn name(&self) -> &'static str {
        "go"
    }

    fn extensions(&self) -> &[&'static str] {
        &["go"]
    }

    fn grammar(&self) -> TsLanguage {
        tree_sitter_go::LANGUAGE.into()
    }
}
