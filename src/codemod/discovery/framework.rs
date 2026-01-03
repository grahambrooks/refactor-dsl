//! Framework detection and filtering.

use std::collections::HashSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Known frameworks for detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Framework {
    // JavaScript/TypeScript frameworks
    /// React (react, react-dom)
    React,
    /// Next.js (next)
    NextJs,
    /// Vue.js (vue)
    Vue,
    /// Nuxt.js (nuxt)
    NuxtJs,
    /// Angular (@angular/core)
    Angular,
    /// Svelte (svelte)
    Svelte,
    /// Express.js (express)
    Express,
    /// NestJS (@nestjs/core)
    NestJs,
    /// Fastify (fastify)
    Fastify,

    // Python frameworks
    /// Django (django)
    Django,
    /// Flask (flask)
    Flask,
    /// FastAPI (fastapi)
    FastAPI,
    /// Pyramid (pyramid)
    Pyramid,
    /// Tornado (tornado)
    Tornado,

    // Ruby frameworks
    /// Ruby on Rails (rails)
    Rails,
    /// Sinatra (sinatra)
    Sinatra,
    /// Hanami (hanami)
    Hanami,

    // Java frameworks
    /// Spring Boot (spring-boot)
    SpringBoot,
    /// Quarkus (quarkus)
    Quarkus,
    /// Micronaut (micronaut)
    Micronaut,
    /// Jakarta EE
    JakartaEE,

    // .NET frameworks
    /// ASP.NET Core
    AspNetCore,
    /// Blazor
    Blazor,

    // Go frameworks
    /// Gin (gin-gonic/gin)
    Gin,
    /// Echo (labstack/echo)
    Echo,
    /// Fiber (gofiber/fiber)
    Fiber,
    /// Chi (go-chi/chi)
    Chi,

    // Rust frameworks
    /// Actix Web (actix-web)
    ActixWeb,
    /// Axum (axum)
    Axum,
    /// Rocket (rocket)
    Rocket,
    /// Warp (warp)
    Warp,

    // PHP frameworks
    /// Laravel (laravel/framework)
    Laravel,
    /// Symfony (symfony/*)
    Symfony,

    // Testing frameworks
    /// Jest (jest)
    Jest,
    /// Mocha (mocha)
    Mocha,
    /// Pytest (pytest)
    Pytest,
    /// RSpec (rspec)
    RSpec,
    /// JUnit (junit)
    JUnit,
}

impl Framework {
    /// Get the category of this framework.
    pub fn category(&self) -> FrameworkCategory {
        match self {
            Self::React | Self::Vue | Self::Angular | Self::Svelte => FrameworkCategory::Frontend,
            Self::NextJs | Self::NuxtJs => FrameworkCategory::Fullstack,
            Self::Express
            | Self::NestJs
            | Self::Fastify
            | Self::Django
            | Self::Flask
            | Self::FastAPI
            | Self::Pyramid
            | Self::Tornado
            | Self::Rails
            | Self::Sinatra
            | Self::Hanami
            | Self::SpringBoot
            | Self::Quarkus
            | Self::Micronaut
            | Self::JakartaEE
            | Self::AspNetCore
            | Self::Blazor
            | Self::Gin
            | Self::Echo
            | Self::Fiber
            | Self::Chi
            | Self::ActixWeb
            | Self::Axum
            | Self::Rocket
            | Self::Warp
            | Self::Laravel
            | Self::Symfony => FrameworkCategory::Backend,
            Self::Jest | Self::Mocha | Self::Pytest | Self::RSpec | Self::JUnit => {
                FrameworkCategory::Testing
            }
        }
    }

