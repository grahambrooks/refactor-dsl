# Refactor DSL

A domain-specific language for multi-language code refactoring with Git-aware matching.

## What is Refactor DSL?

Refactor DSL is a Rust library and CLI tool that provides a fluent, type-safe API for performin code refactoring operations across multiple programming languages. It combines:

- **Git-aware matching** - Filter repositories by branch, commit history, and working state
- **Flexible file matching** - Glob patterns, extensions, and content-based filtering
- **AST-powered queries** - Find and transform code using tree-sitter syntax patterns
- **LSP integration** - Semantic refactoring through Language Server Protocol
- **Multi-language support** - Rust, TypeScript, Python, Go, Java, C#, and Ruby
- **IDE-like refactoring** - Extract, Inline, Move, Change Signature, Safe Delete, Find Dead Code
- **Scope analysis** - Track bindings, resolve references, and analyze usage across files
- **Enhanced discovery** - Filter repositories by dependencies, frameworks, and metrics

## Key Features

### Fluent Builder API

```rust
use refactor_dsl::prelude::*;

Refactor::in_repo("./my-project")
    .matching(|m| m
        .git(|g| g.branch("main"))
        .files(|f| f.extension("rs").exclude("**/target/**")))
    .transform(|t| t
        .replace_pattern(r"\.unwrap\(\)", ".expect(\"TODO\")"))
    .dry_run()
    .apply()?;
```

### Powerful Matching

- Match repositories by Git branch, commit age, remotes, and working tree state
- Match files by extension, glob patterns, content patterns, and size
- Match code structures using tree-sitter AST queries

### Safe Transformations

- Preview changes with `dry_run()` before applying
- Generate unified diffs for review
- Atomic file operations with rollback on failure

### LSP-based Semantic Refactoring

- Rename symbols with full project awareness
- Auto-install LSP servers from the Mason registry
- Support for rust-analyzer, typescript-language-server, pyright, gopls, jdtls, omnisharp, solargraph, and clangd

### IDE-Like Refactoring Operations

```rust
use refactor_dsl::prelude::*;

// Extract a function from selected code
ExtractFunction::new("calculate_total")
    .from_file("src/checkout.rs")
    .range(Range::new(Position::new(10, 0), Position::new(20, 0)))
    .visibility(Visibility::Public)
    .execute()?;

// Find dead code in a workspace
let report = FindDeadCode::in_workspace("./project")
    .include(DeadCodeType::UnusedFunctions)
    .include(DeadCodeType::UnusedImports)
    .execute()?;

// Safely delete a symbol with usage checking
SafeDelete::symbol("unused_helper")
    .in_file("src/utils.rs")
    .check_usages(true)
    .execute()?;
```

### Enhanced Repository Discovery

```rust
use refactor_dsl::prelude::*;

// Filter GitHub org repos by dependencies and frameworks
Codemod::from_github_org("acme-corp", token)
    .repositories(|r| r
        .has_dependency("react", ">=17.0")
        .uses_framework(Framework::NextJs)
        .lines_of_code(ComparisonOp::GreaterThan, 1000.0))
    .apply(upgrade)
    .execute()?;
```

## When to Use This

Refactor DSL is ideal for:

- **Codebase migrations** - Update API usage patterns across many files
- **Style enforcement** - Apply consistent code patterns
- **Bulk refactoring** - Rename symbols, update imports, transform patterns
- **Multi-repo operations** - Apply the same changes across multiple repositories
- **Automated code cleanup** - Remove deprecated patterns, add missing annotations
- **Code quality** - Find and remove dead code, enforce naming conventions
- **Safe refactoring** - Extract functions, change signatures with call-site updates

## Architecture

```
┌───────────────────────────────────────────────────────────────────────┐
│                           Refactor DSL                                │
├───────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌───────────────┐  ┌────────────┐  │
│  │  Matchers   │  │ Transforms  │  │ LSP Client    │  │ Discovery  │  │
│  │             │  │             │  │               │  │            │  │
│  │ - Git       │  │ - Text      │  │ - Rename      │  │ - Deps     │  │
│  │ - File      │  │ - AST       │  │ - References  │  │ - Framework│  │
│  │ - AST       │  │ - File      │  │ - Definition  │  │ - Metrics  │  │
│  └─────────────┘  └─────────────┘  └───────────────┘  └────────────┘  │
├───────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                    Refactoring Operations                       │  │
│  │                                                                 │  │
│  │  Extract │ Inline │ Move │ ChangeSignature │ SafeDelete │ Dead  │  │
│  └─────────────────────────────────────────────────────────────────┘  │
├───────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                      Scope Analysis                             │  │
│  │                                                                 │  │
│  │  Bindings │ References │ Usage Analysis │ Cross-file Resolution │  │
│  └─────────────────────────────────────────────────────────────────┘  │
├───────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                      Language Support                           │  │
│  │                                                                 │  │
│  │  Rust │ TypeScript │ Python │ Go │ Java │ C# │ Ruby │ C/C++     │  │
│  └─────────────────────────────────────────────────────────────────┘  │
├───────────────────────────────────────────────────────────────────────┤
│  tree-sitter │ git2 │ walkdir │ globset │ regex │ lsp-types           │
└───────────────────────────────────────────────────────────────────────┘
```
