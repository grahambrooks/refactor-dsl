---
name: refactor-dsl
description: Use the refactor-dsl `refactor` CLI to search code by tree-sitter query or make a regex find-and-replace across many files, previewing a diff first. Use when you need to list every function, call or other syntax node matching a tree-sitter pattern in Rust, TypeScript/JavaScript, Python, Go, Java, C# or Ruby; when a repo-wide edit is a textual pattern (a regex with capture groups) scoped by extension or glob; or when deciding between refactor-dsl and colab for a mechanical change. Not for symbol-aware renames; `refactor rename` is a plain substring replace.
---

# refactor-dsl

The `refactor` binary has four subcommands:

| Command | What it does |
|---|---|
| `refactor find -q <query> [-e ext] [path]` | Runs a tree-sitter query and prints every capture as `file:line:col: text (capture)`. Read-only. |
| `refactor replace -p <regex> -r <replacement> [-e ext] [-i glob] [--exclude glob] [--dry-run] [path]` | Regex find-and-replace over the matched files. `$1` etc. refer to capture groups. |
| `refactor rename --from <a> --to <b> [-e ext] [--dry-run] [path]` | Literal substring replace of `a` with `b`. **Not** symbol-aware. |
| `refactor languages` | Lists the supported languages and their extensions. |

`path` defaults to `.`. Every command and output below came from running the
v2026.11.1 binary.

## refactor-dsl or colab?

Both use tree-sitter, but they do different jobs.

- **colab** (the `colab-codemod` skill) *rewrites syntax*: renaming a symbol
  together with its call sites, changing import/use/package paths, rewriting
  calls, editing TOML/JSON/Markdown by key. It knows a symbol from the same
  letters inside a string or comment. Use colab for any rename or
  import change it supports.
- **refactor-dsl** *finds syntax* and *replaces text*. Use it to:
  - answer "where are all the X?" with a tree-sitter query (`refactor find`),
    for example to measure how big a change is before writing a colab rule;
  - make an edit that is really a text pattern, not a symbol: replacing
    `.unwrap()` with `.expect("TODO")`, rewriting a string format, a regex with
    capture groups. colab has no regex action.

If you are about to use `refactor rename` or a bare-word `replace` to rename an
identifier, use colab instead. If colab doesn't support the language or
construct, use `replace` with `\b` word boundaries and read the diff line by line.

## Find syntax nodes

```sh
refactor find -q '(function_item name: (identifier) @fn)' -e rs .
```
```
./src/lib.rs:1:8: fetch_user (fn)
./src/lib.rs:8:8: fetch_user_list (fn)
```

```sh
refactor find -q '(call function: (identifier) @callee)' -e py .
```
```
./src/app.py:4:1: print (callee)
./src/app.py:4:7: fetch_user (callee)
```

Things to know:

- Queries use the grammar's own node names: `function_item` and
  `call_expression` in Rust, `function_definition` and `call` in Python. If you
  don't know a node name, look it up in the grammar's `node-types.json`.
- **A bad query prints nothing and exits 0**, exactly like a query that has no
  matches. If you expected hits and got none, check the node names before you
  conclude the code is clean.
- Without `-e`, the query runs against every supported file and only matches
  where it is valid for that language. Pass `-e` so it is clear what was
  searched.
- Code inside macro invocations is a token tree, not parsed syntax.
  `vec![fetch_user(1)]` does **not** match a Rust `call_expression` query.

## Replace text across files

Always do a dry run first. It prints a coloured unified diff and a summary line:

```sh
refactor replace -p '\bfetch_user\b' -r load_user -e rs --exclude '**/target/**' --dry-run .
```
```
--- a/./src/lib.rs
+++ b/./src/lib.rs
-pub fn fetch_user(id: u32) -> Option<String> {
+pub fn load_user(id: u32) -> Option<String> {
     let v: Option<String> = None;
     v.clone().unwrap();
-    // fetch_user is called by fetch_user_list
-    Some(format!("fetch_user {id}"))
+    // load_user is called by fetch_user_list
+    Some(format!("load_user {id}"))
 }
...
1 file(s) changed, 4 insertions(+), 4 deletions(-)
```

The diff shows the hazard: the comment and the string literal changed too.
`replace` is text-only. To skip them, use colab.

Capture groups and include globs:

```sh
refactor replace -p 'fetch_(\w+)' -r 'load_$1' -e py --dry-run .
refactor replace -p '\.unwrap\(\)' -r '.expect("TODO")' -i 'src/**/*.rs' --dry-run .
```

Once the diff looks right, run the same command without `--dry-run`:

```sh
refactor replace -p '\bfetch_user\b' -r load_user -e rs --exclude '**/target/**' .
```
```
Modified 1 file(s)
```

Then check with `git diff` and build or test.

## Gotchas (all observed)

- **Always pass `-e` or `-i`.** Without a filter the walk includes `.git/`, and
  the command fails on a binary file:
  `Error: ... IO error: stream did not contain valid UTF-8` (exit 1). This
  applies to `replace` and `rename` even when nothing would match.
- **`.gitignore` is not honoured.** `target/`, `node_modules/` and other build
  output get scanned and rewritten. Add `--exclude '**/target/**'` (or similar);
  `find` has no `--exclude`, so pass it a narrower path such as `src`.
- **`rename` is a substring replace.** `--from fetch_user --to load_user` also
  turns `fetch_user_list` into `load_user_list` and rewrites comments and
  strings. Prefer colab; failing that, use `replace` with `\b...\b`.
- **An invalid regex panics** (exit 101, `invalid regex: Syntax(...)`). Quote
  patterns in single quotes so the shell leaves backslashes alone.
- The dry-run diff contains ANSI colour codes even when piped. Strip them with
  `sed 's/\x1b\[[0-9;]*m//g'` if you need to parse it.
- The library supports Git-aware matching (branch, history, working state) and
  LSP-backed operations, but the CLI does not expose them. Those need the Rust
  API (`Refactor::in_repo(...)`); see the project docs.