    /// Get the primary language for this framework.
    pub fn language(&self) -> &'static str {
        match self {
            Self::React
            | Self::NextJs
            | Self::Vue
            | Self::NuxtJs
            | Self::Angular
            | Self::Svelte
            | Self::Express
            | Self::NestJs
            | Self::Fastify
            | Self::Jest
            | Self::Mocha => "javascript",
            Self::Django
            | Self::Flask
            | Self::FastAPI
            | Self::Pyramid
            | Self::Tornado
            | Self::Pytest => "python",
            Self::Rails | Self::Sinatra | Self::Hanami | Self::RSpec => "ruby",
            Self::SpringBoot | Self::Quarkus | Self::Micronaut | Self::JakartaEE | Self::JUnit => {
                "java"
            }
            Self::AspNetCore | Self::Blazor => "csharp",
            Self::Gin | Self::Echo | Self::Fiber | Self::Chi => "go",
            Self::ActixWeb | Self::Axum | Self::Rocket | Self::Warp => "rust",
            Self::Laravel | Self::Symfony => "php",
        }
    }

    /// Get the package names to detect this framework.
    fn detection_packages(&self) -> Vec<&'static str> {
        match self {
            Self::React => vec!["react", "react-dom"],
            Self::NextJs => vec!["next"],
            Self::Vue => vec!["vue"],
            Self::NuxtJs => vec!["nuxt"],
            Self::Angular => vec!["@angular/core"],
            Self::Svelte => vec!["svelte"],
            Self::Express => vec!["express"],
            Self::NestJs => vec!["@nestjs/core"],
            Self::Fastify => vec!["fastify"],
            Self::Django => vec!["django", "Django"],
            Self::Flask => vec!["flask", "Flask"],
            Self::FastAPI => vec!["fastapi"],
            Self::Pyramid => vec!["pyramid"],
            Self::Tornado => vec!["tornado"],
            Self::Rails => vec!["rails"],
            Self::Sinatra => vec!["sinatra"],
            Self::Hanami => vec!["hanami"],
            Self::SpringBoot => vec!["spring-boot", "org.springframework.boot"],
            Self::Quarkus => vec!["quarkus", "io.quarkus"],
            Self::Micronaut => vec!["micronaut", "io.micronaut"],
            Self::JakartaEE => vec!["jakarta.ee", "javax.servlet"],
            Self::AspNetCore => vec!["Microsoft.AspNetCore"],
            Self::Blazor => vec!["Microsoft.AspNetCore.Components"],
            Self::Gin => vec!["github.com/gin-gonic/gin"],
            Self::Echo => vec!["github.com/labstack/echo"],
            Self::Fiber => vec!["github.com/gofiber/fiber"],
            Self::Chi => vec!["github.com/go-chi/chi"],
            Self::ActixWeb => vec!["actix-web"],
            Self::Axum => vec!["axum"],
            Self::Rocket => vec!["rocket"],
            Self::Warp => vec!["warp"],
            Self::Laravel => vec!["laravel/framework"],
            Self::Symfony => vec!["symfony/framework-bundle"],
            Self::Jest => vec!["jest"],
            Self::Mocha => vec!["mocha"],
            Self::Pytest => vec!["pytest"],
            Self::RSpec => vec!["rspec"],
            Self::JUnit => vec!["junit", "org.junit"],
        }
    }

    /// Get file patterns to detect this framework.
    fn detection_files(&self) -> Vec<&'static str> {
        match self {
            Self::NextJs => vec!["next.config.js", "next.config.mjs", "next.config.ts"],
            Self::NuxtJs => vec!["nuxt.config.js", "nuxt.config.ts"],
            Self::Angular => vec!["angular.json", ".angular.json"],
            Self::Svelte => vec!["svelte.config.js"],
            Self::Rails => vec!["config/application.rb", "Gemfile"],
            Self::Django => vec!["manage.py", "settings.py"],
            Self::Flask => vec!["app.py", "wsgi.py"],
            Self::Laravel => vec!["artisan", "app/Http/Kernel.php"],
            Self::SpringBoot => vec!["application.properties", "application.yml"],
            Self::Quarkus => vec!["application.properties"],
            _ => vec![],
        }
    }
}

/// Framework category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FrameworkCategory {
    /// Frontend frameworks (React, Vue, Angular, etc.)
    Frontend,
    /// Backend frameworks (Express, Django, Spring, etc.)
    Backend,
    /// Full-stack frameworks (Next.js, Nuxt.js, etc.)
    Fullstack,
    /// Testing frameworks (Jest, Pytest, etc.)
    Testing,
}

