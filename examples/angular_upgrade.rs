//! Example: Angular Version Upgrade Across Multiple GitHub Repositories
//!
//! This comprehensive example demonstrates how to use the DSL to perform
//! a major Angular version upgrade across multiple repositories from a
//! GitHub account, using:
//!
//! - Multi-repository cloning and management from GitHub
//! - LSP-based semantic refactoring (typescript-language-server)
//! - Import path migrations (@angular/http → @angular/common/http)
//! - API signature changes (Http → HttpClient)
//! - RxJS pipeable operator migration
//! - Batch processing with progress tracking
//!
//! This simulates the kind of large-scale, organization-wide refactoring
//! needed when upgrading Angular applications across multiple projects.

use refactor::prelude::*;
use std::path::Path;

fn main() -> Result<()> {
    println!("=== Multi-Repository Angular Upgrade ===\n");
    println!("Upgrading Angular projects from v4/5 to v15+ across GitHub repositories\n");

    // Phase 1: Multi-repo setup and discovery
    println!("Phase 1: Repository Discovery & Setup");
    println!("{}", "─".repeat(60));
    demonstrate_multi_repo_setup()?;

    // Phase 2: LSP-based semantic refactoring
    println!("\nPhase 2: LSP-Based Semantic Refactoring");
    println!("{}", "─".repeat(60));
    demonstrate_lsp_refactoring()?;

    // Phase 3: Batch text transformations
    println!("\nPhase 3: Batch Import & API Migrations");
    println!("{}", "─".repeat(60));
    demonstrate_batch_transforms()?;

    // Phase 4: Full multi-repo workflow
    println!("\nPhase 4: Complete Multi-Repository Workflow");
    println!("{}", "─".repeat(60));
    demonstrate_full_multi_repo_workflow()?;

    println!("\n=== Multi-Repository Upgrade Complete ===");
    Ok(())
}

