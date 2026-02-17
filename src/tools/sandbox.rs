use async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use crate::config::WorkingDirIsolation;
use crate::llm_types::ToolDefinition;
use crate::tools::{resolve_tool_working_dir, schema_object, Tool, ToolResult};

pub struct SandboxTool {
    working_dir: PathBuf,
    working_dir_isolation: WorkingDirIsolation,
    enabled: bool,
}

impl SandboxTool {
    pub fn new(working_dir: &str, working_dir_isolation: WorkingDirIsolation) -> Self {
        Self::with_config(working_dir, working_dir_isolation, true)
    }

    pub fn new_with_config(
        working_dir: &str,
        working_dir_isolation: WorkingDirIsolation,
        enabled: bool,
    ) -> Self {
        Self::with_config(working_dir, working_dir_isolation, enabled)
    }

    pub fn with_config(
        working_dir: &str,
        working_dir_isolation: WorkingDirIsolation,
        enabled: bool,
    ) -> Self {
        Self {
            working_dir: PathBuf::from(working_dir),
            working_dir_isolation,
            enabled,
        }
    }

    async fn check_msb_available(&self) -> bool {
        Command::new("msb")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| true)
            .unwrap_or(false)
    }

    async fn execute_in_sandbox(
        &self,
        code: &str,
        timeout_secs: u64,
        working_dir: &PathBuf,
    ) -> ToolResult {
        // Check if msb is available
        if !self.check_msb_available().await {
            return ToolResult::error(
                "Microsandbox (msb) is not installed. Please install it with: curl -sSL https://get.microsandbox.dev | sh".into(),
            );
        }

        // Execute using msb CLI with stdin
        // msb exe python reads code from stdin
        let result = timeout(
            Duration::from_secs(timeout_secs),
            async {
                let mut child = Command::new("msb")
                    .args(["exe", "python"])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .stdin(Stdio::piped())
                    .current_dir(working_dir)
                    .spawn()
                    .expect("Failed to spawn msb");

                // Write code to stdin
                if let Some(ref mut stdin) = child.stdin {
                    stdin.write_all(code.as_bytes()).await.ok();
                }

                let output = child.wait_with_output().await;

                match output {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                        // Build result similar to bash tool
                        let mut result_text = String::new();
                        if !stdout.is_empty() {
                            result_text.push_str(&stdout);
                        }
                        if !stderr.is_empty() {
                            if !result_text.is_empty() {
                                result_text.push('\n');
                            }
                            result_text.push_str(&stderr);
                        }

                        ToolResult::success(result_text)
                    }
                    Err(e) => {
                        ToolResult::error(format!("Failed to execute sandbox: {}", e))
                    }
                }
            }
        ).await;

        match result {
            Ok(tool_result) => tool_result,
            Err(_) => ToolResult::error(format!(
                "Sandbox execution timed out after {} seconds",
                timeout_secs
            )),
        }
    }
}

#[async_trait]
impl Tool for SandboxTool {
    fn name(&self) -> &str {
        "sandbox_run"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "sandbox_run".into(),
            description: "Execute Python code in an isolated sandbox environment using Microsandbox. Provides hardware-level isolation for running untrusted code.".into(),
            input_schema: schema_object(
                json!({
                    "code": {
                        "type": "string",
                        "description": "The Python code to execute in the sandbox"
                    },
                    "timeout": {
                        "type": "integer",
                        "description": "Timeout in seconds (default: 30, max: 300)"
                    }
                }),
                &["code"],
            ),
        }
    }

    async fn execute(&self, input: serde_json::Value) -> ToolResult {
        if !self.enabled {
            return ToolResult::error("Sandbox is disabled in configuration".into());
        }

        let code = match input.get("code").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return ToolResult::error("Missing 'code' parameter".into()),
        };

        let timeout_secs = input
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(30)
            .min(300); // Max 5 minutes

        let working_dir = resolve_tool_working_dir(
            &self.working_dir,
            self.working_dir_isolation,
            &input,
        );

        if let Err(e) = tokio::fs::create_dir_all(&working_dir).await {
            return ToolResult::error(format!(
                "Failed to create working directory {}: {}",
                working_dir.display(),
                e
            ));
        }

        self.execute_in_sandbox(code, timeout_secs, &working_dir).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_tool_name() {
        let tool = SandboxTool::new("/tmp", WorkingDirIsolation::Chat);
        assert_eq!(tool.name(), "sandbox_run");
    }

    #[test]
    fn test_sandbox_tool_definition() {
        let tool = SandboxTool::new("/tmp", WorkingDirIsolation::Chat);
        let def = tool.definition();
        assert_eq!(def.name, "sandbox_run");
        assert!(!def.description.is_empty());
    }

    #[test]
    fn test_sandbox_disabled() {
        let tool = SandboxTool::with_config(
            "/tmp",
            WorkingDirIsolation::Chat,
            false,
        );
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            tool.execute(json!({"code": "print(1)"})).await
        });
        assert!(result.is_error);
    }
}
