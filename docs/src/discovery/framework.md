# Framework Filters

Detect and filter repositories by the frameworks they use, going beyond simple dependency checks.

## Overview

`FrameworkFilter` identifies frameworks through multiple signals:

- Configuration files (e.g., `next.config.js`, `angular.json`)
- Dependency patterns (combinations of packages)
- Project structure (directories, file patterns)
- Import patterns in source code

## Supported Frameworks

### JavaScript/TypeScript

| Framework | Detection Method |
|-----------|------------------|
| React | `react` dependency + JSX files |
| Next.js | `next.config.*` + `next` dependency |
| Vue | `vue` dependency + `.vue` files |
| Nuxt | `nuxt.config.*` + `nuxt` dependency |
| Angular | `angular.json` + `@angular/core` |
| Svelte | `svelte.config.*` + `svelte` dependency |
| Express | `express` + server patterns |
| NestJS | `@nestjs/core` + decorators |
| Gatsby | `gatsby-config.*` + `gatsby` |
| Remix | `remix.config.*` + `@remix-run/*` |

### Python

| Framework | Detection Method |
|-----------|------------------|
| Django | `django` + `manage.py` + `settings.py` |
| Flask | `flask` + `app.py` patterns |
| FastAPI | `fastapi` + async patterns |
| Pyramid | `pyramid` dependency |
| Tornado | `tornado` dependency |

### Ruby

| Framework | Detection Method |
|-----------|------------------|
| Rails | `rails` gem + `config/application.rb` |
| Sinatra | `sinatra` gem + app patterns |
| Hanami | `hanami` gem |

### Java

| Framework | Detection Method |
|-----------|------------------|
| Spring Boot | `spring-boot-*` + `@SpringBootApplication` |
| Spring | `spring-*` dependencies |
| Quarkus | `quarkus-*` + `application.properties` |
| Micronaut | `micronaut-*` |
| Jakarta EE | `jakarta.*` dependencies |

### Go

| Framework | Detection Method |
|-----------|------------------|
| Gin | `github.com/gin-gonic/gin` |
| Echo | `github.com/labstack/echo` |
| Fiber | `github.com/gofiber/fiber` |
| Chi | `github.com/go-chi/chi` |

### Rust

| Framework | Detection Method |
|-----------|------------------|
| Actix | `actix-web` crate |
| Axum | `axum` crate |
| Rocket | `rocket` crate |
| Warp | `warp` crate |

### C#

| Framework | Detection Method |
|-----------|------------------|
| ASP.NET Core | `Microsoft.AspNetCore.*` |
| Blazor | `Microsoft.AspNetCore.Components.*` |
| WPF | `PresentationFramework` reference |
| WinForms | `System.Windows.Forms` reference |

## Basic Usage

```rust
use refactor::prelude::*;

// Find all Next.js projects
Codemod::from_github_org("company", token)
    .repositories(|r| r
        .uses_framework(Framework::NextJs))
    .apply(transform)
    .execute()?;
```

## Framework Enum

```rust
pub enum Framework {
    // JavaScript/TypeScript
    React,
    NextJs,
    Vue,
    Nuxt,
    Angular,
    Svelte,
    Express,
    NestJS,
    Gatsby,
    Remix,

    // Python
    Django,
    Flask,
    FastAPI,

    // Ruby
    Rails,
    Sinatra,

    // Java
    Spring,
    SpringBoot,
    Quarkus,

    // Go
    Gin,
    Echo,
    Fiber,

    // Rust
    Actix,
    Axum,
    Rocket,

    // C#
    AspNetCore,
    Blazor,
}
```

## Multiple Frameworks

### All Required (AND)

```rust
.repositories(|r| r
    .uses_framework(Framework::React)
    .uses_framework(Framework::Express))  // Full-stack React + Express
```

### Any Required (OR)

```rust
// Find repos using any React meta-framework
.repositories(|r| r
    .uses_any_framework(&[
        Framework::NextJs,
        Framework::Gatsby,
        Framework::Remix,
    ]))
```

## Framework Detection Details

### Next.js Detection