/// Phase 1: Demonstrates setting up multi-repository operations with discovery
fn demonstrate_multi_repo_setup() -> Result<()> {
    println!("  GitHub Repository Discovery & Project Detection\n");

    // Show GitHub discovery code
    println!("  Step 1: Discover repositories from GitHub organization");
    println!("  ─────────────────────────────────────────────────────────");
    println!("  ```rust");
    println!("  use refactor::github::GitHubClient;");
    println!();
    println!("  let client = GitHubClient::new(github_token);");
    println!();
    println!("  // Discover all repositories from an organization");
    println!("  let repos = client.list_org_repos(\"acme-corp\")?;");
    println!();
    println!("  // Or discover from a user account");
    println!("  let repos = client.list_user_repos(\"username\")?;");
    println!();
    println!("  // Or search by topic/language");
    println!("  let repos = client.search_repos(\"org:acme-corp language:typescript\")?;");
    println!("  ```\n");

    // Simulated discovered repositories
    let workspace = std::path::Path::new("/tmp/angular-upgrade");
    let discovered_repos = vec![
        ("frontend-app", "main"),
        ("admin-dashboard", "main"),
        ("customer-portal", "develop"),
        ("shared-components", "main"),
        ("backend-api", "main"),
        ("mobile-app", "main"),
        ("design-system", "main"),
        ("data-pipeline", "main"),
    ];

    println!("  Step 2: Clone repositories to workspace");
    println!("  ─────────────────────────────────────────────────────────");
    println!("  Workspace: {}\n", workspace.display());

    for (name, branch) in &discovered_repos {
        println!("    • {:<22} (branch: {})", name, branch);
    }

    // Show the ProjectMatcher DSL
    println!("\n  Step 3: Filter using ProjectMatcher DSL");
    println!("  ─────────────────────────────────────────────────────────");
    println!("  ```rust");
    println!("  use refactor::matcher::ProjectMatcher;");
    println!();
    println!("  // Create a matcher to detect Angular projects");
    println!("  let angular_matcher = ProjectMatcher::new()");
    println!("      .has_angular()           // Checks for angular.json + @angular deps");
    println!("      .min_angular_version(4)  // Optional: minimum version");
    println!("      .max_angular_version(14) // Optional: projects needing upgrade");
    println!("      .exclude_archived();     // Skip archived repos");
    println!();
    println!("  // Filter discovered repositories");
    println!("  let angular_repos: Vec<_> = discovered_repos");
    println!("      .iter()");
    println!("      .filter(|repo| angular_matcher.matches(&repo.local_path).unwrap_or(false))");
    println!("      .collect();");
    println!("  ```\n");

    // Demonstrate the ProjectMatcher in action
    println!("  Step 4: ProjectMatcher Detection Results");
    println!("  ─────────────────────────────────────────────────────────\n");

    // Simulated detection results
    let detection_results = vec![
        DetectionResult {
            name: "frontend-app".to_string(),
            project_type: ProjectType::Angular,
            version: Some("12.2.0".to_string()),
            needs_upgrade: true,
        },
        DetectionResult {
            name: "admin-dashboard".to_string(),
            project_type: ProjectType::Angular,
            version: Some("11.0.0".to_string()),
            needs_upgrade: true,
        },
        DetectionResult {
            name: "customer-portal".to_string(),
            project_type: ProjectType::Angular,
            version: Some("14.0.0".to_string()),
            needs_upgrade: true,
        },
        DetectionResult {
            name: "shared-components".to_string(),
            project_type: ProjectType::Angular,
            version: Some("13.1.0".to_string()),
            needs_upgrade: true,
        },
        DetectionResult {
            name: "backend-api".to_string(),
            project_type: ProjectType::Node,
            version: None,
            needs_upgrade: false,
        },
        DetectionResult {
            name: "mobile-app".to_string(),
            project_type: ProjectType::React,
            version: Some("18.2.0".to_string()),
            needs_upgrade: false,
        },
        DetectionResult {
            name: "design-system".to_string(),
            project_type: ProjectType::Angular,
            version: Some("15.0.0".to_string()),
            needs_upgrade: false, // Already on 15+
        },
        DetectionResult {
            name: "data-pipeline".to_string(),
            project_type: ProjectType::Python,
            version: None,
            needs_upgrade: false,
        },
    ];

    println!("  ┌────────────────────────┬────────────┬──────────┬───────────────┐");
    println!("  │ Repository             │ Type       │ Version  │ Needs Upgrade │");
    println!("  ├────────────────────────┼────────────┼──────────┼───────────────┤");
    for result in &detection_results {
        let type_str = match result.project_type {
            ProjectType::Angular => "Angular",
            ProjectType::React => "React",
            ProjectType::Node => "Node.js",
            ProjectType::Python => "Python",
        };
        let version = result.version.as_deref().unwrap_or("-");
        let upgrade = if result.needs_upgrade { "Yes" } else { "No" };
        println!(
            "  │ {:<22} │ {:<10} │ {:<8} │ {:<13} │",
            result.name, type_str, version, upgrade
        );
    }
    println!("  └────────────────────────┴────────────┴──────────┴───────────────┘");

    let angular_count = detection_results
        .iter()
        .filter(|r| matches!(r.project_type, ProjectType::Angular) && r.needs_upgrade)
        .count();
    println!(
        "\n  Found {} Angular repositories requiring upgrade\n",
        angular_count
    );

    // Show the full ProjectMatcher API
    println!("  ProjectMatcher DSL - Full API:");
    println!("  ─────────────────────────────────────────────────────────");
    println!("  ```rust");
    println!("  /// Predicates for detecting project types and frameworks");
    println!("  impl ProjectMatcher {{");
    println!("      // Framework detection");
    println!("      pub fn has_angular(self) -> Self;      // angular.json + @angular/*");
    println!("      pub fn has_react(self) -> Self;        // react in package.json");
    println!("      pub fn has_vue(self) -> Self;          // vue in package.json");
    println!("      pub fn has_svelte(self) -> Self;       // svelte in package.json");
    println!("      pub fn has_nextjs(self) -> Self;       // next in package.json");
    println!();
    println!("      // Language detection");
    println!("      pub fn has_typescript(self) -> Self;   // tsconfig.json exists");
    println!("      pub fn has_rust(self) -> Self;         // Cargo.toml exists");
    println!("      pub fn has_python(self) -> Self;       // pyproject.toml/setup.py");
    println!("      pub fn has_go(self) -> Self;           // go.mod exists");
    println!();
    println!("      // Version constraints");
    println!("      pub fn min_angular_version(self, v: u32) -> Self;");
    println!("      pub fn max_angular_version(self, v: u32) -> Self;");
    println!("      pub fn dependency_version(self, pkg: &str, constraint: &str) -> Self;");
    println!();
    println!("      // Project structure");
    println!("      pub fn has_file(self, path: &str) -> Self;");
    println!("      pub fn has_dir(self, path: &str) -> Self;");
    println!("      pub fn package_json_contains(self, pattern: &str) -> Self;");
    println!();
    println!("      // Execute");
    println!("      pub fn matches(&self, path: &Path) -> Result<bool>;");
    println!("      pub fn detect(&self, path: &Path) -> Result<ProjectInfo>;");
    println!("  }}");
    println!("  ```\n");

    // Show has_angular implementation detail
    println!("  has_angular() Implementation:");
    println!("  ─────────────────────────────────────────────────────────");
    println!("  ```rust");
    println!("  /// Detects if a project is an Angular application");
    println!("  pub fn has_angular(mut self) -> Self {{");
    println!("      self.predicates.push(Box::new(|path: &Path| {{");
    println!("          // Check 1: angular.json or .angular-cli.json exists");
    println!("          let has_angular_json = path.join(\"angular.json\").exists()");
    println!("              || path.join(\".angular-cli.json\").exists();");
    println!();
    println!("          if !has_angular_json {{");
    println!("              return Ok(false);");
    println!("          }}");
    println!();
    println!("          // Check 2: package.json has @angular/core dependency");
    println!("          let package_json = path.join(\"package.json\");");
    println!("          if package_json.exists() {{");
    println!("              let content = std::fs::read_to_string(&package_json)?;");
    println!("              let pkg: serde_json::Value = serde_json::from_str(&content)?;");
    println!();
    println!("              // Check dependencies and devDependencies");
    println!("              let has_angular_core = [\"dependencies\", \"devDependencies\"]");
    println!("                  .iter()");
    println!("                  .filter_map(|k| pkg.get(k))");
    println!("                  .any(|deps| deps.get(\"@angular/core\").is_some());");
    println!();
    println!("              return Ok(has_angular_core);");
    println!("          }}");
    println!();
    println!("          Ok(false)");
    println!("      }}));");
    println!("      self");
    println!("  }}");
    println!("  ```\n");

    // Show combined workflow
    println!("  Complete Discovery & Filter Workflow:");
    println!("  ─────────────────────────────────────────────────────────");
    println!("  ```rust");
    println!("  // Discover repos from GitHub");
    println!("  let github = GitHubClient::new(token);");
    println!("  let repos = github.list_org_repos(\"acme-corp\")?;");
    println!();
    println!("  // Clone to workspace");
    println!("  let workspace = Path::new(\"/tmp/upgrade-workspace\");");
    println!("  for repo in &repos {{");
    println!("      let local_path = workspace.join(&repo.name);");
    println!("      if !local_path.exists() {{");
    println!("          git2::Repository::clone(&repo.clone_url, &local_path)?;");
    println!("      }}");
    println!("  }}");
    println!();
    println!("  // Filter to Angular projects needing upgrade");
    println!("  let matcher = ProjectMatcher::new()");
    println!("      .has_angular()");
    println!("      .max_angular_version(14);  // Find projects below v15");
    println!();
    println!("  let to_upgrade: Vec<_> = repos");
    println!("      .iter()");
    println!("      .filter(|r| {{");
    println!("          let path = workspace.join(&r.name);");
    println!("          matcher.matches(&path).unwrap_or(false)");
    println!("      }})");
    println!("      .collect();");
    println!();
    println!("  println!(\"Found {{}} repos to upgrade\", to_upgrade.len());");
    println!("  ```");

    Ok(())
}

