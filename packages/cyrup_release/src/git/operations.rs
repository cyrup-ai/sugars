//! Git operations using CLI commands via tokio::process

use crate::error::{Result, GitError};
use semver::Version;
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Trait defining required Git operations for release management
#[async_trait::async_trait]
pub trait GitOperations {
    async fn create_release_commit(&self, version: &Version, message: Option<String>) -> Result<CommitInfo>;
    async fn create_version_tag(&self, version: &Version, message: Option<String>) -> Result<TagInfo>;
    async fn push_to_remote(&self, remote_name: Option<&str>, push_tags: bool) -> Result<PushInfo>;
    async fn is_working_directory_clean(&self) -> Result<bool>;
    async fn get_current_branch(&self) -> Result<BranchInfo>;
    async fn tag_exists(&self, tag_name: &str) -> Result<bool>;
    async fn delete_tag(&self, tag_name: &str, delete_remote: bool) -> Result<()>;
    async fn reset_to_commit(&self, commit_id: &str, reset_type: ResetType) -> Result<()>;
    async fn get_recent_commits(&self, count: usize) -> Result<Vec<CommitInfo>>;
    async fn get_remotes(&self) -> Result<Vec<RemoteInfo>>;
    async fn validate_release_readiness(&self) -> Result<ValidationResult>;
}

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub parents: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TagInfo {
    pub name: String,
    pub message: Option<String>,
    pub target_commit: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub is_annotated: bool,
}

#[derive(Debug, Clone)]
pub struct PushInfo {
    pub remote_name: String,
    pub commits_pushed: usize,
    pub tags_pushed: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_head: bool,
    pub upstream: Option<String>,
    pub commit_hash: String,
}

#[derive(Debug, Clone)]
pub struct RemoteInfo {
    pub name: String,
    pub fetch_url: String,
    pub push_url: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ResetType {
    Soft,
    Mixed,
    Hard,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<String>,
}

/// Git repository handle using CLI commands
#[derive(Debug, Clone)]
pub struct GitRepository {
    repo_path: PathBuf,
}

impl GitRepository {
    pub fn discover(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        Ok(Self {
            repo_path: path.to_path_buf(),
        })
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::discover(path)
    }

    pub fn repo_path(&self) -> &Path {
        &self.repo_path
    }

    async fn run_git(&self, args: &[&str]) -> Result<std::process::Output> {
        Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .output()
            .await
            .map_err(|e| GitError::OperationFailed {
                operation: format!("git {}", args.join(" ")),
                reason: format!("Failed to execute: {}", e),
            }.into())
    }

    async fn run_git_checked(&self, args: &[&str]) -> Result<String> {
        let output = self.run_git(args).await?;
        
        if !output.status.success() {
            return Err(GitError::OperationFailed {
                operation: format!("git {}", args.join(" ")),
                reason: String::from_utf8_lossy(&output.stderr).to_string(),
            }.into());
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

#[async_trait::async_trait]
impl GitOperations for GitRepository {
    async fn create_release_commit(&self, version: &Version, message: Option<String>) -> Result<CommitInfo> {
        let commit_message = message.unwrap_or_else(|| format!("release: v{}", version));
        
        // Stage all changes
        self.run_git_checked(&["add", "-A"]).await?;
        
        // Create commit
        self.run_git_checked(&["commit", "-m", &commit_message]).await?;
        
        // Get commit info
        let output = self.run_git_checked(&["log", "-1", "--format=%H%n%h%n%s%n%an%n%ae"]).await?;
        let lines: Vec<&str> = output.lines().collect();
        
        if lines.len() < 5 {
            return Err(GitError::OperationFailed {
                operation: "get commit info".to_string(),
                reason: "Unexpected git log output format".to_string(),
            }.into());
        }
        
        Ok(CommitInfo {
            hash: lines[0].to_string(),
            short_hash: lines[1].to_string(),
            message: lines[2].to_string(),
            author_name: lines[3].to_string(),
            author_email: lines[4].to_string(),
            timestamp: chrono::Utc::now(),
            parents: Vec::new(),
        })
    }

    async fn create_version_tag(&self, version: &Version, message: Option<String>) -> Result<TagInfo> {
        let tag_name = format!("v{}", version);
        let tag_message = message.unwrap_or_else(|| format!("Release v{}", version));
        
        // Create annotated tag
        self.run_git_checked(&["tag", "-a", &tag_name, "-m", &tag_message]).await?;
        
        // Get tag info
        let commit = self.run_git_checked(&["rev-parse", &tag_name]).await?;
        
        Ok(TagInfo {
            name: tag_name,
            message: Some(tag_message),
            target_commit: commit,
            timestamp: chrono::Utc::now(),
            is_annotated: true,
        })
    }

    async fn push_to_remote(&self, remote_name: Option<&str>, push_tags: bool) -> Result<PushInfo> {
        let remote = remote_name.unwrap_or("origin");
        
        // Push commits
        self.run_git_checked(&["push", remote, "HEAD"]).await?;
        
        let mut tags_pushed = 0;
        if push_tags {
            // Push tags
            self.run_git_checked(&["push", remote, "--tags"]).await?;
            tags_pushed = 1; // Simplified
        }
        
        Ok(PushInfo {
            remote_name: remote.to_string(),
            commits_pushed: 1, // Simplified
            tags_pushed,
            warnings: Vec::new(),
        })
    }

    async fn is_working_directory_clean(&self) -> Result<bool> {
        let output = self.run_git_checked(&["status", "--porcelain"]).await?;
        Ok(output.is_empty())
    }

    async fn get_current_branch(&self) -> Result<BranchInfo> {
        let name = self.run_git_checked(&["branch", "--show-current"]).await?;
        let commit_hash = self.run_git_checked(&["rev-parse", "HEAD"]).await?;
        
        let upstream = self.run_git(&["rev-parse", "--abbrev-ref", "@{upstream}"])
            .await
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    None
                }
            });
        
        Ok(BranchInfo {
            name,
            is_head: true,
            upstream,
            commit_hash,
        })
    }

