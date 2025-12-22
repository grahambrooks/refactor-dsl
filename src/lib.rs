//! # Refactor DSL
//!
//! A domain-specific language for multi-language code refactoring with Git-aware matching.
//!
//! This crate provides a fluent API for:
//! - Matching repositories by Git state (branch, commits, remotes)
//! - Matching files by extension, glob patterns, and content
//! - Matching code using AST queries (tree-sitter)
//! - Transforming code with text patterns or AST-aware operations
//! - Restructuring files (move, rename, delete)
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use refactor_dsl::prelude::*;
//!
//! // Replace all .unwrap() calls with .expect() in Rust files
//! let result = Refactor::in_repo("./my-project")
//!     .matching(|m| m
//!         .git(|g| g.branch("main"))
//!         .files(|f| f.extension("rs").exclude("**/target/**")))
//!     .transform(|t| t
//!         .replace_pattern(r"\.unwrap\(\)", ".expect(\"TODO\")"))
//!     .dry_run()
//!     .apply()?;
//!
//! println!("{}", result.diff());
//! # Ok::<(), refactor_dsl::error::RefactorError>(())
//! ```
//!
//! ## Repository Matching
//!
//! ```rust,no_run
//! use refactor_dsl::prelude::*;
//!
//! // Match repos on main branch with recent commits
//! let result = Refactor::in_repo("./project")
//!     .matching(|m| m
//!         .git(|g| g
//!             .branch("main")
//!             .has_file("Cargo.toml")
//!             .recent_commits(30)))
//!     .transform(|t| t.replace_literal("old_api", "new_api"))
//!     .apply()?;
//! # Ok::<(), refactor_dsl::error::RefactorError>(())
//! ```
//!
//! ## AST-based Matching
//!
//! ```rust,no_run
//! use refactor_dsl::prelude::*;
//!
//! // Find all function definitions in Rust code
//! let matcher = AstMatcher::new()
//!     .query("(function_item name: (identifier) @fn_name)");
//!
//! let matches = matcher.find_matches(
//!     "fn hello() {} fn world() {}",
//!     &Rust,
//! )?;
//!
//! for m in matches {
//!     println!("Found function: {}", m.text);
//! }
//! # Ok::<(), refactor_dsl::error::RefactorError>(())
//! ```
//!
//! ## Supported Languages
//!
//! - Rust (`.rs`)
//! - TypeScript/JavaScript (`.ts`, `.tsx`, `.js`, `.jsx`)
//! - Python (`.py`, `.pyi`)

pub mod diff;
pub mod error;
pub mod lang;
pub mod matcher;
pub mod refactor;
pub mod transform;

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::error::{RefactorError, Result};
    pub use crate::lang::{Language, LanguageRegistry, Python, Rust, TypeScript};
    pub use crate::matcher::{AstMatcher, FileMatcher, GitMatcher, Matcher};
    pub use crate::refactor::{MultiRepoRefactor, Refactor, RefactorResult};
    pub use crate::transform::{
        AstTransform, FileTransform, TextTransform, Transform, TransformBuilder,
    };
}

pub use prelude::*;