/// Phase 2: Demonstrates LSP-based semantic refactoring
fn demonstrate_lsp_refactoring() -> Result<()> {
    println!("  Using TypeScript Language Server for semantic refactoring...\n");

    // Example TypeScript service file that needs refactoring
    let service_code = r#"
import { Injectable } from '@angular/core';
import { Http, Response } from '@angular/http';
import { Observable } from 'rxjs/Observable';

@Injectable()
export class UserService {
    constructor(private http: Http) {}

    getUsers(): Observable<User[]> {
        return this.http.get('/api/users')
            .map((res: Response) => res.json());
    }

    getUserById(userId: number): Observable<User> {
        return this.http.get(`/api/users/${userId}`)
            .map((res: Response) => res.json());
    }

    createUser(userData: CreateUserDto): Observable<User> {
        return this.http.post('/api/users', userData)
            .map((res: Response) => res.json());
    }

    updateUser(userId: number, userData: UpdateUserDto): Observable<User> {
        return this.http.put(`/api/users/${userId}`, userData)
            .map((res: Response) => res.json());
    }

    deleteUser(userId: number): Observable<void> {
        return this.http.delete(`/api/users/${userId}`)
            .map(() => undefined);
    }
}
"#;

    println!("  Original service code:");
    print_code_sample(service_code, 8);

    // Demonstrate LSP-based rename operations
    println!("\n  LSP Rename Operations:");
    println!("  ─────────────────────────────────────────────────────────");

    let renames = vec![
        LspRenameOp {
            file: "src/app/services/user.service.ts",
            old_symbol: "http",
            new_symbol: "httpClient",
            description: "Rename injected Http to HttpClient",
        },
        LspRenameOp {
            file: "src/app/services/user.service.ts",
            old_symbol: "getUsers",
            new_symbol: "fetchUsers",
            description: "Rename to more descriptive method name",
        },
        LspRenameOp {
            file: "src/app/services/user.service.ts",
            old_symbol: "getUserById",
            new_symbol: "fetchUserById",
            description: "Consistent naming convention",
        },
        LspRenameOp {
            file: "src/app/models/user.model.ts",
            old_symbol: "CreateUserDto",
            new_symbol: "CreateUserRequest",
            description: "Rename DTO to Request suffix",
        },
        LspRenameOp {
            file: "src/app/models/user.model.ts",
            old_symbol: "UpdateUserDto",
            new_symbol: "UpdateUserRequest",
            description: "Rename DTO to Request suffix",
        },
    ];

    for (i, rename) in renames.iter().enumerate() {
        println!(
            "  {}. {} -> {} ({})",
            i + 1,
            rename.old_symbol,
            rename.new_symbol,
            rename.description
        );
    }

    // Show LSP rename code
    println!("\n  LSP Rename Implementation:");
    println!("  ```rust");
    println!("  use refactor::lsp::{{LspRename, LspRegistry}};");
    println!();
    println!("  // Auto-install typescript-language-server if not present");
    println!("  let result = LspRename::find_symbol(");
    println!("      \"src/app/services/user.service.ts\",");
    println!("      \"http\",      // old name");
    println!("      \"httpClient\" // new name");
    println!("  )?");
    println!("  .root(\"/path/to/project\")");
    println!("  .auto_install()  // Auto-install LSP server if needed");
    println!("  .dry_run()       // Preview changes first");
    println!("  .execute()?;");
    println!();
    println!("  println!(\"Files affected: {{}}\", result.file_count());");
    println!("  println!(\"Total edits: {{}}\", result.edit_count());");
    println!("  println!(\"Diff:\\n{{}}\", result.diff()?);");
    println!("  ```");

    // Show semantic rename benefits
    println!("\n  Benefits of LSP-Based Renaming:");
    println!("  • Finds all references across the entire project");
    println!("  • Handles imports, exports, and re-exports correctly");
    println!("  • Renames in template files (.html) where applicable");
    println!("  • Updates string literals in decorators (@Component, etc.)");
    println!("  • Respects TypeScript's type system and scoping rules");

    // Show batch rename across multiple repos
    println!("\n  Batch LSP Rename Across Repositories:");
    println!("  ```rust");
    println!("  // Define common renames to apply across all repos");
    println!("  let common_renames = vec![");
    println!("      (\"Http\", \"HttpClient\"),");
    println!("      (\"Response\", \"HttpResponse\"),");
    println!("      (\"RequestOptions\", \"HttpRequestOptions\"),");
    println!("  ];");
    println!();
    println!("  for repo_path in &repo_paths {{");
    println!("      // Find all service files");
    println!("      let services = FileMatcher::new()");
    println!("          .extension(\"ts\")");
    println!("          .include(\"**/*.service.ts\")");
    println!("          .collect(repo_path)?;");
    println!();
    println!("      for service_file in services {{");
    println!("          for (old_name, new_name) in &common_renames {{");
    println!("              if let Ok(rename) = LspRename::find_symbol(");
    println!("                  &service_file, old_name, new_name");
    println!("              ) {{");
    println!("                  rename.root(repo_path).execute()?;");
    println!("              }}");
    println!("          }}");
    println!("      }}");
    println!("  }}");
    println!("  ```");

    Ok(())
}

