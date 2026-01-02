//! Java language support.

use super::Language;
use tree_sitter::Language as TsLanguage;

/// Java programming language.
pub struct Java;

impl Language for Java {
    fn name(&self) -> &'static str {
        "java"
    }

    fn extensions(&self) -> &[&'static str] {
        &["java"]
    }

    fn grammar(&self) -> TsLanguage {
        tree_sitter_java::LANGUAGE.into()
    }
}