```rust
// Detected by:
// 1. next.config.js or next.config.mjs exists
// 2. package.json has "next" dependency
// 3. pages/ or app/ directory structure
// 4. Imports from "next/*"

.uses_framework(Framework::NextJs)
```

### Rails Detection

```rust
// Detected by:
// 1. Gemfile contains "rails"
// 2. config/application.rb exists
// 3. app/controllers/ structure
// 4. config/routes.rb exists

.uses_framework(Framework::Rails)
```

### Spring Boot Detection

```rust
// Detected by:
// 1. spring-boot-starter-* dependencies
// 2. @SpringBootApplication annotation
// 3. application.properties or application.yml
// 4. pom.xml with spring-boot-starter-parent

.uses_framework(Framework::SpringBoot)
```

## Custom Framework Detection

Define custom framework detection rules:

```rust
use refactor::discovery::FrameworkFilter;

let custom_framework = FrameworkFilter::custom("my-framework")
    .requires_dependency("my-framework-core", "*")
    .requires_file("my-framework.config.js")
    .requires_pattern("src/**/*.myfw");

.repositories(|r| r
    .framework(custom_framework))
```

## Direct Usage

```rust
use refactor::discovery::FrameworkFilter;

let filter = FrameworkFilter::new(Framework::React);

// Check a single repository
if filter.matches(Path::new("./my-project"))? {
    println!("Project uses React");
}

// Get detailed info
if let Some(info) = filter.detect(Path::new("./my-project"))? {
    println!("Framework: {}", info.name);
    println!("Version: {:?}", info.version);
    println!("Confidence: {:.0}%", info.confidence * 100.0);
}
```

## Confidence Scores

Detection returns confidence scores:

```rust
let result = FrameworkFilter::new(Framework::React)
    .detect(Path::new("./project"))?;

match result {
    Some(detection) if detection.confidence > 0.9 => {
        println!("Definitely React");
    }
    Some(detection) if detection.confidence > 0.5 => {
        println!("Probably React ({}%)", detection.confidence * 100.0);
    }
    Some(_) => {
        println!("Might be React, low confidence");
    }
    None => {
        println!("Not React");
    }
}
```

## Framework Version Detection

```rust
let detection = FrameworkFilter::new(Framework::NextJs)
    .detect(Path::new("./project"))?;

if let Some(d) = detection {
    if let Some(version) = d.version {
        if version.major >= 13 {
            println!("Uses App Router (Next.js 13+)");
        } else {
            println!("Uses Pages Router");
        }
    }
}
```

## Use Cases

### Framework Migration

```rust
// Find Create React App projects to migrate to Next.js
Codemod::from_github_org("company", token)
    .repositories(|r| r
        .uses_framework(Framework::React)
        .has_dependency("react-scripts", "*")
        .missing_framework(Framework::NextJs))
    .collect_repos()?;
```

### Framework Inventory

```rust
// Count framework usage across organization
let repos = Codemod::from_github_org("company", token)
    .collect_all_repos()?;

let mut framework_counts = HashMap::new();
for repo in &repos {
    for framework in Framework::all() {
        if FrameworkFilter::new(framework).matches(&repo.path)? {
            *framework_counts.entry(framework).or_insert(0) += 1;
        }
    }
}
```

### Framework-Specific Transforms

```rust
Codemod::from_github_org("company", token)
    .repositories(|r| r.uses_framework(Framework::Express))
    .apply(|ctx| {
        // Apply Express-specific security patches
        ctx.add_middleware("helmet")
    })
    .execute()?;
```

## Error Handling

```rust
use refactor::error::RefactorError;

let filter = FrameworkFilter::new(Framework::React);

match filter.matches(Path::new("./project")) {
    Ok(true) => println!("Uses React"),
    Ok(false) => println!("Doesn't use React"),
    Err(RefactorError::IoError(e)) => {
        println!("Failed to read project: {}", e);
    }
    Err(e) => return Err(e.into()),
}
```

## See Also

- [Dependency Filters](./dependency.md) - Lower-level package filtering
- [Enhanced Discovery](./README.md) - Full discovery guide