/// Phase 3: Demonstrates batch text transformations
fn demonstrate_batch_transforms() -> Result<()> {
    println!("  Applying batch transformations across repositories...\n");

    // Define transformation chains for different file types
    println!("  Transformation Chains by File Type:");
    println!("  ─────────────────────────────────────────────────────────\n");

    // 1. Service files transformation
    let service_transform = TransformBuilder::new()
        // Update imports
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
        // Update RxJS imports
        .replace_literal(
            "import { Observable } from 'rxjs/Observable';",
            "import { Observable, throwError } from 'rxjs';",
        )
        .replace_pattern(
            r"import 'rxjs/add/operator/(\w+)';",
            "",  // Remove patched operator imports
        )
        // Add pipeable operators import after rxjs import
        .replace_literal(
            "import { Observable, throwError } from 'rxjs';",
            "import { Observable, throwError } from 'rxjs';\nimport { map, catchError, tap, switchMap, filter } from 'rxjs/operators';",
        )
        // Update constructor injection
        .replace_pattern(
            r"constructor\(private (\w+): Http\)",
            "constructor(private $1: HttpClient)",
        )
        // Update response handling - HttpClient auto-parses JSON
        .replace_pattern(
            r"\.map\(\(res(?:ponse)?: Response\) => res(?:ponse)?\.json\(\)\)",
            "",  // Remove - HttpClient handles JSON automatically
        )
        // Convert .catch() to pipe(catchError())
        .replace_pattern(
            r"\.catch\(([^)]+)\)",
            ".pipe(catchError($1))",
        );
    // Note: Complex patterns like .map().filter() chains would need
    // multi-pass processing or AST-based transforms for proper handling

    println!("  1. Service Files (*.service.ts):");
    println!(
        "     Operations: {} transformations",
        service_transform.describe().len()
    );
    for (i, desc) in service_transform.describe().iter().take(5).enumerate() {
        println!("       {}. {}", i + 1, truncate_string(desc, 60));
    }
    if service_transform.describe().len() > 5 {
        println!(
            "       ... and {} more",
            service_transform.describe().len() - 5
        );
    }

    // 2. Module files transformation
    let module_transform = TransformBuilder::new()
        .replace_literal(
            "import { HttpModule } from '@angular/http';",
            "import { HttpClientModule } from '@angular/common/http';",
        )
        .replace_literal("HttpModule,", "HttpClientModule,")
        .replace_literal("HttpModule", "HttpClientModule");

    println!("\n  2. Module Files (*.module.ts):");
    println!(
        "     Operations: {} transformations",
        module_transform.describe().len()
    );
    for desc in module_transform.describe() {
        println!("       • {}", truncate_string(&desc, 60));
    }

    // 3. Component files transformation
    let component_transform = TransformBuilder::new()
        // Update RxJS imports
        .replace_literal(
            "import { Subject } from 'rxjs/Subject';",
            "import { Subject } from 'rxjs';",
        )
        .replace_literal(
            "import { BehaviorSubject } from 'rxjs/BehaviorSubject';",
            "import { BehaviorSubject } from 'rxjs';",
        )
        .replace_literal(
            "import { Subscription } from 'rxjs/Subscription';",
            "import { Subscription } from 'rxjs';",
        )
        // Convert Observable.combineLatest to combineLatest
        .replace_literal("Observable.combineLatest(", "combineLatest([")
        .replace_literal("Observable.forkJoin(", "forkJoin(")
        .replace_literal("Observable.merge(", "merge(");

    println!("\n  3. Component Files (*.component.ts):");
    println!(
        "     Operations: {} transformations",
        component_transform.describe().len()
    );
    for desc in component_transform.describe() {
        println!("       • {}", truncate_string(&desc, 60));
    }

    // Show example transformation
    let example_service = r#"
import { Injectable } from '@angular/core';
import { Http, Response } from '@angular/http';
import { Observable } from 'rxjs/Observable';
import 'rxjs/add/operator/map';
import 'rxjs/add/operator/catch';

@Injectable()
export class DataService {
    constructor(private http: Http) {}

    getData(): Observable<Data[]> {
        return this.http.get('/api/data')
            .map((res: Response) => res.json())
            .catch(this.handleError);
    }
}
"#;

    println!("\n  Example Transformation:");
    println!("  ─────────────────────────────────────────────────────────");
    println!("\n  Before:");
    print_code_sample(example_service, 6);

    let result = service_transform.apply(example_service, Path::new("data.service.ts"))?;

    println!("\n  After:");
    print_code_sample(&result, 6);

    Ok(())
}

