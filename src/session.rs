use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Window {
    pub name: String,
    #[serde(default)]
    pub panes: Vec<String>,
    #[serde(default)]
    pub layout: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionConfig {
    #[serde(default)]
    pub windows: Vec<Window>,
}

impl SessionConfig {
    pub fn load_from_project(project_path: &Path) -> Result<Option<Self>> {
        let config_path = project_path.join(".tmuxido.toml");

        if !config_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read session config: {}", config_path.display()))?;

        let config: SessionConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse session config: {}", config_path.display()))?;

        Ok(Some(config))
    }
}

pub struct TmuxSession {
    session_name: String,
    project_path: String,
    base_index: usize,
}

impl TmuxSession {
    pub fn new(project_path: &Path) -> Self {
        let session_name = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .replace('.', "_")
            .replace(' ', "-");

        let base_index = Self::get_base_index();

        Self {
            session_name,
            project_path: project_path.display().to_string(),
            base_index,
        }
    }

    fn get_base_index() -> usize {
        // Try to get base-index from tmux
        let output = Command::new("tmux")
            .args(["show-options", "-gv", "base-index"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let index_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(index) = index_str.trim().parse::<usize>() {
                    return index;
                }
            }
        }

        // Default to 0 if we can't determine
        0
    }

    pub fn create(&self, config: &SessionConfig) -> Result<()> {
        // Check if we're already inside a tmux session
        let inside_tmux = std::env::var("TMUX").is_ok();

        // Check if session already exists
        let session_exists = Command::new("tmux")
            .args(["has-session", "-t", &self.session_name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if session_exists {
            // Session exists, just switch to it
            if inside_tmux {
                Command::new("tmux")
                    .args(["switch-client", "-t", &self.session_name])
                    .status()
                    .context("Failed to switch to existing session")?;
            } else {
                Command::new("tmux")
                    .args(["attach-session", "-t", &self.session_name])
                    .status()
                    .context("Failed to attach to existing session")?;
            }
            return Ok(());
        }

        // Create new session
        if config.windows.is_empty() {
            // Create simple session with one window
            self.create_simple_session()?;
        } else {
            // Create session with custom windows
            self.create_custom_session(config)?;
        }

        // Attach or switch to the session
        if inside_tmux {
            Command::new("tmux")
                .args(["switch-client", "-t", &self.session_name])
                .status()
                .context("Failed to switch to new session")?;
        } else {
            Command::new("tmux")
                .args(["attach-session", "-t", &self.session_name])
                .status()
                .context("Failed to attach to new session")?;
        }

        Ok(())
    }

    fn create_simple_session(&self) -> Result<()> {
        // Create a detached session with one window
        Command::new("tmux")
            .args([
                "new-session",
                "-d",
                "-s",
                &self.session_name,
                "-c",
                &self.project_path,
            ])
            .status()
            .context("Failed to create tmux session")?;

        Ok(())
    }

    fn create_custom_session(&self, config: &SessionConfig) -> Result<()> {
        // Create session with first window
        let first_window = &config.windows[0];
        Command::new("tmux")
            .args([
                "new-session",
                "-d",
                "-s",
                &self.session_name,
                "-n",
                &first_window.name,
                "-c",
                &self.project_path,
            ])
            .status()
            .context("Failed to create tmux session")?;

        // Create panes for first window if specified
        if !first_window.panes.is_empty() {
            self.create_panes(self.base_index, &first_window.panes)?;
        }

        // Apply layout for first window if specified
        if let Some(layout) = &first_window.layout {
            self.apply_layout(self.base_index, layout)?;
        }

        // Create additional windows
        for (index, window) in config.windows.iter().skip(1).enumerate() {
            let window_index = self.base_index + index + 1;

            Command::new("tmux")
                .args([
                    "new-window",
                    "-t",
                    &format!("{}:{}", self.session_name, window_index),
                    "-n",
                    &window.name,
                    "-c",
                    &self.project_path,
                ])
                .status()
                .with_context(|| format!("Failed to create window: {}", window.name))?;

            // Create panes if specified
            if !window.panes.is_empty() {
                self.create_panes(window_index, &window.panes)?;
            }

            // Apply layout if specified
            if let Some(layout) = &window.layout {
                self.apply_layout(window_index, layout)?;
            }
        }

        // Select the first window
        Command::new("tmux")
            .args(["select-window", "-t", &format!("{}:{}", self.session_name, self.base_index)])
            .status()
            .context("Failed to select first window")?;

        Ok(())
    }

    fn create_panes(&self, window_index: usize, panes: &[String]) -> Result<()> {
        for (pane_index, command) in panes.iter().enumerate() {
            let target = format!("{}:{}", self.session_name, window_index);

            // First pane already exists (created with the window), skip split
            if pane_index > 0 {
                // Create new pane by splitting
                Command::new("tmux")
                    .args([
                        "split-window",
                        "-t",
                        &target,
                        "-c",
                        &self.project_path,
                    ])
                    .status()
                    .context("Failed to split pane")?;
            }

            // Send the command to the pane if it's not empty
            if !command.is_empty() {
                let pane_target = format!("{}:{}.{}", self.session_name, window_index, pane_index);
                Command::new("tmux")
                    .args([
                        "send-keys",
                        "-t",
                        &pane_target,
                        command,
                        "Enter",
                    ])
                    .status()
                    .context("Failed to send keys to pane")?;
            }
        }

        Ok(())
    }

    fn apply_layout(&self, window_index: usize, layout: &str) -> Result<()> {
        Command::new("tmux")
            .args([
                "select-layout",
                "-t",
                &format!("{}:{}", self.session_name, window_index),
                layout,
            ])
            .status()
            .with_context(|| format!("Failed to apply layout: {}", layout))?;

        Ok(())
    }
}
