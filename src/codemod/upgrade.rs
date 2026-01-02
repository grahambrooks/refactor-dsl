//! Upgrade definitions and pre-built migrations.

use crate::matcher::Matcher;
use crate::transform::TransformBuilder;

/// A composable upgrade/migration that can be applied to repositories.
///
/// Upgrades bundle together matching criteria and transformations, making
/// them reusable across different codemod executions.
///
/// # Example
///
/// ```rust
/// use refactor_dsl::codemod::Upgrade;
/// use refactor_dsl::matcher::Matcher;
/// use refactor_dsl::transform::TransformBuilder;
///
/// struct MyUpgrade;
///
/// impl Upgrade for MyUpgrade {
///     fn name(&self) -> &str {
///         "my-upgrade"
///     }
///
///     fn description(&self) -> &str {
///         "My custom upgrade"
///     }
///
///     fn matcher(&self) -> Matcher {
///         Matcher::new()
///             .files(|f| f.extension("rs"))
///     }
///
///     fn transform(&self) -> TransformBuilder {
///         TransformBuilder::new()
///             .replace_literal("old_api", "new_api")
///     }
/// }
/// ```
pub trait Upgrade: Send + Sync {
    /// Unique name for this upgrade.
    fn name(&self) -> &str;

    /// Human-readable description of what this upgrade does.
    fn description(&self) -> &str;

    /// The matching criteria for files this upgrade applies to.
    fn matcher(&self) -> Matcher;

    /// The transformations to apply.
    fn transform(&self) -> TransformBuilder;
}

/// Angular v4/v5 to v15+ upgrade.
///
/// Migrates:
/// - `@angular/http` to `@angular/common/http`
/// - `Http` to `HttpClient`
/// - RxJS import paths to barrel imports
/// - Removes `.json()` calls (HttpClient auto-parses)
pub struct AngularV4V5Upgrade;

impl Upgrade for AngularV4V5Upgrade {
    fn name(&self) -> &str {
        "angular-v4v5-upgrade"
    }

    fn description(&self) -> &str {
        "Upgrade Angular applications from v4/v5 to v15+ (HttpModule -> HttpClientModule, RxJS updates)"
    }

    fn matcher(&self) -> Matcher {
        Matcher::new().files(|f| {
            f.extensions(["ts", "tsx"])
                .exclude("**/node_modules/**")
                .exclude("**/*.spec.ts")
                .exclude("**/*.test.ts")
        })
    }

    fn transform(&self) -> TransformBuilder {
        TransformBuilder::new()
            // HttpModule -> HttpClientModule imports
            .replace_literal(
                "import { HttpModule } from '@angular/http';",
                "import { HttpClientModule } from '@angular/common/http';",
            )
            .replace_literal(
                "import { Http, Headers, RequestOptions, Response } from '@angular/http';",
                "import { HttpClient, HttpHeaders, HttpResponse, HttpErrorResponse } from '@angular/common/http';",
            )
            .replace_literal(
                "import { Http, Response } from '@angular/http';",
                "import { HttpClient, HttpResponse } from '@angular/common/http';",
            )
            .replace_literal(
                "import { Http } from '@angular/http';",
                "import { HttpClient } from '@angular/common/http';",
            )
            // Module array updates
            .replace_literal("HttpModule,", "HttpClientModule,")
            .replace_literal("HttpModule", "HttpClientModule")
            // Type updates
            .replace_literal(": Http)", ": HttpClient)")
            .replace_literal(": Http,", ": HttpClient,")
            // RxJS barrel imports
            .replace_literal(
                "import { Observable } from 'rxjs/Observable';",
                "import { Observable } from 'rxjs';",
            )
            .replace_literal(
                "import { Subject } from 'rxjs/Subject';",
                "import { Subject } from 'rxjs';",
            )
            .replace_literal(
                "import { BehaviorSubject } from 'rxjs/BehaviorSubject';",
                "import { BehaviorSubject } from 'rxjs';",
            )
            .replace_literal(
                "import { ReplaySubject } from 'rxjs/ReplaySubject';",
                "import { ReplaySubject } from 'rxjs';",
            )
            .replace_literal(
                "import { Subscription } from 'rxjs/Subscription';",
                "import { Subscription } from 'rxjs';",
            )
            // Remove patched operator imports
            .replace_pattern(r"import 'rxjs/add/operator/\w+';[\r\n]*", "")
            .replace_pattern(r"import 'rxjs/add/observable/\w+';[\r\n]*", "")
            // Remove .json() calls (HttpClient auto-parses)
            .replace_pattern(
                r"\.map\(\s*\(?\s*res(?:ponse)?\s*:?\s*Response\s*\)?\s*=>\s*res(?:ponse)?\.json\(\)\s*\)",
                "",
            )
    }
}