/// Phase 4: Full multi-repository workflow
fn demonstrate_full_multi_repo_workflow() -> Result<()> {
    println!("  Complete workflow for upgrading multiple repositories...\n");

    // Show the complete workflow
    println!("  ```rust");
    println!("  use refactor::prelude::*;");
    println!("  use refactor::lsp::LspRename;");
    println!("  use git2::Repository;");
    println!("  use std::path::Path;");
    println!();
    println!("  /// Upgrade all Angular projects in a GitHub organization");
    println!("  fn upgrade_organization(");
    println!("      org_name: &str,");
    println!("      github_token: &str,");
    println!("      workspace: &Path,");
    println!("  ) -> Result<UpgradeReport> {{");
    println!("      let mut report = UpgradeReport::new();");
    println!();
    println!("      // Step 1: Fetch repository list from GitHub API");
    println!("      let repos = fetch_org_repos(org_name, github_token)?;");
    println!("      ");
    println!("      // Step 2: Filter to Angular projects");
    println!("      let angular_repos: Vec<_> = repos.iter()");
    println!("          .filter(|r| r.topics.contains(&\"angular\".to_string()))");
    println!("          .collect();");
    println!();
    println!("      println!(\"Found {{}} Angular repositories\", angular_repos.len());");
    println!();
    println!("      // Step 3: Process each repository");
    println!("      for repo_info in angular_repos {{");
    println!("          let repo_path = workspace.join(&repo_info.name);");
    println!("          ");
    println!("          // Clone or update repository");
    println!("          if repo_path.exists() {{");
    println!("              pull_latest(&repo_path)?;");
    println!("          }} else {{");
    println!("              Repository::clone(&repo_info.clone_url, &repo_path)?;");
    println!("          }}");
    println!();
    println!("          // Create upgrade branch");
    println!("          create_branch(&repo_path, \"chore/angular-upgrade\")?;");
    println!();
    println!("          // Step 4: Run refactoring pipeline");
    println!("          let repo_result = upgrade_repository(&repo_path)?;");
    println!("          report.add_repo_result(&repo_info.name, repo_result);");
    println!();
    println!("          // Step 5: Commit and push changes");
    println!("          if repo_result.has_changes {{");
    println!("              commit_changes(&repo_path, \"chore: upgrade to Angular 15\")?;");
    println!("              push_branch(&repo_path, \"chore/angular-upgrade\")?;");
    println!("              ");
    println!("              // Step 6: Create pull request");
    println!("              create_pull_request(");
    println!("                  org_name,");
    println!("                  &repo_info.name,");
    println!("                  \"chore/angular-upgrade\",");
    println!("                  &repo_info.default_branch,");
    println!("                  github_token,");
    println!("              )?;");
    println!("          }}");
    println!("      }}");
    println!();
    println!("      Ok(report)");
    println!("  }}");
    println!();
    println!("  /// Upgrade a single repository");
    println!("  fn upgrade_repository(repo_path: &Path) -> Result<RepoUpgradeResult> {{");
    println!("      let mut result = RepoUpgradeResult::new();");
    println!();
    println!("      // Phase A: LSP-based semantic renames");
    println!("      println!(\"  Running LSP-based refactoring...\");");
    println!("      ");
    println!("      // Find all service files and rename Http -> HttpClient");
    println!("      let services = FileMatcher::new()");
    println!("          .extension(\"ts\")");
    println!("          .include(\"**/*.service.ts\")");
    println!("          .exclude(\"**/node_modules/**\")");
    println!("          .collect(repo_path)?;");
    println!();
    println!("      for service_file in &services {{");
    println!("          // Use LSP for semantic rename of constructor parameter");
    println!("          if let Ok(rename) = LspRename::find_symbol(");
    println!("              service_file, \"http\", \"httpClient\"");
    println!("          ) {{");
    println!("              let lsp_result = rename");
    println!("                  .root(repo_path)");
    println!("                  .auto_install()");
    println!("                  .execute()?;");
    println!("              ");
    println!("              result.lsp_edits += lsp_result.edit_count();");
    println!("              result.files_modified.extend(");
    println!("                  lsp_result.workspace_edit.affected_files()");
    println!("              );");
    println!("          }}");
    println!("      }}");
    println!();
    println!("      // Phase B: Text-based transformations");
    println!("      println!(\"  Running text transformations...\");");
    println!("      ");
    println!("      let transform = TransformBuilder::new()");
    println!("          // Import updates");
    println!("          .replace_literal(");
    println!("              \"import {{ Http }} from '@angular/http';\",");
    println!("              \"import {{ HttpClient }} from '@angular/common/http';\"");
    println!("          )");
    println!("          .replace_literal(");
    println!("              \"import {{ HttpModule }} from '@angular/http';\",");
    println!("              \"import {{ HttpClientModule }} from '@angular/common/http';\"");
    println!("          )");
    println!("          // RxJS updates");
    println!("          .replace_pattern(");
    println!("              r\"import 'rxjs/add/operator/\\w+';\",");
    println!("              \"\"");
    println!("          )");
    println!("          .replace_literal(");
    println!("              \"import {{ Observable }} from 'rxjs/Observable';\",");
    println!("              \"import {{ Observable }} from 'rxjs';\"");
    println!("          );");
    println!();
    println!("      // Apply to all TypeScript files");
    println!("      let ts_files = FileMatcher::new()");
    println!("          .extension(\"ts\")");
    println!("          .exclude(\"**/node_modules/**\")");
    println!("          .exclude(\"**/*.spec.ts\")");
    println!("          .collect(repo_path)?;");
    println!();
    println!("      for file in &ts_files {{");
    println!("          let content = std::fs::read_to_string(file)?;");
    println!("          let updated = transform.apply(&content, file)?;");
    println!("          ");
    println!("          if content != updated {{");
    println!("              std::fs::write(file, &updated)?;");
    println!("              result.text_edits += 1;");
    println!("              result.files_modified.insert(file.clone());");
    println!("          }}");
    println!("      }}");
    println!();
    println!("      result.has_changes = !result.files_modified.is_empty();");
    println!("      Ok(result)");
    println!("  }}");
    println!("  ```");

    // Show example output
    println!("\n  Example Execution Output:");
    println!("  ─────────────────────────────────────────────────────────");
    println!();
    println!("  $ cargo run --example angular_upgrade -- --org acme-corp --token $GITHUB_TOKEN");
    println!();
    println!("  Discovering repositories for acme-corp...");
    println!("  Found 12 repositories, 8 are Angular projects");
    println!();
    println!("  [1/8] Processing frontend-app...");
    println!("    ✓ Cloned to /tmp/angular-upgrade/frontend-app");
    println!("    ✓ Created branch: chore/angular-upgrade");
    println!("    ✓ LSP renames: 24 edits across 8 files");
    println!("    ✓ Text transforms: 45 edits across 12 files");
    println!("    ✓ Committed: chore: upgrade to Angular 15");
    println!("    ✓ Created PR #142: Angular 15 Upgrade");
    println!();
    println!("  [2/8] Processing admin-dashboard...");
    println!("    ✓ Cloned to /tmp/angular-upgrade/admin-dashboard");
    println!("    ✓ Created branch: chore/angular-upgrade");
    println!("    ✓ LSP renames: 18 edits across 6 files");
    println!("    ✓ Text transforms: 32 edits across 9 files");
    println!("    ✓ Committed: chore: upgrade to Angular 15");
    println!("    ✓ Created PR #87: Angular 15 Upgrade");
    println!();
    println!("  ... (6 more repositories)");
    println!();

    // Show summary table
    println!("  ┌─────────────────────────────────────────────────────────────────────┐");
    println!("  │                    Multi-Repository Upgrade Summary                 │");
    println!("  ├─────────────────────┬──────────┬────────────┬──────────┬────────────┤");
    println!("  │ Repository          │ LSP Edits│ Text Edits │ Files    │ PR Created │");
    println!("  ├─────────────────────┼──────────┼────────────┼──────────┼────────────┤");
    println!("  │ frontend-app        │ 24       │ 45         │ 15       │ #142       │");
    println!("  │ admin-dashboard     │ 18       │ 32         │ 11       │ #87        │");
    println!("  │ customer-portal     │ 31       │ 58         │ 22       │ #203       │");
    println!("  │ shared-components   │ 12       │ 28         │ 9        │ #45        │");
    println!("  │ mobile-app          │ 22       │ 41         │ 14       │ #78        │");
    println!("  │ analytics-dashboard │ 15       │ 35         │ 12       │ #156       │");
    println!("  │ settings-panel      │ 8        │ 19         │ 7        │ #34        │");
    println!("  │ notification-service│ 6        │ 14         │ 5        │ #22        │");
    println!("  ├─────────────────────┼──────────┼────────────┼──────────┼────────────┤");
    println!("  │ TOTAL               │ 136      │ 272        │ 95       │ 8 PRs      │");
    println!("  └─────────────────────┴──────────┴────────────┴──────────┴────────────┘");

    // Show additional LSP operations
    println!("\n  Additional LSP Operations Available:");
    println!("  ─────────────────────────────────────────────────────────");
    println!();
    println!("  1. Find All References:");
    println!("     ```rust");
    println!("     let refs = LspClient::find_references(");
    println!("         \"src/app/services/user.service.ts\",");
    println!("         Position::new(10, 15)  // line, column");
    println!("     )?;");
    println!("     println!(\"Found {{}} references\", refs.len());");
    println!("     ```");
    println!();
    println!("  2. Go to Definition:");
    println!("     ```rust");
    println!("     let definition = LspClient::goto_definition(");
    println!("         \"src/app/components/user.component.ts\",");
    println!("         Position::new(25, 20)");
    println!("     )?;");
    println!("     println!(\"Defined in: {{}}:{{}}\", definition.file, definition.line);");
    println!("     ```");
    println!();
    println!("  3. Workspace-wide Symbol Search:");
    println!("     ```rust");
    println!("     let symbols = LspClient::workspace_symbols(\"UserService\")?;");
    println!("     for sym in symbols {{");
    println!("         println!(\"{{}} in {{}}\", sym.name, sym.location.file);");
    println!("     }}");
    println!("     ```");

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper types and functions
// ─────────────────────────────────────────────────────────────────────────────

/// Detected project type
#[derive(Debug, Clone, Copy)]
enum ProjectType {
    Angular,
    React,
    Node,
    Python,
}

/// Result of project detection
struct DetectionResult {
    name: String,
    project_type: ProjectType,
    version: Option<String>,
    needs_upgrade: bool,
}

/// LSP rename operation details
#[allow(dead_code)]
struct LspRenameOp {
    file: &'static str,
    old_symbol: &'static str,
    new_symbol: &'static str,
    description: &'static str,
}

/// Prints a code sample with line numbers
fn print_code_sample(code: &str, max_lines: usize) {
    let lines: Vec<&str> = code.lines().collect();
    let total = lines.len();

    // Skip empty first line if present
    let start = if lines.first().map_or(false, |l| l.is_empty()) {
        1
    } else {
        0
    };

    let display_lines: Vec<_> = lines.iter().skip(start).take(max_lines).collect();

    for (i, line) in display_lines.iter().enumerate() {
        println!("     {:>3} │ {}", start + i + 1, line);
    }

    if total > start + max_lines {
        println!("         │ ... ({} more lines)", total - start - max_lines);
    }
}

/// Truncates a string to a maximum length
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
