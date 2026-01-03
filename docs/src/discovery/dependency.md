# Dependency Filters

Filter repositories based on their package dependencies across multiple ecosystems.

## Overview

`DependencyFilter` analyzes package manifest files to find repositories using specific dependencies:

- **npm/yarn** - `package.json`
- **Cargo** - `Cargo.toml`
- **pip/Poetry** - `requirements.txt`, `pyproject.toml`, `setup.py`
- **Maven** - `pom.xml`
- **Gradle** - `build.gradle`, `build.gradle.kts`
- **Bundler** - `Gemfile`
- **Go modules** - `go.mod`
- **NuGet** - `*.csproj`, `packages.config`

## Basic Usage

```rust
use refactor_dsl::prelude::*;

// Find repos using a specific dependency
Codemod::from_github_org("company", token)
    .repositories(|r| r
        .has_dependency("react", ">=17.0"))
    .apply(transform)
    .execute()?;
```

## Version Specifiers

### Exact Version

```rust
.has_dependency("lodash", "4.17.21")
```

### Range

```rust
// Greater than or equal
.has_dependency("react", ">=17.0")

// Less than
.has_dependency("vulnerable-pkg", "<2.0")

// Between
.has_dependency("typescript", ">=4.0,<5.0")
```

### Any Version

```rust
.has_dependency("express", "*")
```

### Semantic Versioning

```rust
// Major version
.has_dependency("react", "^17.0")  // >=17.0.0, <18.0.0

// Minor version
.has_dependency("axios", "~0.21")  // >=0.21.0, <0.22.0
```

## Ecosystem-Specific Filters

### NPM/Yarn

```rust
use refactor_dsl::discovery::DependencyFilter;

// Production dependency
DependencyFilter::npm("react", ">=17.0")

// Dev dependency only
DependencyFilter::npm_dev("typescript", ">=4.0")

// Peer dependency
DependencyFilter::npm_peer("react", ">=16.8")
```

### Cargo (Rust)

```rust
// Regular dependency
DependencyFilter::cargo("tokio", ">=1.0")

// Dev dependency
DependencyFilter::cargo_dev("mockall", "*")

// Build dependency
DependencyFilter::cargo_build("cc", "*")
```

### Python (pip/Poetry)

```rust
// Any Python package
DependencyFilter::pip("django", ">=3.0")

// Poetry-specific
DependencyFilter::poetry("fastapi", "*")

// From requirements.txt
DependencyFilter::requirements("numpy", ">=1.20")
```

### Maven/Gradle (Java)

```rust
// Maven dependency
DependencyFilter::maven("org.springframework:spring-boot", ">=2.5")

// Gradle
DependencyFilter::gradle("com.google.guava:guava", "*")
```

### Bundler (Ruby)

```rust
DependencyFilter::bundler("rails", ">=6.0")
DependencyFilter::bundler("rspec", "*")
```

### Go Modules

```rust
DependencyFilter::go("github.com/gin-gonic/gin", ">=1.7")
```

### NuGet (C#)

```rust
DependencyFilter::nuget("Newtonsoft.Json", ">=13.0")
```

## Multiple Dependencies

### All Required (AND)

```rust
.repositories(|r| r
    .has_dependency("react", ">=17.0")
    .has_dependency("react-dom", ">=17.0")
    .has_dependency("typescript", ">=4.0"))
```

### Any Required (OR)

Use separate filters and combine:

```rust
let has_react = DependencyFilter::npm("react", "*");
let has_vue = DependencyFilter::npm("vue", "*");

// Find repos with either
let repos = discover_repos("./workspace")?
    .filter(|repo| has_react.matches(repo) || has_vue.matches(repo))
    .collect();
```

## Negative Filters

Find repos that **don't** have a dependency:

```rust
.repositories(|r| r
    .missing_dependency("moment")  // Repos without moment.js
)
```

## Transitive Dependencies

By default, only direct dependencies are checked. Enable transitive:

```rust
DependencyFilter::npm("lodash", "*")
    .include_transitive(true)
```

**Note:** Transitive analysis requires `package-lock.json`, `Cargo.lock`, etc.

## Direct Usage

```rust
use refactor_dsl::discovery::DependencyFilter;

let filter = DependencyFilter::npm("react", ">=17.0");

// Check a single repository
if filter.matches(Path::new("./my-project"))? {
    println!("Project uses React 17+");
}

// Get dependency info
if let Some(info) = filter.get_dependency_info(Path::new("./my-project"))? {
    println!("Found: {} v{}", info.name, info.version);
    println!("Type: {:?}", info.dependency_type);  // Production, Dev, Peer
}
```

## Parsing Examples

### package.json

```json
{
  "dependencies": {
    "react": "^17.0.2",
    "react-dom": "^17.0.2"
  },
  "devDependencies": {
    "typescript": "^4.5.0"
  }
}
```

```rust
.has_dependency("react", ">=17.0")      // Matches ^17.0.2
.has_dependency("typescript", ">=4.5")  // Matches ^4.5.0 in devDeps
```

### Cargo.toml

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = "1.0"

[dev-dependencies]
mockall = "0.11"
```

```rust
.has_dependency("tokio", ">=1.0")   // Matches
.has_dependency("serde", ">=1.0")   // Matches
```

### requirements.txt

```
Django>=3.2,<4.0
numpy==1.21.0
requests
```

```rust
.has_dependency("django", ">=3.2")   // Matches
.has_dependency("numpy", "1.21.0")   // Matches exact
.has_dependency("requests", "*")     // Matches (no version = any)
```

## Error Handling

```rust
use refactor_dsl::error::RefactorError;

let filter = DependencyFilter::npm("react", ">=17.0");

match filter.matches(Path::new("./project")) {
    Ok(true) => println!("Matches"),
    Ok(false) => println!("Doesn't match"),
    Err(RefactorError::ManifestNotFound(path)) => {
        println!("No package.json found at {}", path.display());
    }
    Err(RefactorError::ParseError { path, message }) => {
        println!("Failed to parse {}: {}", path.display(), message);
    }
    Err(e) => return Err(e.into()),
}
```

## Performance Tips

1. **Specify ecosystem** - Use `npm()`, `cargo()`, etc. instead of generic `has_dependency()`
2. **Avoid transitive** - Only enable when necessary
3. **Cache results** - Discovery caches manifest parsing

## See Also

- [Framework Filters](./framework.md) - Higher-level framework detection
- [Enhanced Discovery](./README.md) - Full discovery guide
