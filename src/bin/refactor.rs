//! CLI for the refactor-dsl tool.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use refactor_dsl::prelude::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "refactor")]
#[command(author, version, about = "Multi-language code refactoring tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Replace text patterns in files
    Replace {
        /// Pattern to search for (regex)
        #[arg(short, long)]
        pattern: String,

        /// Replacement text
        #[arg(short, long)]
        replacement: String,

        /// File extension to filter (e.g., "rs", "ts")
        #[arg(short, long)]
        extension: Option<String>,

        /// Glob pattern to include
        #[arg(short, long)]
        include: Option<String>,

        /// Glob pattern to exclude
        #[arg(long)]
        exclude: Option<String>,

        /// Path to the repository
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Preview changes without applying
        #[arg(long)]
        dry_run: bool,
    },

    /// Find AST patterns in code
    Find {
        /// Tree-sitter query pattern
        #[arg(short, long)]
        query: String,

        /// File extension to filter
        #[arg(short, long)]
        extension: Option<String>,

        /// Path to search
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Rename a symbol across files
    Rename {
        /// Original symbol name
        #[arg(short, long)]
        from: String,

        /// New symbol name
        #[arg(short, long)]
        to: String,

        /// File extension to filter
        #[arg(short, long)]
        extension: Option<String>,

        /// Path to the repository
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Preview changes without applying
        #[arg(long)]
        dry_run: bool,
    },

    /// Show supported languages
    Languages,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Replace {
            pattern,
            replacement,
            extension,
            include,
            exclude,
            path,
            dry_run,
        } => {
            cmd_replace(pattern, replacement, extension, include, exclude, path, dry_run)
        }
        Commands::Find {
            query,
            extension,
            path,
        } => cmd_find(query, extension, path),
        Commands::Rename {
            from,
            to,
            extension,
            path,
            dry_run,
        } => cmd_rename(from, to, extension, path, dry_run),
        Commands::Languages => cmd_languages(),
    }
}

fn cmd_replace(
    pattern: String,
    replacement: String,
    extension: Option<String>,
    include: Option<String>,
    exclude: Option<String>,
    path: PathBuf,
    dry_run: bool,
) -> Result<()> {
    let mut refactor = Refactor::in_repo(&path).matching(|m| {
        let mut fm = FileMatcher::new();
        if let Some(ref ext) = extension {
            fm = fm.extension(ext);
        }
        if let Some(ref inc) = include {
            fm = fm.include(inc);
        }
        if let Some(ref exc) = exclude {
            fm = fm.exclude(exc);
        }
        m.files(|_| fm)
    });

    refactor = refactor.transform(|t| t.replace_pattern(&pattern, &replacement));

    if dry_run {
        refactor = refactor.dry_run();
    }

    let result = refactor.apply().context("Refactoring failed")?;

    if dry_run {
        println!("{}", result.colorized_diff());
        println!("\n{}", result.summary);
    } else {
        println!(
            "Modified {} file(s)",
            result.changes.iter().filter(|c| c.is_modified()).count()
        );
    }

    Ok(())
}

fn cmd_find(query: String, extension: Option<String>, path: PathBuf) -> Result<()> {
    let registry = LanguageRegistry::new();
    let matcher = AstMatcher::new().query(&query);

    let file_matcher = if let Some(ref ext) = extension {
        FileMatcher::new().extension(ext)
    } else {
        FileMatcher::new()
    };

    let files = file_matcher.collect(&path).context("Failed to collect files")?;

    for file in files {
        if let Ok(matches) = matcher.find_matches_in_file(&file, &registry) {
            for m in matches {
                println!(
                    "{}:{}:{}: {} ({})",
                    file.display(),
                    m.start_row + 1,
                    m.start_col + 1,
                    m.text,
                    m.capture_name
                );
            }
        }
    }

    Ok(())
}

fn cmd_rename(
    from: String,
    to: String,
    extension: Option<String>,
    path: PathBuf,
    dry_run: bool,
) -> Result<()> {
    let mut refactor = Refactor::in_repo(&path).matching(|m| {
        if let Some(ref ext) = extension {
            m.files(|f| f.extension(ext))
        } else {
            m
        }
    });

    refactor = refactor.transform(|t| t.replace_literal(&from, &to));

    if dry_run {
        refactor = refactor.dry_run();
    }

    let result = refactor.apply().context("Rename failed")?;

    if dry_run {
        println!("{}", result.colorized_diff());
        println!("\n{}", result.summary);
    } else {
        println!(
            "Renamed '{}' to '{}' in {} file(s)",
            from,
            to,
            result.changes.iter().filter(|c| c.is_modified()).count()
        );
    }

    Ok(())
}

fn cmd_languages() -> Result<()> {
    let registry = LanguageRegistry::new();
    println!("Supported languages:");
    for lang in registry.all() {
        println!(
            "  {} (extensions: {})",
            lang.name(),
            lang.extensions().join(", ")
        );
    }
    Ok(())
}
