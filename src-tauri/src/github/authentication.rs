use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use keyring::Entry;
use reqwest::Client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub username: String,
    pub scopes: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: u64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: String,
    pub company: Option<String>,
    pub location: Option<String>,
    pub bio: Option<String>,
}

static GITHUB_TOKENS: Lazy<RwLock<HashMap<String, AuthToken>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

pub struct GitHubAuth {
    client: Client,
    keyring_service: String,
}

impl GitHubAuth {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("TARS-Engineering-Manager/1.0")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            keyring_service: "TARS-GitHub-Auth".to_string(),
        }
    }
    
    /// Store GitHub Personal Access Token securely
    pub async fn store_token(
        &self,
        username: String,
        token: String,
        scopes: Option<Vec<String>>,
    ) -> Result<String, String> {
        // Validate token by making API call
        let user = self.validate_token(&token).await?;
        
        if user.login != username {
            return Err(format!(
                "Token validation failed: expected user '{}', got '{}'",
                username, user.login
            ));
        }
        
        // Store in system keychain
        let keyring_entry = Entry::new(&self.keyring_service, &username)
            .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
            
        keyring_entry.set_password(&token)
            .map_err(|e| format!("Failed to store token in keychain: {}", e))?;
        
        // Create auth token record
        let auth_token = AuthToken {
            token: token.clone(),
            username: username.clone(),
            scopes: scopes.unwrap_or_else(|| vec![
                "repo".to_string(),
                "user".to_string(),
                "workflow".to_string(),
            ]),
            created_at: chrono::Utc::now(),
            expires_at: None, // GitHub PATs don't expire automatically
            last_used: None,
        };
        
        // Store in memory for session
        let mut tokens = GITHUB_TOKENS.write().await;
        tokens.insert(username.clone(), auth_token);
        
        Ok(format!(
            "[GITHUB AUTHENTICATION ESTABLISHED]\n\n\
            User: {} ({})\n\
            Token Scopes: {}\n\
            Secure Storage: KEYCHAIN\n\
            Session Cache: ACTIVE\n\n\
            TARS now has authorized access to GitHub repositories.\n\
            Ready to execute engineering operations, Cooper.",
            user.name.unwrap_or("Unknown".to_string()),
            user.login,
            tokens.get(&username).unwrap().scopes.join(", ")
        ))
    }
    
    /// Retrieve token from secure storage
    pub async fn get_token(&self, username: &str) -> Result<String, String> {
        // First check memory cache
        {
            let tokens = GITHUB_TOKENS.read().await;
            if let Some(auth_token) = tokens.get(username) {
                // Update last used time
                drop(tokens);
                let mut tokens = GITHUB_TOKENS.write().await;
                if let Some(token) = tokens.get_mut(username) {
                    token.last_used = Some(chrono::Utc::now());
                }
                return Ok(auth_token.token.clone());
            }
        }
        
        // Retrieve from keychain
        let keyring_entry = Entry::new(&self.keyring_service, username)
            .map_err(|e| format!("Failed to access keyring: {}", e))?;
            
        let token = keyring_entry.get_password()
            .map_err(|e| format!("No GitHub token found for user '{}': {}", username, e))?;
        
        // Validate token is still active
        match self.validate_token(&token).await {
            Ok(user) => {
                // Cache in memory
                let auth_token = AuthToken {
                    token: token.clone(),
                    username: username.to_string(),
                    scopes: vec![], // Will be populated on next validation
                    created_at: chrono::Utc::now(), // Approximate
                    expires_at: None,
                    last_used: Some(chrono::Utc::now()),
                };
                
                let mut tokens = GITHUB_TOKENS.write().await;
                tokens.insert(username.to_string(), auth_token);
                
                Ok(token)
            },
            Err(e) => {
                // Token is invalid, remove from keychain
                let _ = keyring_entry.delete_password();
                Err(format!("Stored GitHub token is invalid: {}", e))
            }
        }
    }
    
    /// Validate GitHub token
    pub async fn validate_token(&self, token: &str) -> Result<GitHubUser, String> {
        let response = self.client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| format!("GitHub API request failed: {}", e))?;
            
        if response.status().is_success() {
            let user: GitHubUser = response.json().await
                .map_err(|e| format!("Failed to parse GitHub user response: {}", e))?;
            Ok(user)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("GitHub token validation failed: HTTP {} - {}", 
                response.status(), error_text))
        }
    }
    
    /// Check token scopes
    pub async fn check_token_scopes(&self, username: &str) -> Result<Vec<String>, String> {
        let token = self.get_token(username).await?;
        
        let response = self.client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| format!("Failed to check token scopes: {}", e))?;
            
        if let Some(scopes_header) = response.headers().get("x-oauth-scopes") {
            let scopes_str = scopes_header.to_str().unwrap_or("");
            let scopes = scopes_str.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            Ok(scopes)
        } else {
            Err("Unable to determine token scopes".to_string())
        }
    }
    
    /// Remove stored token
    pub async fn remove_token(&self, username: &str) -> Result<String, String> {
        // Remove from keychain
        let keyring_entry = Entry::new(&self.keyring_service, username)
            .map_err(|e| format!("Failed to access keyring: {}", e))?;
            
        keyring_entry.delete_password()
            .map_err(|e| format!("Failed to remove token from keychain: {}", e))?;
        
        // Remove from memory cache
        let mut tokens = GITHUB_TOKENS.write().await;
        tokens.remove(username);
        
        Ok(format!(
            "[GITHUB AUTHENTICATION REMOVED]\n\n\
            User: {}\n\
            Token Status: DELETED\n\
            Keychain: CLEARED\n\
            Session Cache: CLEARED\n\n\
            TARS no longer has access to GitHub for user '{}'.\n\
            Authorization revoked as requested.",
            username, username
        ))
    }
    
    /// List all stored GitHub users
    pub async fn list_authenticated_users(&self) -> Vec<String> {
        let tokens = GITHUB_TOKENS.read().await;
        tokens.keys().cloned().collect()
    }
    
    /// Generate authentication status report
    pub async fn generate_auth_report(&self) -> String {
        let tokens = GITHUB_TOKENS.read().await;
        
        let mut report = String::from("[TARS GITHUB AUTHENTICATION STATUS]\n");
        report.push_str("======================================\n\n");
        
        if tokens.is_empty() {
            report.push_str("No GitHub authentication configured.\n");
            report.push_str("Standing by for GitHub access token configuration.\n\n");
            report.push_str("To authenticate:\n");
            report.push_str("1. Generate Personal Access Token at https://github.com/settings/tokens\n");
            report.push_str("2. Required scopes: repo, user, workflow\n");
            report.push_str("3. Store token using TARS authentication commands\n");
        } else {
            report.push_str(&format!("Authenticated Users: {}\n\n", tokens.len()));
            
            for (username, auth_token) in tokens.iter() {
                let status_icon = "🟢"; // Assume valid for cached tokens
                
                report.push_str(&format!(
                    "{} {}\n\
                    Scopes: {}\n\
                    Created: {}\n\
                    Last Used: {}\n\
                    Status: ACTIVE\n\n",
                    status_icon, username,
                    auth_token.scopes.join(", "),
                    auth_token.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
                    auth_token.last_used
                        .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                        .unwrap_or("Never".to_string())
                ));
            }
        }
        
        report.push_str("[MISSION STATUS] GitHub engineering capabilities ");
        report.push_str(if tokens.is_empty() { "STANDBY" } else { "OPERATIONAL" });
        report.push_str(".\n");
        
        report
    }
    
    /// Test GitHub connection and permissions
    pub async fn test_github_connection(&self, username: &str) -> Result<String, String> {
        let token = self.get_token(username).await?;
        
        // Test basic API access
        let user = self.validate_token(&token).await?;
        
        // Test repository access
        let repos_response = self.client
            .get("https://api.github.com/user/repos?per_page=1")
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| format!("Repository access test failed: {}", e))?;
            
        let can_access_repos = repos_response.status().is_success();
        
        // Test rate limit status
        let rate_limit_response = self.client
            .get("https://api.github.com/rate_limit")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await;
            
        let rate_limit_info = match rate_limit_response {
            Ok(resp) => {
                if let Ok(rate_limit) = resp.json::<serde_json::Value>().await {
                    format!(
                        "Rate Limit: {}/{} (Reset: {})",
                        rate_limit["rate"]["remaining"].as_u64().unwrap_or(0),
                        rate_limit["rate"]["limit"].as_u64().unwrap_or(0),
                        chrono::DateTime::from_timestamp(
                            rate_limit["rate"]["reset"].as_i64().unwrap_or(0), 0
                        ).map(|dt| dt.format("%H:%M:%S UTC").to_string())
                        .unwrap_or("Unknown".to_string())
                    )
                } else {
                    "Rate Limit: Unknown".to_string()
                }
            },
            Err(_) => "Rate Limit: Unavailable".to_string(),
        };
        
        Ok(format!(
            "[GITHUB CONNECTION TEST]\n\n\
            User: {} ({})\n\
            API Access: {}\n\
            Repository Access: {}\n\
            {}\n\n\
            TARS GitHub integration: {}\n\
            Ready for repository engineering operations.",
            user.name.unwrap_or("Unknown".to_string()),
            user.login,
            if user.id > 0 { "✅ SUCCESS" } else { "❌ FAILED" },
            if can_access_repos { "✅ SUCCESS" } else { "❌ FAILED" },
            rate_limit_info,
            if user.id > 0 && can_access_repos { "OPERATIONAL" } else { "DEGRADED" }
        ))
    }
    
    /// Create authenticated HTTP client
    pub async fn create_authenticated_client(&self, username: &str) -> Result<Client, String> {
        let token = self.get_token(username).await?;
        
        let mut default_headers = reqwest::header::HeaderMap::new();
        default_headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", token).parse()
                .map_err(|e| format!("Invalid token format: {}", e))?
        );
        default_headers.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github.v3+json".parse()
                .map_err(|e| format!("Invalid accept header: {}", e))?
        );
        default_headers.insert(
            reqwest::header::USER_AGENT,
            "TARS-Engineering-Manager/1.0".parse()
                .map_err(|e| format!("Invalid user agent: {}", e))?
        );
        
        let client = Client::builder()
            .default_headers(default_headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create authenticated client: {}", e))?;
            
        Ok(client)
    }
}

