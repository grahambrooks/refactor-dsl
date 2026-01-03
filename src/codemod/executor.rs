//! Codemod execution engine.

use crate::codemod::{Codemod, RepoInfo, Upgrade};
use crate::error::{RefactorError, Result};
use crate::git::{BranchOps, CommitOps, GitOps, PushOps};
use crate::github::{CreatePullRequest, PullRequestOps};
use crate::refactor::Refactor;
use std::collections::HashSet;
use std::path::PathBuf;

/// Status of a repository after codemod execution.
#[derive(Debug, Clone)]
pub enum RepoStatus {
    /// Successfully processed with changes.
    Success,
    /// Processed but no changes were needed.
    NoChanges,
    /// Skipped (didn't match filter or other criteria).
    Skipped(String),
    /// Failed to process.
    Failed(String),
}

/// Result for a single repository.
#[derive(Debug)]
pub struct RepoResult {
    pub name: String,
    pub full_name: String,
    pub path: PathBuf,
    pub status: RepoStatus,
    pub files_modified: usize,
    pub branch_created: Option<String>,
    pub pushed: bool,
    pub pr_url: Option<String>,
    pub errors: Vec<String>,
}

/// Summary of the codemod execution.
#[derive(Debug, Default)]
pub struct CodemodSummary {
    pub total_repos: usize,
    pub modified_repos: usize,
    pub skipped_repos: usize,
    pub failed_repos: usize,
    pub total_files_modified: usize,
    pub prs_created: usize,
}

/// Result of executing a codemod across repositories.
#[derive(Debug)]
pub struct CodemodResult {
    pub repo_results: Vec<RepoResult>,
    pub summary: CodemodSummary,
}

/// Executor for running codemods.
pub struct CodemodExecutor {
    config: Codemod,
}

impl CodemodExecutor {
    /// Create a new executor from a codemod configuration.
    pub fn new(config: Codemod) -> Self {
        Self { config }
    }

    /// Execute the codemod.
    pub fn execute(self) -> Result<CodemodResult> {
        let mut repo_results = Vec::new();
        let mut summary = CodemodSummary::default();

        // Create workspace if needed
        std::fs::create_dir_all(&self.config.workspace)?;

        // Get repositories from source
        let repos = self
            .config
            .source
            .get_repositories(&self.config.workspace)?;
        summary.total_repos = repos.len();

        for repo in repos {
            let result = self.process_repo(&repo);

            match &result.status {
                RepoStatus::Success => {
                    summary.modified_repos += 1;
                    summary.total_files_modified += result.files_modified;
                    if result.pr_url.is_some() {
                        summary.prs_created += 1;
                    }
                }
                RepoStatus::NoChanges => {
                    summary.skipped_repos += 1;
                }
                RepoStatus::Skipped(_) => {
                    summary.skipped_repos += 1;
                }
                RepoStatus::Failed(_) => {
                    summary.failed_repos += 1;
                }
            }

            repo_results.push(result);
        }

        Ok(CodemodResult {
            repo_results,
            summary,
        })
    }