/// Convenience function to create an Angular upgrade.
pub fn angular_v4v5_upgrade() -> AngularV4V5Upgrade {
    AngularV4V5Upgrade
}

/// RxJS v5 to v6+ migration.
///
/// Migrates:
/// - Deep imports to barrel imports
/// - `Observable.of/from/etc` to standalone functions
/// - Removes patched operator imports
pub struct RxJS5To6Upgrade;

impl Upgrade for RxJS5To6Upgrade {
    fn name(&self) -> &str {
        "rxjs-5-to-6"
    }

    fn description(&self) -> &str {
        "Migrate RxJS from v5 to v6+ with pipeable operators"
    }

    fn matcher(&self) -> Matcher {
        Matcher::new().files(|f| {
            f.extensions(["ts", "js", "tsx", "jsx"])
                .exclude("**/node_modules/**")
        })
    }

    fn transform(&self) -> TransformBuilder {
        TransformBuilder::new()
            // Deep imports to barrel imports
            .replace_literal(
                "import { Observable } from 'rxjs/Observable';",
                "import { Observable } from 'rxjs';",
            )
            .replace_literal(
                "import { Subject } from 'rxjs/Subject';",
                "import { Subject } from 'rxjs';",
            )
            .replace_literal(
                "import { BehaviorSubject } from 'rxjs/BehaviorSubject';",
                "import { BehaviorSubject } from 'rxjs';",
            )
            .replace_literal(
                "import { ReplaySubject } from 'rxjs/ReplaySubject';",
                "import { ReplaySubject } from 'rxjs';",
            )
            .replace_literal(
                "import { AsyncSubject } from 'rxjs/AsyncSubject';",
                "import { AsyncSubject } from 'rxjs';",
            )
            .replace_literal(
                "import { Subscription } from 'rxjs/Subscription';",
                "import { Subscription } from 'rxjs';",
            )
            // Static method migrations
            .replace_literal("Observable.of(", "of(")
            .replace_literal("Observable.from(", "from(")
            .replace_literal("Observable.fromEvent(", "fromEvent(")
            .replace_literal("Observable.fromPromise(", "from(")
            .replace_literal("Observable.interval(", "interval(")
            .replace_literal("Observable.timer(", "timer(")
            .replace_literal("Observable.combineLatest(", "combineLatest(")
            .replace_literal("Observable.forkJoin(", "forkJoin(")
            .replace_literal("Observable.merge(", "merge(")
            .replace_literal("Observable.concat(", "concat(")
            .replace_literal("Observable.zip(", "zip(")
            .replace_literal("Observable.race(", "race(")
            .replace_literal("Observable.empty()", "EMPTY")
            .replace_literal("Observable.never()", "NEVER")
            .replace_literal("Observable.throw(", "throwError(")
            // Remove patched imports
            .replace_pattern(r"import 'rxjs/add/operator/\w+';[\r\n]*", "")
            .replace_pattern(r"import 'rxjs/add/observable/\w+';[\r\n]*", "")
    }
}

/// Convenience function to create an RxJS upgrade.
pub fn rxjs_5_to_6_upgrade() -> RxJS5To6Upgrade {
    RxJS5To6Upgrade
}

/// A closure-based upgrade for quick custom transformations.
///
/// # Example
///
/// ```rust
/// use refactor_dsl::codemod::ClosureUpgrade;
/// use refactor_dsl::matcher::Matcher;
/// use refactor_dsl::transform::TransformBuilder;
///
/// let upgrade = ClosureUpgrade::new(
///     "rename-api",
///     "Rename old_api to new_api",
///     || Matcher::new().files(|f| f.extension("rs")),
///     || TransformBuilder::new().replace_literal("old_api", "new_api"),
/// );
/// ```
pub struct ClosureUpgrade<M, T>
where
    M: Fn() -> Matcher + Send + Sync,
    T: Fn() -> TransformBuilder + Send + Sync,
{
    name: String,
    description: String,
    matcher_fn: M,
    transform_fn: T,
}

impl<M, T> ClosureUpgrade<M, T>
where
    M: Fn() -> Matcher + Send + Sync,
    T: Fn() -> TransformBuilder + Send + Sync,
{
    /// Create a new closure-based upgrade.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        matcher_fn: M,
        transform_fn: T,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            matcher_fn,
            transform_fn,
        }
    }
}

impl<M, T> Upgrade for ClosureUpgrade<M, T>
where
    M: Fn() -> Matcher + Send + Sync,
    T: Fn() -> TransformBuilder + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn matcher(&self) -> Matcher {
        (self.matcher_fn)()
    }

    fn transform(&self) -> TransformBuilder {
        (self.transform_fn)()
    }
}