/// Filter repositories by frameworks.
#[derive(Debug, Clone, Default)]
pub struct FrameworkFilter {
    /// Required frameworks.
    required: HashSet<Framework>,
    /// Excluded frameworks.
    excluded: HashSet<Framework>,
    /// Required categories.
    required_categories: HashSet<FrameworkCategory>,
}

impl FrameworkFilter {
    /// Create a new framework filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Require a specific framework.
    pub fn uses(mut self, framework: Framework) -> Self {
        self.required.insert(framework);
        self
    }

    /// Exclude a specific framework.
    pub fn excludes(mut self, framework: Framework) -> Self {
        self.excluded.insert(framework);
        self
    }

    /// Require a framework category.
    pub fn category(mut self, category: FrameworkCategory) -> Self {
        self.required_categories.insert(category);
        self
    }

    /// Check if a repository matches this filter.
    pub fn matches(&self, repo_path: &Path) -> Result<bool> {
        let detected = FrameworkInfo::detect(repo_path)?;

        // Check required frameworks
        for fw in &self.required {
            if !detected.frameworks.contains(fw) {
                return Ok(false);
            }
        }

        // Check excluded frameworks
        for fw in &self.excluded {
            if detected.frameworks.contains(fw) {
                return Ok(false);
            }
        }

        // Check required categories
        for cat in &self.required_categories {
            let has_category = detected.frameworks.iter().any(|fw| fw.category() == *cat);
            if !has_category {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// Information about detected frameworks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrameworkInfo {
    /// Detected frameworks.
    pub frameworks: HashSet<Framework>,
    /// Primary framework (most likely main one).
    pub primary: Option<Framework>,
}

impl FrameworkInfo {
    /// Detect frameworks in a repository.
    pub fn detect(repo_path: &Path) -> Result<Self> {
        let mut frameworks = HashSet::new();

        // Check for framework-specific files
        for framework in Self::all_frameworks() {
            if Self::check_framework(repo_path, framework) {
                frameworks.insert(framework);
            }
        }

        // Determine primary framework
        let primary = Self::determine_primary(&frameworks);

        Ok(Self {
            frameworks,
            primary,
        })
    }

    /// Get all known frameworks.
    fn all_frameworks() -> Vec<Framework> {
        vec![
            Framework::React,
            Framework::NextJs,
            Framework::Vue,
            Framework::NuxtJs,
            Framework::Angular,
            Framework::Svelte,
            Framework::Express,
            Framework::NestJs,
            Framework::Fastify,
            Framework::Django,
            Framework::Flask,
            Framework::FastAPI,
            Framework::Pyramid,
            Framework::Tornado,
            Framework::Rails,
            Framework::Sinatra,
            Framework::Hanami,
            Framework::SpringBoot,
            Framework::Quarkus,
            Framework::Micronaut,
            Framework::JakartaEE,
            Framework::AspNetCore,
            Framework::Blazor,
            Framework::Gin,
            Framework::Echo,
            Framework::Fiber,
            Framework::Chi,
            Framework::ActixWeb,
            Framework::Axum,
            Framework::Rocket,
            Framework::Warp,
            Framework::Laravel,
            Framework::Symfony,
            Framework::Jest,
            Framework::Mocha,
            Framework::Pytest,
            Framework::RSpec,
            Framework::JUnit,
        ]
    }

    /// Check if a specific framework is present.
    fn check_framework(repo_path: &Path, framework: Framework) -> bool {
        // Check for detection files
        for file in framework.detection_files() {
            if repo_path.join(file).exists() {
                return true;
            }
        }

        // Check for packages in manifests
        let packages = framework.detection_packages();
        if packages.is_empty() {
            return false;
        }

        // Check package.json
        if let Ok(content) = std::fs::read_to_string(repo_path.join("package.json")) {
            for pkg in &packages {
                if content.contains(&format!("\"{}\"", pkg)) {
                    return true;
                }
            }
        }

        // Check Cargo.toml
        if let Ok(content) = std::fs::read_to_string(repo_path.join("Cargo.toml")) {
            for pkg in &packages {
                if content.contains(pkg) {
                    return true;
                }
            }
        }

        // Check requirements.txt
        if let Ok(content) = std::fs::read_to_string(repo_path.join("requirements.txt")) {
            for pkg in &packages {
                if content.to_lowercase().contains(&pkg.to_lowercase()) {
                    return true;
                }
            }
        }

        // Check pyproject.toml
        if let Ok(content) = std::fs::read_to_string(repo_path.join("pyproject.toml")) {
            for pkg in &packages {
                if content.to_lowercase().contains(&pkg.to_lowercase()) {
                    return true;
                }
            }
        }

        // Check Gemfile
        if let Ok(content) = std::fs::read_to_string(repo_path.join("Gemfile")) {
            for pkg in &packages {
                if content.contains(pkg) {
                    return true;
                }
            }
        }

        // Check go.mod
        if let Ok(content) = std::fs::read_to_string(repo_path.join("go.mod")) {
            for pkg in &packages {
                if content.contains(pkg) {
                    return true;
                }
            }
        }

        // Check pom.xml
        if let Ok(content) = std::fs::read_to_string(repo_path.join("pom.xml")) {
            for pkg in &packages {
                if content.contains(pkg) {
                    return true;
                }
            }
        }

        // Check composer.json
        if let Ok(content) = std::fs::read_to_string(repo_path.join("composer.json")) {
            for pkg in &packages {
                if content.contains(pkg) {
                    return true;
                }
            }
        }

        false
    }

    /// Determine the primary framework.
    fn determine_primary(frameworks: &HashSet<Framework>) -> Option<Framework> {
        // Prefer full-stack over frontend
        for fw in frameworks {
            if fw.category() == FrameworkCategory::Fullstack {
                return Some(*fw);
            }
        }

        // Then backend
        for fw in frameworks {
            if fw.category() == FrameworkCategory::Backend {
                return Some(*fw);
            }
        }

        // Then frontend
        for fw in frameworks {
            if fw.category() == FrameworkCategory::Frontend {
                return Some(*fw);
            }
        }

        // Any framework
        frameworks.iter().next().copied()
    }

    /// Check if a framework is detected.
    pub fn has(&self, framework: Framework) -> bool {
        self.frameworks.contains(&framework)
    }

    /// Get frameworks by category.
    pub fn by_category(&self, category: FrameworkCategory) -> Vec<Framework> {
        self.frameworks
            .iter()
            .filter(|fw| fw.category() == category)
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_category() {
        assert_eq!(Framework::React.category(), FrameworkCategory::Frontend);
        assert_eq!(Framework::NextJs.category(), FrameworkCategory::Fullstack);
        assert_eq!(Framework::Express.category(), FrameworkCategory::Backend);
        assert_eq!(Framework::Jest.category(), FrameworkCategory::Testing);
    }

    #[test]
    fn test_framework_language() {
        assert_eq!(Framework::React.language(), "javascript");
        assert_eq!(Framework::Django.language(), "python");
        assert_eq!(Framework::Rails.language(), "ruby");
        assert_eq!(Framework::SpringBoot.language(), "java");
        assert_eq!(Framework::ActixWeb.language(), "rust");
    }

    #[test]
    fn test_framework_filter_builder() {
        let filter = FrameworkFilter::new()
            .uses(Framework::React)
            .uses(Framework::Express)
            .excludes(Framework::Vue)
            .category(FrameworkCategory::Testing);

        assert!(filter.required.contains(&Framework::React));
        assert!(filter.required.contains(&Framework::Express));
        assert!(filter.excluded.contains(&Framework::Vue));
        assert!(
            filter
                .required_categories
                .contains(&FrameworkCategory::Testing)
        );
    }

    #[test]
    fn test_framework_detection_packages() {
        assert!(Framework::React.detection_packages().contains(&"react"));
        assert!(Framework::Django.detection_packages().contains(&"django"));
        assert!(
            Framework::ActixWeb
                .detection_packages()
                .contains(&"actix-web")
        );
    }

    #[test]
    fn test_determine_primary() {
        let mut frameworks = HashSet::new();
        frameworks.insert(Framework::React);
        frameworks.insert(Framework::NextJs);

        let primary = FrameworkInfo::determine_primary(&frameworks);
        assert_eq!(primary, Some(Framework::NextJs)); // Fullstack preferred
    }
}
