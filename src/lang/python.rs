//! Python language support.

use super::Language;
use tree_sitter::Language as TsLanguage;

/// Python programming language.
pub struct Python;

impl Language for Python {
    fn name(&self) -> &'static str {
        "python"
    }

    fn extensions(&self) -> &[&'static str] {
        &["py", "pyi"]
    }

    fn grammar(&self) -> TsLanguage {
        tree_sitter_python::LANGUAGE.into()
    }
}
