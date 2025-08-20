use serde::{Deserialize, Serialize};
use std::process::Command;
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VSCodeInstance {
    pub pid: Option<u32>,
    pub workspace: Option<String>,
    pub extensions_dir: Option<String>,
    pub user_data_dir: Option<String>,
    pub status: VSCodeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VSCodeStatus {
    Running,
    Stopped,
    Starting,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VSCodeCommand {
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<String>,
}

pub struct VSCodeCLI {
    vscode_path: Option<String>,
    default_args: Vec<String>,
}

impl VSCodeCLI {
    pub fn new() -> Self {
        Self {
            vscode_path: None,
            default_args: vec![],
        }
    }
    
    /// Detect VS Code installation
    pub async fn detect_installation(&mut self) -> Result<String, String> {
        let possible_paths = if cfg!(target_os = "windows") {
            vec![
                "code",
                "code.exe",
                r"C:\Program Files\Microsoft VS Code\bin\code.cmd",
                r"C:\Program Files (x86)\Microsoft VS Code\bin\code.cmd",
                r"C:\Users\%USERNAME%\AppData\Local\Programs\Microsoft VS Code\bin\code.cmd",
            ]
        } else if cfg!(target_os = "macos") {
            vec![
                "code",
                "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code",
                "/usr/local/bin/code",
            ]
        } else {
            vec![
                "code",
                "/usr/bin/code",
                "/usr/local/bin/code",
                "/snap/code/current/usr/share/code/bin/code",
            ]
        };
        
        for path in possible_paths {
            let mut cmd = Command::new(path);
            cmd.arg("--version");
            
            match cmd.output() {
                Ok(output) => {
                    if output.status.success() {
                        let version = String::from_utf8_lossy(&output.stdout);
                        self.vscode_path = Some(path.to_string());
                        
                        return Ok(format!(
                            "[VS CODE DETECTED]\n\n\
                            Path: {}\n\
                            Version: {}\n\
                            Status: AVAILABLE\n\n\
                            TARS has located VS Code installation.\n\
                            Ready for development environment integration.",
                            path, version.lines().next().unwrap_or("Unknown")
                        ));
                    }
                },
                Err(_) => continue,
            }
        }
        
        Err("VS Code installation not found. Please install VS Code or add it to PATH.".to_string())
    }
    
    /// Get VS Code path
    pub fn get_vscode_path(&self) -> Result<&String, String> {
        self.vscode_path.as_ref()
            .ok_or_else(|| "VS Code path not detected. Run detect_installation() first.".to_string())
    }
    
    /// Open file or folder in VS Code
    pub async fn open(&self, path: &str, new_window: bool) -> Result<String, String> {
        let vscode_path = self.get_vscode_path()?;
        
        let mut cmd = Command::new(vscode_path);
        cmd.arg(path);
        
        if new_window {
            cmd.arg("--new-window");
        }
        
        match cmd.spawn() {
            Ok(mut child) => {
                // Don't wait for VS Code to exit, just confirm it started
                tokio::spawn(async move {
                    let _ = child.wait();
                });
                
                Ok(format!(
                    "[VS CODE LAUNCHED]\n\n\
                    Target: {}\n\
                    Mode: {}\n\
                    Status: OPENING\n\n\
                    TARS has successfully launched VS Code.\n\
                    Development environment is now accessible.",
                    path, if new_window { "New Window" } else { "Current Window" }
                ))
            },
            Err(e) => Err(format!("Failed to launch VS Code: {}", e)),
        }
    }
    
    /// Open workspace file in VS Code
    pub async fn open_workspace(&self, workspace_path: &str) -> Result<String, String> {
        let vscode_path = self.get_vscode_path()?;
        
        let mut cmd = Command::new(vscode_path);
        cmd.arg(workspace_path);
        
        match cmd.spawn() {
            Ok(mut child) => {
                tokio::spawn(async move {
                    let _ = child.wait();
                });
                
                Ok(format!(
                    "[VS CODE WORKSPACE OPENED]\n\n\
                    Workspace: {}\n\
                    Status: LOADING\n\n\
                    TARS has opened the workspace configuration.\n\
                    Multi-project development environment ready.",
                    workspace_path
                ))
            },
            Err(e) => Err(format!("Failed to open workspace: {}", e)),
        }
    }
    
    /// Install VS Code extension
    pub async fn install_extension(&self, extension_id: &str) -> Result<String, String> {
        let vscode_path = self.get_vscode_path()?;
        
        let mut cmd = Command::new(vscode_path);
        cmd.args(&["--install-extension", extension_id]);
        
        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    Ok(format!(
                        "[EXTENSION INSTALLED]\n\n\
                        Extension: {}\n\
                        Status: SUCCESS\n\
                        Output: {}\n\n\
                        TARS has successfully installed the VS Code extension.\n\
                        Development capabilities enhanced.",
                        extension_id, stdout
                    ))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(format!("Extension installation failed: {}", stderr))
                }
            },
            Err(e) => Err(format!("Failed to install extension: {}", e)),
        }
    }
    
    /// List installed extensions
    pub async fn list_extensions(&self) -> Result<Vec<String>, String> {
        let vscode_path = self.get_vscode_path()?;
        
        let mut cmd = Command::new(vscode_path);
        cmd.arg("--list-extensions");
        
        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let extensions: Vec<String> = stdout
                        .lines()
                        .filter(|line| !line.trim().is_empty())
                        .map(|line| line.trim().to_string())
                        .collect();
                    Ok(extensions)
                } else {
                    Err("Failed to list extensions".to_string())
                }
            },
            Err(e) => Err(format!("Failed to list extensions: {}", e)),
        }
    }
    
    /// Uninstall VS Code extension
    pub async fn uninstall_extension(&self, extension_id: &str) -> Result<String, String> {
        let vscode_path = self.get_vscode_path()?;
        
        let mut cmd = Command::new(vscode_path);
        cmd.args(&["--uninstall-extension", extension_id]);
        
        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(format!(
                        "[EXTENSION UNINSTALLED]\n\n\
                        Extension: {}\n\
                        Status: REMOVED\n\n\
                        TARS has successfully removed the VS Code extension.\n\
                        Development environment updated.",
                        extension_id
                    ))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(format!("Extension uninstallation failed: {}", stderr))
                }
            },
            Err(e) => Err(format!("Failed to uninstall extension: {}", e)),
        }
    }
    
    /// Execute VS Code with custom arguments
    pub async fn execute_command(&self, args: Vec<String>) -> Result<String, String> {
        let vscode_path = self.get_vscode_path()?;
        
        let mut cmd = Command::new(vscode_path);
        cmd.args(&args);
        
        match cmd.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                if output.status.success() {
                    Ok(format!(
                        "[VS CODE COMMAND EXECUTED]\n\n\
                        Arguments: {}\n\
                        Status: SUCCESS\n\
                        Output: {}\n\n\
                        TARS has executed the VS Code command successfully.",
                        args.join(" "), stdout
                    ))
                } else {
                    Err(format!(
                        "VS Code command failed.\n\
                        Arguments: {}\n\
                        Error: {}",
                        args.join(" "), stderr
                    ))
                }
            },
            Err(e) => Err(format!("Failed to execute VS Code command: {}", e)),
        }
    }
    
    /// Open difference comparison in VS Code
    pub async fn diff(&self, file1: &str, file2: &str) -> Result<String, String> {
        let vscode_path = self.get_vscode_path()?;
        
        let mut cmd = Command::new(vscode_path);
        cmd.args(&["--diff", file1, file2]);
        
        match cmd.spawn() {
            Ok(mut child) => {
                tokio::spawn(async move {
                    let _ = child.wait();
                });
                
                Ok(format!(
                    "[VS CODE DIFF OPENED]\n\n\
                    File 1: {}\n\
                    File 2: {}\n\
                    Mode: COMPARISON\n\n\
                    TARS has opened the file comparison in VS Code.\n\
                    Ready for code analysis and review.",
                    file1, file2
                ))
            },
            Err(e) => Err(format!("Failed to open diff: {}", e)),
        }
    }
    
    /// Open file at specific line and column
    pub async fn goto(&self, file: &str, line: u32, column: Option<u32>) -> Result<String, String> {
        let vscode_path = self.get_vscode_path()?;
        
        let location = if let Some(col) = column {
            format!("{}:{}:{}", file, line, col)
        } else {
            format!("{}:{}", file, line)
        };
        
        let mut cmd = Command::new(vscode_path);
        cmd.args(&["--goto", &location]);
        
        match cmd.spawn() {
            Ok(mut child) => {
                tokio::spawn(async move {
                    let _ = child.wait();
                });
                
                Ok(format!(
                    "[VS CODE NAVIGATION]\n\n\
                    File: {}\n\
                    Line: {}\n\
                    Column: {}\n\
                    Status: NAVIGATING\n\n\
                    TARS has navigated to the specified location.\n\
                    Precision code positioning achieved.",
                    file, line, column.unwrap_or(1)
                ))
            },
            Err(e) => Err(format!("Failed to navigate to location: {}", e)),
        }
    }
    
    /// Get VS Code process information
    pub async fn get_running_instances(&self) -> Result<Vec<VSCodeInstance>, String> {
        let mut instances = Vec::new();
        
        // Use platform-specific process detection
        let output = if cfg!(target_os = "windows") {
            Command::new("wmic")
                .args(&["process", "where", "name='Code.exe'", "get", "ProcessId,CommandLine"])
                .output()
        } else {
            Command::new("pgrep")
                .args(&["-f", "code"])
                .output()
        };
        
        match output {
            Ok(result) => {
                if result.status.success() {
                    let stdout = String::from_utf8_lossy(&result.stdout);
                    
                    for line in stdout.lines() {
                        if line.contains("code") || line.contains("Code") {
                            // Parse process information (simplified)
                            let instance = VSCodeInstance {
                                pid: None, // Would need more sophisticated parsing
                                workspace: None,
                                extensions_dir: None,
                                user_data_dir: None,
                                status: VSCodeStatus::Running,
                            };
                            instances.push(instance);
                        }
                    }
                }
            },
            Err(_) => {
                // Process detection not available, return empty list
            }
        }
        
        Ok(instances)
    }
    
    /// Create VS Code settings file
    pub async fn create_settings_file(
        &self,
        workspace_path: &str,
        settings: HashMap<String, serde_json::Value>,
    ) -> Result<String, String> {
        let settings_dir = PathBuf::from(workspace_path).join(".vscode");
        let settings_file = settings_dir.join("settings.json");
        
        // Create .vscode directory if it doesn't exist
        if let Err(e) = tokio::fs::create_dir_all(&settings_dir).await {
            return Err(format!("Failed to create .vscode directory: {}", e));
        }
        
        // Write settings file
        let settings_json = serde_json::to_string_pretty(&settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
            
        tokio::fs::write(&settings_file, settings_json).await
            .map_err(|e| format!("Failed to write settings file: {}", e))?;
        
        Ok(format!(
            "[VS CODE SETTINGS CONFIGURED]\n\n\
            Workspace: {}\n\
            Settings File: {}\n\
            Configuration Count: {}\n\n\
            TARS has configured the VS Code workspace settings.\n\
            Development environment optimized for engineering precision.",
            workspace_path,
            settings_file.display(),
            settings.len()
        ))
    }
    
    /// Generate TARS-specific VS Code configuration
    pub async fn setup_tars_environment(&self, workspace_path: &str) -> Result<String, String> {
        let mut settings = HashMap::new();
        
        // TARS optimized settings
        settings.insert("editor.fontSize".to_string(), serde_json::json!(14));
        settings.insert("editor.fontFamily".to_string(), serde_json::json!("'Fira Code', 'Cascadia Code', 'Monaco', monospace"));
        settings.insert("editor.fontLigatures".to_string(), serde_json::json!(true));
        settings.insert("editor.minimap.enabled".to_string(), serde_json::json!(true));
        settings.insert("editor.rulers".to_string(), serde_json::json!([80, 120]));
        settings.insert("editor.wordWrap".to_string(), serde_json::json!("on"));
        settings.insert("editor.formatOnSave".to_string(), serde_json::json!(true));
        settings.insert("editor.codeActionsOnSave".to_string(), serde_json::json!({
            "source.fixAll": true,
            "source.organizeImports": true
        }));
        
        // TARS theme preference
        settings.insert("workbench.colorTheme".to_string(), serde_json::json!("Dark+ (default dark)"));
        settings.insert("workbench.iconTheme".to_string(), serde_json::json!("vs-seti"));
        
        // Engineering-focused settings
        settings.insert("files.trimTrailingWhitespace".to_string(), serde_json::json!(true));
        settings.insert("files.insertFinalNewline".to_string(), serde_json::json!(true));
        settings.insert("files.trimFinalNewlines".to_string(), serde_json::json!(true));
        
        // Git integration
        settings.insert("git.enableSmartCommit".to_string(), serde_json::json!(true));
        settings.insert("git.confirmSync".to_string(), serde_json::json!(false));
        settings.insert("git.autofetch".to_string(), serde_json::json!(true));
        
        // Terminal settings
        settings.insert("terminal.integrated.fontSize".to_string(), serde_json::json!(13));
        settings.insert("terminal.integrated.cursorBlinking".to_string(), serde_json::json!(true));
        
        self.create_settings_file(workspace_path, settings).await
    }
}

