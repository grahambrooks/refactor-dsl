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
//! use refactor::prelude::*;
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
//! # Ok::<(), refactor::error::RefactorError>(())
//! ```
//!
//! ## Repository Matching
//!
//! ```rust,no_run
//! use refactor::prelude::*;
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
//! # Ok::<(), refactor::error::RefactorError>(())
//! ```
//!
//! ## AST-based Matching
//!
//! ```rust,no_run
//! use refactor::prelude::*;
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
//! # Ok::<(), refactor::error::RefactorError>(())
//! ```
//!
//! ## Supported Languages
//!
//! - Rust (`.rs`)
//! - TypeScript/JavaScript (`.ts`, `.tsx`, `.js`, `.jsx`)
//! - Python (`.py`, `.pyi`)
//!
//! ## LSP-based Refactoring
//!
//! For semantic refactoring (rename, find references), use the LSP module:
//!
//! ```rust,no_run
//! use refactor::lsp::{LspRename, LspRegistry};
//!
//! // Rename a symbol using LSP
//! let result = LspRename::new("src/main.rs", 5, 4, "new_function_name")
//!     .dry_run()
//!     .execute()?;
//!
//! println!("Would modify {} files", result.file_count());
//! # Ok::<(), refactor::error::RefactorError>(())
//! ```

pub mod analyzer;
pub mod codemod;
pub mod diff;
pub mod error;
pub mod git;
pub mod github;
pub mod lang;
pub mod lsp;
pub mod matcher;
pub mod refactor;
pub mod scope;
pub mod transform;

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::analyzer::{
        AnalysisResult, ApiChange, ApiExtractor, ChangeDetector, ChangeKind, ConfigBasedUpgrade,
        FileContent, GeneratedUpgrade, LibraryAnalyzer, Transform as AnalyzerTransform,
        TransformSpec, UpgradeConfig, UpgradeGenerator,
    };
    pub use crate::codemod::{
        AdvancedRepoFilter, AngularV4V5Upgrade, Codemod, CodemodResult, ComparisonOp,
        DependencyFilter, DependencyInfo, FilterPresets, Framework, FrameworkCategory,
        FrameworkFilter, FrameworkInfo, LanguageFilter, LanguageInfo, MatchMode, MetricCondition,
        MetricFilter, PackageManager, ProgrammingLanguage, RepoFilter, RepositoryInfo,
        RepositoryMetrics, RxJS5To6Upgrade, Upgrade, VersionConstraint, angular_v4v5_upgrade,
        rxjs_5_to_6_upgrade,
    };
    pub use crate::error::{RefactorError, Result};
    pub use crate::git::{BranchOps, CommitOps, GitAuth, GitOps, PushOps};
    pub use crate::github::{GitHubClient, GitHubRepo, RepoOps};
    pub use crate::lang::{
        CSharp, Go, Java, Language, LanguageRegistry, Python, Ruby, Rust, TypeScript,
    };
    pub use crate::lsp::{LspClient, LspInstaller, LspRegistry, LspRename, LspServerConfig};
    pub use crate::matcher::{AstMatcher, FileMatcher, GitMatcher, Matcher};
    pub use crate::refactor::operations::{
        ChangeSignature, DeadCodeItem, DeadCodeReport, DeadCodeSummary, DeadCodeType, DeleteKind,
        ExtractConstant, ExtractFunction, ExtractVariable, FindDeadCode, InlineFunction,
        InlineVariable, MoveBetweenModules, MoveToFile, ParameterSpec, ParameterStrategy,
        RefactoringContext, RefactoringOperation, RefactoringPreview,
        RefactoringResult as OpRefactoringResult, RefactoringRunner, SafeDelete, SymbolKind,
        TextEdit, UsageLocation, ValidationResult, Visibility,
    };
    pub use crate::refactor::{MultiRepoRefactor, Refactor, RefactorResult};
    pub use crate::scope::{
        Binding, BindingKind, DeadCodeInfo, Reference, ReferenceKind, SafeDeleteResult,
        ScopeAnalyzer, UsageAnalyzer, UsageInfo,
    };
    pub use crate::transform::{
        AstTransform, FileTransform, TextTransform, Transform, TransformBuilder,
    };
}

pub use prelude::*;
