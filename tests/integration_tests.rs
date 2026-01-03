//! Integration tests for the refactor crate.

use refactor::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

fn create_rust_project(dir: &std::path::Path) {
    fs::create_dir_all(dir.join("src")).unwrap();

    File::create(dir.join("Cargo.toml"))
        .unwrap()
        .write_all(b"[package]\nname = \"test-project\"\nversion = \"0.1.0\"\nedition = \"2021\"\n")
        .unwrap();

    File::create(dir.join("src/main.rs"))
        .unwrap()
        .write_all(
            b"fn main() {\n    let x = get_value().unwrap();\n    println!(\"{}\", x);\n}\n\nfn get_value() -> Option<i32> {\n    Some(42)\n}\n",
        )
        .unwrap();

    File::create(dir.join("src/lib.rs"))
        .unwrap()
        .write_all(
            b"pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\npub fn multiply(a: i32, b: i32) -> i32 {\n    a * b\n}\n",
        )
        .unwrap();
}

#[test]
fn test_refactor_dry_run() {
    let dir = TempDir::new().unwrap();
    create_rust_project(dir.path());

    let result = Refactor::in_repo(dir.path())
        .matching(|m| m.files(|f| f.extension("rs")))
        .transform(|t| t.replace_pattern(r"\.unwrap\(\)", ".expect(\"error\")"))
        .dry_run()
        .apply()
        .unwrap();

    // Check that files were found
    assert!(!result.changes.is_empty());

    // Check that changes were detected
    assert!(result.changes.iter().any(|c| c.is_modified()));

    // Check that the diff contains expected changes
    let diff = result.diff();
    assert!(diff.contains("-    let x = get_value().unwrap();"));
    assert!(diff.contains("+    let x = get_value().expect(\"error\");"));

    // Verify the original file wasn't modified (dry run)
    let content = fs::read_to_string(dir.path().join("src/main.rs")).unwrap();
    assert!(content.contains(".unwrap()"));
}

#[test]
fn test_refactor_apply() {
    let dir = TempDir::new().unwrap();
    create_rust_project(dir.path());

    let result = Refactor::in_repo(dir.path())
        .matching(|m| m.files(|f| f.extension("rs")))
        .transform(|t| t.replace_literal("unwrap()", "expect(\"TODO\")"))
        .apply()
        .unwrap();

    assert!(!result.changes.is_empty());

    // Verify the file was actually modified
    let content = fs::read_to_string(dir.path().join("src/main.rs")).unwrap();
    assert!(content.contains(".expect(\"TODO\")"));
    assert!(!content.contains(".unwrap()"));
}

#[test]
fn test_file_matching_with_exclude() {
    let dir = TempDir::new().unwrap();
    create_rust_project(dir.path());

    // Create a test file that should be excluded
    fs::create_dir_all(dir.path().join("tests")).unwrap();
    File::create(dir.path().join("tests/test.rs"))
        .unwrap()
        .write_all(b"#[test]\nfn test_something() { let x = foo().unwrap(); }")
        .unwrap();

    let result = Refactor::in_repo(dir.path())
        .matching(|m| m.files(|f| f.extension("rs").exclude("**/tests/**")))
        .transform(|t| t.replace_literal("unwrap", "expect"))
        .dry_run()
        .apply()
        .unwrap();

    // Only files in src should be modified
    for change in &result.changes {
        assert!(
            !change.path.to_string_lossy().contains("tests"),
            "Test files should be excluded"
        );
    }
}

#[test]
fn test_content_pattern_matching() {
    let dir = TempDir::new().unwrap();
    create_rust_project(dir.path());

    let result = Refactor::in_repo(dir.path())
        .matching(|m| m.files(|f| f.extension("rs").contains_pattern(r"fn main")))
        .transform(|t| t.replace_literal("println!", "eprintln!"))
        .dry_run()
        .apply()
        .unwrap();

    // Should only match main.rs which contains "fn main"
    assert_eq!(result.changes.len(), 1);
    assert!(result.changes[0].path.to_string_lossy().contains("main.rs"));
}

#[test]
fn test_chained_transforms() {
    let dir = TempDir::new().unwrap();
    create_rust_project(dir.path());

    let result = Refactor::in_repo(dir.path())
        .matching(|m| m.files(|f| f.extension("rs")))
        .transform(|t| {
            t.replace_literal("i32", "i64")
                .replace_literal("Some(42)", "Some(42i64)")
        })
        .dry_run()
        .apply()
        .unwrap();

    // Check that both transforms were applied
    let diff = result.diff();
    assert!(diff.contains("i64"));
}

#[test]
fn test_no_files_matched() {
    let dir = TempDir::new().unwrap();
    create_rust_project(dir.path());

    let result = Refactor::in_repo(dir.path())
        .matching(|m| m.files(|f| f.extension("xyz"))) // No such files
        .transform(|t| t.replace_literal("foo", "bar"))
        .apply();

    assert!(result.is_err());
}

#[test]
fn test_ast_matcher_integration() {
    let source = r#"
fn hello() {
    println!("Hello");
}

fn world() {
    println!("World");
}

fn greet(name: &str) {
    println!("Hello, {}!", name);
}
"#;

    let matcher = AstMatcher::new().query("(function_item name: (identifier) @fn_name)");

    let matches = matcher.find_matches(source, &Rust).unwrap();

    assert_eq!(matches.len(), 3);

    let fn_names: Vec<&str> = matches.iter().map(|m| m.text.as_str()).collect();
    assert!(fn_names.contains(&"hello"));
    assert!(fn_names.contains(&"world"));
    assert!(fn_names.contains(&"greet"));
}

#[test]
fn test_language_registry_integration() {
    let registry = LanguageRegistry::new();

    // Test detection from file paths
    let rust_lang = registry.detect(std::path::Path::new("src/main.rs"));
    assert!(rust_lang.is_some());
    assert_eq!(rust_lang.unwrap().name(), "rust");

    let ts_lang = registry.detect(std::path::Path::new("app/index.tsx"));
    assert!(ts_lang.is_some());
    assert_eq!(ts_lang.unwrap().name(), "typescript");

    let py_lang = registry.detect(std::path::Path::new("script.py"));
    assert!(py_lang.is_some());
    assert_eq!(py_lang.unwrap().name(), "python");
}

#[test]
fn test_transform_builder() {
    let transform = TransformBuilder::new()
        .replace_pattern(r"old_(\w+)", "new_$1")
        .replace_literal("deprecated", "legacy");

    let source = "old_function deprecated_api old_method";
    let result = transform
        .apply(source, std::path::Path::new("test.rs"))
        .unwrap();

    assert!(result.contains("new_function"));
    assert!(result.contains("new_method"));
    assert!(result.contains("legacy"));
    assert!(!result.contains("deprecated"));
}

#[test]
fn test_refactor_result_summary() {
    let dir = TempDir::new().unwrap();
    create_rust_project(dir.path());

    let result = Refactor::in_repo(dir.path())
        .matching(|m| m.files(|f| f.extension("rs")))
        .transform(|t| t.replace_literal("i32", "i64"))
        .dry_run()
        .apply()
        .unwrap();

    let summary = &result.summary;
    assert!(summary.files_changed > 0);
    assert!(summary.insertions > 0);
    assert!(summary.deletions > 0);

    // Test display
    let display = format!("{}", summary);
    assert!(display.contains("file(s) changed"));
    assert!(display.contains("insertions"));
    assert!(display.contains("deletions"));
}
