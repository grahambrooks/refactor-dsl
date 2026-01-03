# Getting Started

This guide will help you get up and running with Refactor DSL quickly.

## Overview

Refactor DSL can be used in two ways:

1. **As a Rust library** - Integrate into your Rust projects for programmatic refactoring
2. **As a CLI tool** - Run refactoring operations from the command line

## Prerequisites

- Rust 1.70+ (for building from source)
- Git (for repository operations)

## Quick Example

Here's a simple example that replaces all `.unwrap()` calls with `.expect()`:

```rust
use refactor::prelude::*;

fn main() -> Result<()> {
    let result = Refactor::in_repo("./my-project")
        .matching(|m| m
            .files(|f| f.extension("rs")))
        .transform(|t| t
            .replace_pattern(r"\.unwrap\(\)", ".expect(\"TODO\")"))
        .dry_run()  // Preview first
        .apply()?;

    println!("{}", result.diff());
    Ok(())
}
```

Or using the CLI:

```bash
refactor replace \
    --pattern '\.unwrap\(\)' \
    --replacement '.expect("TODO")' \
    --extension rs \
    --dry-run
```

## Next Steps

- [Installation](./getting-started/installation.md) - Install the library or CLI
- [Quick Start](./getting-started/quick-start.md) - Walk through a complete example
- [Matchers](./matchers/README.md) - Learn how to select files and code
- [Transforms](./transforms/README.md) - Learn how to modify code