    async fn tag_exists(&self, tag_name: &str) -> Result<bool> {
        let output = self.run_git(&["tag", "-l", tag_name]).await?;
        Ok(output.status.success() && !String::from_utf8_lossy(&output.stdout).trim().is_empty())
    }

    async fn delete_tag(&self, tag_name: &str, delete_remote: bool) -> Result<()> {
        // Delete local tag
        self.run_git_checked(&["tag", "-d", tag_name]).await?;
        
        if delete_remote {
            // Delete remote tag
            self.run_git_checked(&["push", "origin", &format!(":{}", tag_name)]).await?;
        }
        
        Ok(())
    }

    async fn reset_to_commit(&self, commit_id: &str, reset_type: ResetType) -> Result<()> {
        let reset_arg = match reset_type {
            ResetType::Soft => "--soft",
            ResetType::Mixed => "--mixed",
            ResetType::Hard => "--hard",
        };
        
        self.run_git_checked(&["reset", reset_arg, commit_id]).await?;
        Ok(())
    }

    async fn get_recent_commits(&self, count: usize) -> Result<Vec<CommitInfo>> {
        let output = self.run_git_checked(&[
            "log",
            &format!("-{}", count),
            "--format=%H%x00%h%x00%s%x00%an%x00%ae%x00"
        ]).await?;
        
        let mut commits = Vec::new();
        for entry in output.split('\n').filter(|s| !s.is_empty()) {
            let parts: Vec<&str> = entry.split('\0').collect();
            if parts.len() >= 5 {
                commits.push(CommitInfo {
                    hash: parts[0].to_string(),
                    short_hash: parts[1].to_string(),
                    message: parts[2].to_string(),
                    author_name: parts[3].to_string(),
                    author_email: parts[4].to_string(),
                    timestamp: chrono::Utc::now(),
                    parents: Vec::new(),
                });
            }
        }
        
        Ok(commits)
    }

    async fn get_remotes(&self) -> Result<Vec<RemoteInfo>> {
        let output = self.run_git_checked(&["remote", "-v"]).await?;
        
        let mut remotes = std::collections::HashMap::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let name = parts[0];
                let url = parts[1];
                let remote_type = parts[2].trim_matches(|c| c == '(' || c == ')');
                
                let entry = remotes.entry(name.to_string()).or_insert_with(|| RemoteInfo {
                    name: name.to_string(),
                    fetch_url: String::new(),
                    push_url: String::new(),
                });
                
                if remote_type == "fetch" {
                    entry.fetch_url = url.to_string();
                } else if remote_type == "push" {
                    entry.push_url = url.to_string();
                }
            }
        }
        
        Ok(remotes.into_values().collect())
    }

    async fn validate_release_readiness(&self) -> Result<ValidationResult> {
        let mut issues = Vec::new();
        
        // Check if working directory is clean
        if !self.is_working_directory_clean().await? {
            issues.push("Working directory has uncommitted changes".to_string());
        }
        
        // Check if we have a remote
        let remotes = self.get_remotes().await?;
        if remotes.is_empty() {
            issues.push("No git remotes configured".to_string());
        }
        
        // Check if on a branch
        let branch = self.get_current_branch().await;
        if let Err(_) = branch {
            issues.push("Not on a branch (detached HEAD)".to_string());
        }
        
        Ok(ValidationResult {
            is_valid: issues.is_empty(),
            issues,
        })
    }
}

impl BranchInfo {
    pub fn commit_hash(&self) -> &str {
        &self.commit_hash
    }
}
