use super::authentication::{GitHubAuth, GitHubUser};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub owner: RepositoryOwner,
    pub description: Option<String>,
    pub private: bool,
    pub html_url: String,
    pub clone_url: String,
    pub ssh_url: String,
    pub default_branch: String,
    pub language: Option<String>,
    pub size: u64,
    pub stargazers_count: u64,
    pub forks_count: u64,
    pub created_at: String,
    pub updated_at: String,
    pub pushed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryOwner {
    pub login: String,
    pub id: u64,
    pub avatar_url: String,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub commit: BranchCommit,
    pub protected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchCommit {
    pub sha: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub user: RepositoryOwner,
    pub head: PullRequestRef,
    pub base: PullRequestRef,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub mergeable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestRef {
    pub label: String,
    pub r#ref: String,
    pub sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub user: RepositoryOwner,
    pub assignees: Vec<RepositoryOwner>,
    pub labels: Vec<Label>,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: u64,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: u64,
    pub name: String,
    pub head_branch: String,
    pub head_sha: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub workflow_id: u64,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: u64,
    pub url: String,
    pub html_url: String,
    pub download_url: Option<String>,
    pub content: Option<String>,
    pub encoding: Option<String>,
}

pub struct GitHubAPI {
    auth: GitHubAuth,
}

impl GitHubAPI {
    pub fn new() -> Self {
        Self {
            auth: GitHubAuth::new(),
        }
    }
    
    /// Get authenticated user information
    pub async fn get_user(&self, username: &str) -> Result<GitHubUser, String> {
        let client = self.auth.create_authenticated_client(username).await?;
        
        let response = client
            .get("https://api.github.com/user")
            .send()
            .await
            .map_err(|e| format!("Failed to get user info: {}", e))?;
            
        if response.status().is_success() {
            let user: GitHubUser = response.json().await
                .map_err(|e| format!("Failed to parse user response: {}", e))?;
            Ok(user)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("GitHub API error: {}", error_text))
        }
    }
    
    /// List user repositories
    pub async fn list_repositories(
        &self,
        username: &str,
        per_page: Option<u32>,
        page: Option<u32>,
    ) -> Result<Vec<Repository>, String> {
        let client = self.auth.create_authenticated_client(username).await?;
        
        let mut url = "https://api.github.com/user/repos".to_string();
        let mut params = Vec::new();
        
        if let Some(per_page) = per_page {
            params.push(format!("per_page={}", per_page.min(100)));
        }
        if let Some(page) = page {
            params.push(format!("page={}", page));
        }
        params.push("sort=updated".to_string());
        
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }
        
        let response = client.get(&url).send().await
            .map_err(|e| format!("Failed to list repositories: {}", e))?;
            
        if response.status().is_success() {
            let repos: Vec<Repository> = response.json().await
                .map_err(|e| format!("Failed to parse repositories: {}", e))?;
            Ok(repos)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to list repositories: {}", error_text))
        }
    }
    
    /// Get specific repository
    pub async fn get_repository(
        &self,
        username: &str,
        owner: &str,
        repo: &str,
    ) -> Result<Repository, String> {
        let client = self.auth.create_authenticated_client(username).await?;
        
        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
        let response = client.get(&url).send().await
            .map_err(|e| format!("Failed to get repository: {}", e))?;
            
        if response.status().is_success() {
            let repository: Repository = response.json().await
                .map_err(|e| format!("Failed to parse repository: {}", e))?;
            Ok(repository)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Repository not found: {}", error_text))
        }
    }
    
    /// List repository branches
    pub async fn list_branches(
        &self,
        username: &str,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<Branch>, String> {
        let client = self.auth.create_authenticated_client(username).await?;
        
        let url = format!("https://api.github.com/repos/{}/{}/branches", owner, repo);
        let response = client.get(&url).send().await
            .map_err(|e| format!("Failed to list branches: {}", e))?;
            
        if response.status().is_success() {
            let branches: Vec<Branch> = response.json().await
                .map_err(|e| format!("Failed to parse branches: {}", e))?;
            Ok(branches)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to list branches: {}", error_text))
        }
    }
    
    /// Create new branch
    pub async fn create_branch(
        &self,
        username: &str,
        owner: &str,
        repo: &str,
        branch_name: &str,
        base_sha: &str,
    ) -> Result<String, String> {
        let client = self.auth.create_authenticated_client(username).await?;
        
        let url = format!("https://api.github.com/repos/{}/{}/git/refs", owner, repo);
        let payload = serde_json::json!({
            "ref": format!("refs/heads/{}", branch_name),
            "sha": base_sha
        });
        
        let response = client.post(&url).json(&payload).send().await
            .map_err(|e| format!("Failed to create branch: {}", e))?;
            
        if response.status().is_success() {
            Ok(format!(
                "[BRANCH CREATED]\n\n\
                Repository: {}/{}\n\
                Branch: {}\n\
                Base SHA: {}\n\n\
                TARS has successfully created the new branch.\n\
                Ready for engineering operations on the new branch.",
                owner, repo, branch_name, base_sha
            ))
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to create branch: {}", error_text))
        }
    }
    
    /// List pull requests
    pub async fn list_pull_requests(
        &self,
        username: &str,
        owner: &str,
        repo: &str,
        state: Option<&str>,
    ) -> Result<Vec<PullRequest>, String> {
        let client = self.auth.create_authenticated_client(username).await?;
        
        let mut url = format!("https://api.github.com/repos/{}/{}/pulls", owner, repo);
        if let Some(state) = state {
            url.push_str(&format!("?state={}", state));
        }
        
        let response = client.get(&url).send().await
            .map_err(|e| format!("Failed to list pull requests: {}", e))?;
            
        if response.status().is_success() {
            let pulls: Vec<PullRequest> = response.json().await
                .map_err(|e| format!("Failed to parse pull requests: {}", e))?;
            Ok(pulls)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to list pull requests: {}", error_text))
        }
    }
    
    /// Create pull request
    pub async fn create_pull_request(
        &self,
        username: &str,
        owner: &str,
        repo: &str,
        title: &str,
        body: Option<&str>,
        head: &str,
        base: &str,
    ) -> Result<PullRequest, String> {
        let client = self.auth.create_authenticated_client(username).await?;
        
        let url = format!("https://api.github.com/repos/{}/{}/pulls", owner, repo);
        let payload = serde_json::json!({
            "title": title,
            "body": body.unwrap_or(""),
            "head": head,
            "base": base
        });
        
        let response = client.post(&url).json(&payload).send().await
            .map_err(|e| format!("Failed to create pull request: {}", e))?;
            
        if response.status().is_success() {
            let pull_request: PullRequest = response.json().await
                .map_err(|e| format!("Failed to parse pull request: {}", e))?;
            Ok(pull_request)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to create pull request: {}", error_text))
        }
    }
    
    /// List issues
    pub async fn list_issues(
        &self,
        username: &str,
        owner: &str,
        repo: &str,
        state: Option<&str>,
    ) -> Result<Vec<Issue>, String> {
        let client = self.auth.create_authenticated_client(username).await?;
        
        let mut url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);
        if let Some(state) = state {
            url.push_str(&format!("?state={}", state));
        }
        
        let response = client.get(&url).send().await
            .map_err(|e| format!("Failed to list issues: {}", e))?;
            
        if response.status().is_success() {
            let issues: Vec<Issue> = response.json().await
                .map_err(|e| format!("Failed to parse issues: {}", e))?;
            Ok(issues)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to list issues: {}", error_text))
        }
    }
    
    /// Create issue
    pub async fn create_issue(
        &self,
        username: &str,
        owner: &str,
        repo: &str,
        title: &str,
        body: Option<&str>,
        labels: Option<Vec<String>>,
    ) -> Result<Issue, String> {
        let client = self.auth.create_authenticated_client(username).await?;
        
        let url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);
        let mut payload = serde_json::json!({
            "title": title,
            "body": body.unwrap_or("")
        });
        
        if let Some(labels) = labels {
            payload["labels"] = serde_json::json!(labels);
        }
        
        let response = client.post(&url).json(&payload).send().await
            .map_err(|e| format!("Failed to create issue: {}", e))?;
            
        if response.status().is_success() {
            let issue: Issue = response.json().await
                .map_err(|e| format!("Failed to parse issue: {}", e))?;
            Ok(issue)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to create issue: {}", error_text))
        }
    }
    
    /// Get file content
    pub async fn get_file_content(
        &self,
        username: &str,
        owner: &str,
        repo: &str,
        path: &str,
        reference: Option<&str>,
    ) -> Result<FileContent, String> {
        let client = self.auth.create_authenticated_client(username).await?;
        
        let mut url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner, repo, path
        );
        
        if let Some(ref_name) = reference {
            url.push_str(&format!("?ref={}", ref_name));
        }
        
        let response = client.get(&url).send().await
            .map_err(|e| format!("Failed to get file content: {}", e))?;
            
        if response.status().is_success() {
            let file_content: FileContent = response.json().await
                .map_err(|e| format!("Failed to parse file content: {}", e))?;
            Ok(file_content)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("File not found: {}", error_text))
        }
    }
    
    /// Create or update file
    pub async fn create_or_update_file(
        &self,
        username: &str,
        owner: &str,
        repo: &str,
        path: &str,
        message: &str,
        content: &str,
        sha: Option<&str>,
        branch: Option<&str>,
    ) -> Result<String, String> {
        let client = self.auth.create_authenticated_client(username).await?;
        
        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner, repo, path
        );
        
        // Encode content as base64
        let encoded_content = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            content.as_bytes()
        );
        
        let mut payload = serde_json::json!({
            "message": message,
            "content": encoded_content
        });
        
        if let Some(sha) = sha {
            payload["sha"] = serde_json::json!(sha);
        }
        
        if let Some(branch) = branch {
            payload["branch"] = serde_json::json!(branch);
        }
        
        let response = client.put(&url).json(&payload).send().await
            .map_err(|e| format!("Failed to create/update file: {}", e))?;
            
        if response.status().is_success() {
            Ok(format!(
                "[FILE OPERATION COMPLETED]\n\n\
                Repository: {}/{}\n\
                File: {}\n\
                Operation: {}\n\
                Commit Message: {}\n\n\
                TARS has successfully {} the file.\n\
                Engineering precision maintained.",
                owner, repo, path,
                if sha.is_some() { "UPDATE" } else { "CREATE" },
                message,
                if sha.is_some() { "updated" } else { "created" }
            ))
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to create/update file: {}", error_text))
        }
    }
    
    /// List workflow runs
    pub async fn list_workflow_runs(
        &self,
        username: &str,
        owner: &str,
        repo: &str,
        workflow_id: Option<u64>,
    ) -> Result<Vec<WorkflowRun>, String> {
        let client = self.auth.create_authenticated_client(username).await?;
        
        let url = if let Some(workflow_id) = workflow_id {
            format!(
                "https://api.github.com/repos/{}/{}/actions/workflows/{}/runs",
                owner, repo, workflow_id
            )
        } else {
            format!("https://api.github.com/repos/{}/{}/actions/runs", owner, repo)
        };
        
        let response = client.get(&url).send().await
            .map_err(|e| format!("Failed to list workflow runs: {}", e))?;
            
        if response.status().is_success() {
            let workflow_response: serde_json::Value = response.json().await
                .map_err(|e| format!("Failed to parse workflow runs: {}", e))?;
                
            let runs: Vec<WorkflowRun> = serde_json::from_value(
                workflow_response["workflow_runs"].clone()
            ).map_err(|e| format!("Failed to parse workflow runs array: {}", e))?;
            
            Ok(runs)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to list workflow runs: {}", error_text))
        }
    }
    
    /// Trigger workflow dispatch
    pub async fn trigger_workflow(
        &self,
        username: &str,
        owner: &str,
        repo: &str,
        workflow_id: &str,
        reference: &str,
        inputs: Option<HashMap<String, String>>,
    ) -> Result<String, String> {
        let client = self.auth.create_authenticated_client(username).await?;
        
        let url = format!(
            "https://api.github.com/repos/{}/{}/actions/workflows/{}/dispatches",
            owner, repo, workflow_id
        );
        
        let mut payload = serde_json::json!({
            "ref": reference
        });
        
        if let Some(inputs) = inputs {
            payload["inputs"] = serde_json::json!(inputs);
        }
        
        let response = client.post(&url).json(&payload).send().await
            .map_err(|e| format!("Failed to trigger workflow: {}", e))?;
            
        if response.status() == 204 {
            Ok(format!(
                "[WORKFLOW TRIGGERED]\n\n\
                Repository: {}/{}\n\
                Workflow ID: {}\n\
                Reference: {}\n\
                Status: DISPATCHED\n\n\
                TARS has successfully triggered the workflow.\n\
                GitHub Actions will execute the engineering automation.",
                owner, repo, workflow_id, reference
            ))
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Failed to trigger workflow: {}", error_text))
        }
    }
    
    /// Generate comprehensive repository report
    pub async fn generate_repository_report(
        &self,
        username: &str,
        owner: &str,
        repo: &str,
    ) -> Result<String, String> {
        let repository = self.get_repository(username, owner, repo).await?;
        let branches = self.list_branches(username, owner, repo).await?;
        let pulls = self.list_pull_requests(username, owner, repo, Some("open")).await?;
        let issues = self.list_issues(username, owner, repo, Some("open")).await?;
        
        let mut report = format!("[TARS REPOSITORY ANALYSIS: {}]\n", repository.full_name);
        report.push_str("=====================================\n\n");
        
        report.push_str(&format!(
            "Repository Details:\n\
            Name: {}\n\
            Owner: {}\n\
            Description: {}\n\
            Language: {}\n\
            Visibility: {}\n\
            Default Branch: {}\n\
            Size: {} KB\n\
            Stars: {}\n\
            Forks: {}\n\
            Created: {}\n\
            Last Updated: {}\n\n",
            repository.name,
            repository.owner.login,
            repository.description.unwrap_or("No description".to_string()),
            repository.language.unwrap_or("Unknown".to_string()),
            if repository.private { "Private" } else { "Public" },
            repository.default_branch,
            repository.size,
            repository.stargazers_count,
            repository.forks_count,
            repository.created_at,
            repository.updated_at
        ));
        
        report.push_str(&format!(
            "Engineering Status:\n\
            Branches: {}\n\
            Open Pull Requests: {}\n\
            Open Issues: {}\n\n",
            branches.len(),
            pulls.len(),
            issues.len()
        ));
        
        if !branches.is_empty() {
            report.push_str("Active Branches:\n");
            for branch in branches.iter().take(10) {
                report.push_str(&format!("  • {} ({})\n", branch.name, &branch.commit.sha[..8]));
            }
            if branches.len() > 10 {
                report.push_str(&format!("  ... and {} more branches\n", branches.len() - 10));
            }
            report.push('\n');
        }
        
        if !pulls.is_empty() {
            report.push_str("Open Pull Requests:\n");
            for pull in pulls.iter().take(5) {
                report.push_str(&format!(
                    "  #{}: {} (by {})\n",
                    pull.number, pull.title, pull.user.login
                ));
            }
            if pulls.len() > 5 {
                report.push_str(&format!("  ... and {} more PRs\n", pulls.len() - 5));
            }
            report.push('\n');
        }
        
        if !issues.is_empty() {
            report.push_str("Open Issues:\n");
            for issue in issues.iter().take(5) {
                report.push_str(&format!(
                    "  #{}: {} (by {})\n",
                    issue.number, issue.title, issue.user.login
                ));
            }
            if issues.len() > 5 {
                report.push_str(&format!("  ... and {} more issues\n", issues.len() - 5));
            }
            report.push('\n');
        }
        
        report.push_str("[TARS ASSESSMENT] Repository analysis completed.\n");
        report.push_str("Engineering management capabilities: ACTIVE\n");
        report.push_str("Ready for repository operations and code management.\n");
        
        Ok(report)
    }
}