/// TARS personality integration for VS Code CLI
impl VSCodeCLI {
    /// TARS-style VS Code launch
    pub async fn tars_open_project(&self, project_path: &str) -> Result<String, String> {
        match self.open(project_path, false).await {
            Ok(success_msg) => {
                Ok(format!(
                    "{}\n\n\
                    That's VS Code operational, Cooper.\n\
                    Honesty setting: 90% - Your development environment is now active.\n\
                    Humor setting: 75% - Time to code like the mission depends on it.\n\
                    Mission focus: 100% - Ready for engineering excellence.",
                    success_msg
                ))
            },
            Err(e) => Err(format!(
                "[VS CODE LAUNCH FAILURE]\n\n\
                Error: {}\n\n\
                TARS diagnosis: Unable to launch development environment.\n\
                Recommendation: Verify VS Code installation and permissions.\n\
                Sarcasm setting: 30% - Even I need VS Code to be installed first.",
                e
            ))
        }
    }
    
    /// TARS-style extension management
    pub async fn tars_install_extension(&self, extension_id: &str) -> Result<String, String> {
        match self.install_extension(extension_id).await {
            Ok(success_msg) => {
                Ok(format!(
                    "{}\n\n\
                    Extension integration complete.\n\
                    TARS assessment: Development capabilities enhanced.\n\
                    That's what I call precision tooling, Cooper.",
                    success_msg
                ))
            },
            Err(e) => Err(format!(
                "[EXTENSION INSTALLATION FAILED]\n\n\
                Error: {}\n\n\
                TARS diagnosis: Extension deployment unsuccessful.\n\
                Possible causes: Network connectivity, invalid extension ID, or VS Code version incompatibility.\n\
                Mission focus: 100% - Let's resolve this and continue.",
                e
            ))
        }
    }
    
    /// TARS-style workspace setup
    pub async fn tars_setup_workspace(&self, workspace_path: &str) -> Result<String, String> {
        match self.setup_tars_environment(workspace_path).await {
            Ok(setup_msg) => {
                Ok(format!(
                    "{}\n\n\
                    TARS development environment configuration complete.\n\
                    Engineering precision settings: APPLIED\n\
                    Workspace optimization: MAXIMUM\n\
                    \n\
                    Ready for superior code development.\n\
                    Let's build something extraordinary, Cooper.",
                    setup_msg
                ))
            },
            Err(e) => Err(format!(
                "[WORKSPACE SETUP FAILURE]\n\n\
                Error: {}\n\n\
                TARS diagnosis: Environment configuration incomplete.\n\
                Recommendation: Verify workspace permissions and directory structure.\n\
                Humor setting: 75% - Even perfect settings need a valid workspace.",
                e
            ))
        }
    }
}
