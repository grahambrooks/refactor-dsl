//! Refactoring operations for IDE-like code transformations.
//!
//! This module provides structured refactoring operations that can be validated,
//! previewed, and applied to code. Each operation implements the [`RefactoringOperation`]
//! trait.
//!
//! ## Available Operations
//!
//! ### Extract Operations
//! - [`ExtractFunction`] - Extract code into a new function
//! - [`ExtractVariable`] - Extract expression into a variable
//! - [`ExtractConstant`] - Extract value into a constant
//!
//! ### Inline Operations
//! - [`InlineVariable`] - Replace variable with its value
//! - [`InlineFunction`] - Replace function calls with body
//!
//! ### Move Operations
//! - [`MoveToFile`] - Move a symbol to a different file
//! - [`MoveBetweenModules`] - Move a symbol between modules
//!
//! ### Signature Operations
//! - [`ChangeSignature`] - Change function signature (add/remove/rename parameters)
//!
//! ### Safe Operations
//! - [`SafeDelete`] - Safely delete a symbol, checking for usages
//! - [`FindDeadCode`] - Find and report dead code
//!
//! ## Example
//!
//! ```rust,no_run
//! use refactor_dsl::refactor::operations::{
//!     RefactoringContext, ExtractFunction, RefactoringOperation,
//! };
//!
//! let mut ctx = RefactoringContext::new("/workspace", "src/lib.rs")
//!     .with_source("fn main() { let x = 1 + 2; }")
//!     .with_selection(0, 16, 0, 21);
//!
//! let op = ExtractFunction::new("add_numbers").public();
//!
//! // Validate the operation
//! let validation = op.validate(&ctx)?;
//! if validation.is_valid {
//!     // Preview changes
//!     let preview = op.preview(&ctx)?;
//!     println!("Will modify {} file(s)", preview.affected_files.len());
//!
//!     // Apply changes
//!     let result = op.apply(&mut ctx)?;
//!     println!("{}", result.description);
//! }
//! # Ok::<(), refactor_dsl::error::RefactorError>(())
//! ```

mod context;
mod dead_code;
mod extract;
mod inline;
mod move_ops;
mod safe_delete;
mod signature;

pub use context::{
    RefactoringContext, RefactoringPreview, RefactoringResult, TextEdit, ValidationResult,
};
pub use dead_code::{DeadCodeItem, DeadCodeReport, DeadCodeSummary, DeadCodeType, FindDeadCode};
pub use extract::{ExtractConstant, ExtractFunction, ExtractVariable, ParameterStrategy, Visibility};
pub use inline::{InlineFunction, InlineVariable};
pub use move_ops::{MoveBetweenModules, MoveToFile, SymbolKind};
pub use safe_delete::{DeleteKind, SafeDelete, UsageLocation};
pub use signature::{ChangeSignature, ParameterSpec};

use crate::error::Result;

/// Trait for refactoring operations.
///
/// All refactoring operations implement this trait, providing a consistent
/// interface for validation, preview, and application.
pub trait RefactoringOperation {
    /// Returns the name of this operation.
    fn name(&self) -> &'static str;

    /// Validates whether this operation can be applied.
    ///
    /// Returns a [`ValidationResult`] indicating whether the operation is valid
    /// and any warnings or errors.
    fn validate(&self, ctx: &RefactoringContext) -> Result<ValidationResult>;

    /// Generates a preview of the changes without applying them.
    ///
    /// Returns a [`RefactoringPreview`] containing the edits that would be made.
    fn preview(&self, ctx: &RefactoringContext) -> Result<RefactoringPreview>;

    /// Applies the refactoring operation.
    ///
    /// This modifies the source code and writes changes to disk.
    fn apply(&self, ctx: &mut RefactoringContext) -> Result<RefactoringResult>;

    /// Validates and applies the operation in one step.
    fn execute(&self, ctx: &mut RefactoringContext) -> Result<RefactoringResult> {
        let validation = self.validate(ctx)?;
        if !validation.is_valid {
            return Err(crate::error::RefactorError::InvalidConfig(
                validation.errors.join("; "),
            ));
        }
        self.apply(ctx)
    }

    /// Returns a dry-run preview as a diff string.
    fn dry_run(&self, ctx: &RefactoringContext) -> Result<String> {
        let preview = self.preview(ctx)?;
        Ok(preview.diff)
    }
}

/// Builder for running refactoring operations.
pub struct RefactoringRunner {
    dry_run: bool,
}

impl Default for RefactoringRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl RefactoringRunner {
    /// Create a new refactoring runner.
    pub fn new() -> Self {
        Self { dry_run: false }
    }

    /// Enable dry-run mode (preview only).
    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    /// Run an operation on a context.
    pub fn run(
        &self,
        operation: &dyn RefactoringOperation,
        ctx: &mut RefactoringContext,
    ) -> Result<RefactoringResult> {
        let validation = operation.validate(ctx)?;

        if !validation.is_valid {
            return Err(crate::error::RefactorError::InvalidConfig(
                validation.errors.join("; "),
            ));
        }

        // Print warnings
        for warning in &validation.warnings {
            eprintln!("Warning: {}", warning);
        }

        if self.dry_run {
            let preview = operation.preview(ctx)?;
            Ok(RefactoringResult::success(format!(
                "[DRY RUN] Would apply: {}\n{}",
                operation.name(),
                preview.diff
            )))
        } else {
            operation.apply(ctx)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refactoring_context_basic() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("fn hello() {}")
            .with_selection(0, 0, 0, 13);

        assert_eq!(ctx.selected_text(), "fn hello() {}");
    }

    #[test]
    fn test_extract_function_trait() {
        let op = ExtractFunction::new("my_func");
        assert_eq!(op.name(), "Extract Function");
    }

    #[test]
    fn test_extract_variable_trait() {
        let op = ExtractVariable::new("my_var");
        assert_eq!(op.name(), "Extract Variable");
    }

    #[test]
    fn test_inline_variable_trait() {
        let op = InlineVariable::new();
        assert_eq!(op.name(), "Inline Variable");
    }

    #[test]
    fn test_inline_function_trait() {
        let op = InlineFunction::new();
        assert_eq!(op.name(), "Inline Function");
    }

    #[test]
    fn test_move_to_file_trait() {
        let op = MoveToFile::new("target.rs");
        assert_eq!(op.name(), "Move to File");
    }

    #[test]
    fn test_move_between_modules_trait() {
        let op = MoveBetweenModules::new("utils");
        assert_eq!(op.name(), "Move Between Modules");
    }

    #[test]
    fn test_change_signature_trait() {
        let op = ChangeSignature::new();
        assert_eq!(op.name(), "Change Signature");
    }

    #[test]
    fn test_safe_delete_trait() {
        let op = SafeDelete::new();
        assert_eq!(op.name(), "Safe Delete");
    }

    #[test]
    fn test_find_dead_code_trait() {
        let op = FindDeadCode::new();
        assert_eq!(op.name(), "Find Dead Code");
    }

    #[test]
    fn test_runner_dry_run() {
        let runner = RefactoringRunner::new().dry_run();
        assert!(runner.dry_run);
    }
}
