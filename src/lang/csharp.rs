//! C# language support.

use super::Language;
use tree_sitter::Language as TsLanguage;

/// C# programming language.
pub struct CSharp;

impl Language for CSharp {
    fn name(&self) -> &'static str {
        "csharp"
    }

    fn extensions(&self) -> &[&'static str] {
        &["cs", "csx"]
    }

    fn grammar(&self) -> TsLanguage {
        tree_sitter_c_sharp::LANGUAGE.into()
    }
}