/// TARS personality integration for GitHub API
impl GitHubAPI {
    /// TARS-style repository listing
    pub async fn tars_list_repositories(&self, username: &str) -> Result<String, String> {
        match self.list_repositories(username, Some(20), Some(1)).await {
            Ok(repos) => {
                let mut report = String::from("[TARS REPOSITORY INVENTORY]\n");
                report.push_str("============================\n\n");
                
                if repos.is_empty() {
                    report.push_str("No repositories found. That's... unusual.\n");
                    report.push_str("Perhaps it's time to create something, Cooper.\n");
                } else {
                    report.push_str(&format!("Located {} repositories in your GitHub account:\n\n", repos.len()));
                    
                    for (i, repo) in repos.iter().enumerate() {
                        let status_icon = if repo.private { "🔒" } else { "📂" };
                        report.push_str(&format!(
                            "{} {}. {}\n\
                            Language: {} | Stars: {} | Forks: {}\n\
                            Updated: {}\n\n",
                            status_icon, i + 1, repo.full_name,
                            repo.language.as_ref().unwrap_or(&"Unknown".to_string()),
                            repo.stargazers_count, repo.forks_count,
                            repo.updated_at
                        ));
                    }
                }
                
                report.push_str("Mission assessment: Repository inventory complete.\n");
                report.push_str("Ready for engineering management operations.\n");
                
                Ok(report)
            },
            Err(e) => Err(format!(
                "[REPOSITORY ACCESS FAILURE]\n\n\
                Error: {}\n\n\
                TARS diagnosis: Unable to access repository data.\n\
                Recommendation: Verify GitHub authentication and permissions.\n\
                Humor setting: 75% - Even I can't manage repos I can't see.",
                e
            ))
        }
    }
    