    fn process_repo(&self, repo: &RepoInfo) -> RepoResult {
        let mut result = RepoResult {
            name: repo.name.clone(),
            full_name: repo.full_name.clone(),
            path: repo.local_path.clone(),
            status: RepoStatus::NoChanges,
            files_modified: 0,
            branch_created: None,
            pushed: false,
            pr_url: None,
            errors: Vec::new(),
        };

        // Check filter
        if let Some(filter) = &self.config.filter {
            match filter.matches(repo) {
                Ok(true) => {}
                Ok(false) => {
                    result.status = RepoStatus::Skipped("Did not match filter".into());
                    return result;
                }
                Err(e) => {
                    result.status = RepoStatus::Failed(format!("Filter error: {}", e));
                    return result;
                }
            }
        }

        // Open git repo
        let git = match GitOps::open(&repo.local_path) {
            Ok(g) => g.with_auth(self.config.git_auth.clone()),
            Err(e) => {
                result.status = RepoStatus::Failed(format!("Failed to open git repo: {}", e));
                return result;
            }
        };

        // Create branch if specified
        if let Some(branch_pattern) = &self.config.branch_pattern {
            let branch_name = self.expand_pattern(branch_pattern, repo);
            if let Err(e) = git.create_and_checkout(&branch_name) {
                result.status = RepoStatus::Failed(format!("Failed to create branch: {}", e));
                return result;
            }
            result.branch_created = Some(branch_name);
        }

        // Apply upgrades
        let mut total_modified = HashSet::new();
        for upgrade in &self.config.upgrades {
            match self.apply_upgrade(upgrade.as_ref(), repo) {
                Ok(files) => {
                    total_modified.extend(files);
                }
                Err(e) => {
                    result.errors.push(format!("{}: {}", upgrade.name(), e));
                }
            }
        }

        result.files_modified = total_modified.len();

        if total_modified.is_empty() {
            result.status = RepoStatus::NoChanges;
            return result;
        }

        // Skip actual git operations in dry-run mode
        if self.config.dry_run {
            result.status = RepoStatus::Success;
            return result;
        }

        // Commit changes
        if let Err(e) = git.stage_all() {
            result.status = RepoStatus::Failed(format!("Failed to stage changes: {}", e));
            return result;
        }

        let commit_message = self
            .config
            .commit_message
            .as_ref()
            .map(|m| self.expand_pattern(m, repo))
            .unwrap_or_else(|| format!("chore: apply {} upgrade(s)", self.config.upgrades.len()));

        if let Err(e) = git.commit(&commit_message) {
            result.status = RepoStatus::Failed(format!("Failed to commit: {}", e));
            return result;
        }

        // Push if enabled
        if self.config.push_enabled
            && let Some(branch) = &result.branch_created
        {
            if let Err(e) = git.push_with_upstream("origin", branch) {
                result.errors.push(format!("Push failed: {}", e));
            } else {
                result.pushed = true;
            }
        }

        // Create PR if enabled
        if self.config.create_pr_enabled
            && result.pushed
            && let Some(client) = self.config.source.github_client()
            && let (Some(title), Some(body)) = (&self.config.pr_title, &self.config.pr_body)
        {
            let title = self.expand_pattern(title, repo);
            let body = self.expand_pattern(body, repo);
            let head = result.branch_created.as_ref().unwrap();

            // Parse owner/repo from full_name
            let parts: Vec<&str> = repo.full_name.split('/').collect();
            if parts.len() == 2 {
                let pr_request = CreatePullRequest::new(&title, &body, head, &repo.default_branch);

                match client.create_pull_request(parts[0], parts[1], pr_request) {
                    Ok(pr) => {
                        result.pr_url = Some(pr.html_url);
                    }
                    Err(e) => {
                        result.errors.push(format!("PR creation failed: {}", e));
                    }
                }
            }
        }

        result.status = RepoStatus::Success;
        result
    }

    fn apply_upgrade(&self, upgrade: &dyn Upgrade, repo: &RepoInfo) -> Result<Vec<PathBuf>> {
        let matcher = upgrade.matcher();
        let transform = upgrade.transform();

        let refactor_result = Refactor::in_repo(&repo.local_path)
            .matching(|_| matcher)
            .transform(|_| transform)
            .dry_run() // We'll do the actual write ourselves
            .apply();

        match refactor_result {
            Ok(result) => {
                let modified_files: Vec<PathBuf> = result
                    .changes
                    .iter()
                    .filter(|c| c.is_modified())
                    .map(|c| c.path.clone())
                    .collect();

                // Apply changes if not in global dry-run
                if !self.config.dry_run {
                    for change in &result.changes {
                        if change.is_modified() {
                            change.apply()?;
                        }
                    }
                }

                Ok(modified_files)
            }
            Err(RefactorError::NoFilesMatched) => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }

    fn expand_pattern(&self, pattern: &str, repo: &RepoInfo) -> String {
        let today = chrono_lite_date();
        pattern
            .replace("{repo}", &repo.name)
            .replace("{date}", &today)
    }
}

/// Get current date in YYYY-MM-DD format without pulling in chrono.
fn chrono_lite_date() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let secs = duration.as_secs();
    let days = secs / 86400;

    // Approximate calculation (not accounting for leap seconds, but close enough)
    let mut year = 1970;
    let mut remaining_days = days;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let days_in_months: [u64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for days_in_month in days_in_months {
        if remaining_days < days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        month += 1;
    }

    let day = remaining_days + 1;

    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn is_leap_year(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}
