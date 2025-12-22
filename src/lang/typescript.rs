//! TypeScript and JavaScript language support.

use super::Language;
use tree_sitter::Language as TsLanguage;

/// TypeScript programming language.
pub struct TypeScript;

impl Language for TypeScript {
    fn name(&self) -> &'static str {
        "typescript"
    }

    fn extensions(&self) -> &[&'static str] {
        &["ts", "tsx", "js", "jsx", "mjs", "cjs"]
    }

    fn grammar(&self) -> TsLanguage {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
    }
}
