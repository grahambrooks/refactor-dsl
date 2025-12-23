# Refactor DSL

A domain-specific language for multi-language code refactoring with Git-aware matching.

## What is Refactor DSL?

Refactor DSL is a Rust library and CLI tool that provides a fluent, type-safe API for performing code refactoring operations across multiple programming languages. It combines:

- **Git-aware matching** - Filter repositories by branch, commit history, and working state
- **Flexible file matching** - Glob patterns, extensions, and content-based filtering
- **AST-powered queries** - Find and transform code using tree-sitter syntax patterns
- **LSP integration** - Semantic refactoring through Language Server Protocol
- **Multi-language support** - Rust, TypeScript, JavaScript, and Python out of the box

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
- Support for rust-analyzer, typescript-language-server, pyright, gopls, and clangd

## When to Use This

Refactor DSL is ideal for:

- **Codebase migrations** - Update API usage patterns across many files
- **Style enforcement** - Apply consistent code patterns
- **Bulk refactoring** - Rename symbols, update imports, transform patterns
- **Multi-repo operations** - Apply the same changes across multiple repositories
- **Automated code cleanup** - Remove deprecated patterns, add missing annotations

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Refactor DSL                         │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
│  │  Matchers   │  │ Transforms  │  │  LSP Client     │ │
│  │             │  │             │  │                 │ │
│  │ - Git       │  │ - Text      │  │ - Rename        │ │
│  │ - File      │  │ - AST       │  │ - References    │ │
│  │ - AST       │  │ - File      │  │ - Definition    │ │
│  └─────────────┘  └─────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────┐   │
│  │              Language Support                    │   │
│  │                                                  │   │
│  │  Rust  │  TypeScript  │  Python  │  (extensible) │   │
│  └─────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────┤
│  tree-sitter │ git2 │ walkdir │ globset │ regex        │
└─────────────────────────────────────────────────────────┘
```