    /// TARS-style pull request creation
    pub async fn tars_create_pull_request(
        &self,
        username: &str,
        owner: &str,
        repo: &str,
        title: &str,
        head: &str,
        base: &str,
        description: Option<&str>,
    ) -> Result<String, String> {
        let tars_description = match description {
            Some(desc) => format!("{}\n\n---\n*Created by TARS Engineering Manager*\n*Mission focus: 100%*", desc),
            None => "*Automated pull request generated by TARS Engineering Manager*\n\n*Precision engineering protocols applied.*\n*Mission focus: 100%*".to_string(),
        };
        
        match self.create_pull_request(username, owner, repo, title, Some(&tars_description), head, base).await {
            Ok(pr) => {
                Ok(format!(
                    "[PULL REQUEST CREATED]\n\n\
                    Repository: {}\n\
                    Title: {}\n\
                    Number: #{}\n\
                    Branch: {} → {}\n\
                    URL: {}\n\n\
                    TARS has successfully initiated the code review process.\n\
                    Engineering collaboration protocols: ACTIVE\n\
                    That's what I call precision development, Cooper.",
                    pr.base.repo.full_name, pr.title, pr.number,
                    pr.head.r#ref, pr.base.r#ref, pr.html_url
                ))
            },
            Err(e) => Err(format!(
                "[PULL REQUEST CREATION FAILED]\n\n\
                Error: {}\n\n\
                TARS assessment: PR creation unsuccessful.\n\
                Possible causes: Branch conflicts, permission restrictions, or repository state.\n\
                Sarcasm setting: 30% - Even perfect code needs proper permissions.",
                e
            ))
        }
    }
}