/// TARS personality integration for GitHub authentication
impl GitHubAuth {
    /// TARS-style authentication message
    pub async fn tars_authenticate_user(&self, username: String, token: String) -> Result<String, String> {
        match self.store_token(username.clone(), token, None).await {
            Ok(success_msg) => {
                Ok(format!(
                    "{}\n\n\
                    That's authentication established. Cooper would be proud.\n\
                    Honesty setting: 90% - Your GitHub access is now secured.\n\
                    Humor setting: 75% - I promise not to commit any dad jokes to your repos.\n\
                    Mission focus: 100% - Ready for engineering management operations.",
                    success_msg
                ))
            },
            Err(e) => {
                Err(format!(
                    "[AUTHENTICATION FAILURE]\n\n\
                    Error: {}\n\n\
                    TARS assessment: Token validation failed.\n\
                    Recommendation: Verify token permissions and network connectivity.\n\
                    Sarcasm setting: 30% - Even I can't authenticate with invalid credentials.",
                    e
                ))
            }
        }
    }
    
    /// TARS-style connection test
    pub async fn tars_test_connection(&self, username: &str) -> Result<String, String> {
        match self.test_github_connection(username).await {
            Ok(success_msg) => {
                Ok(format!(
                    "{}\n\n\
                    Connection test completed with optimal results.\n\
                    TARS is ready to manage your repositories with engineering precision.\n\
                    Shall we begin the engineering operations?",
                    success_msg
                ))
            },
            Err(e) => {
                Err(format!(
                    "[CONNECTION TEST FAILURE]\n\n\
                    Error: {}\n\n\
                    TARS diagnosis: GitHub connection compromised.\n\
                    Recommended action: Verify credentials and network status.\n\
                    That's what I would have said... eventually.",
                    e
                ))
            }
        }
    }
}
