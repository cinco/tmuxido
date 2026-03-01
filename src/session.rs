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

        let config: SessionConfig = toml::from_str(&content).with_context(|| {
            format!("Failed to parse session config: {}", config_path.display())
        })?;

        Ok(Some(config))
    }
}

pub struct TmuxSession {
    pub(crate) session_name: String,
    project_path: String,
}

impl TmuxSession {
    pub fn new(project_path: &Path) -> Self {
        let session_name = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .replace('.', "_")
            .replace(' ', "-");

        Self {
            session_name,
            project_path: project_path.display().to_string(),
        }
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

        let first_target = format!("{}:{}", self.session_name, first_window.name);

        if !first_window.panes.is_empty() {
            self.create_panes(&first_target, &first_window.panes)?;
        }

        if let Some(layout) = &first_window.layout {
            self.apply_layout(&first_target, layout)?;
        }

        // Create additional windows, targeting by session name so tmux auto-assigns the index
        for window in config.windows.iter().skip(1) {
            Command::new("tmux")
                .args([
                    "new-window",
                    "-t",
                    &self.session_name,
                    "-n",
                    &window.name,
                    "-c",
                    &self.project_path,
                ])
                .status()
                .with_context(|| format!("Failed to create window: {}", window.name))?;

            let target = format!("{}:{}", self.session_name, window.name);

            if !window.panes.is_empty() {
                self.create_panes(&target, &window.panes)?;
            }

            if let Some(layout) = &window.layout {
                self.apply_layout(&target, layout)?;
            }
        }

        // Select the first window by name
        Command::new("tmux")
            .args(["select-window", "-t", &first_target])
            .status()
            .context("Failed to select first window")?;

        Ok(())
    }

    fn create_panes(&self, window_target: &str, panes: &[String]) -> Result<()> {
        for (pane_index, command) in panes.iter().enumerate() {
            // First pane already exists (created with the window), skip split
            if pane_index > 0 {
                Command::new("tmux")
                    .args([
                        "split-window",
                        "-t",
                        window_target,
                        "-c",
                        &self.project_path,
                    ])
                    .status()
                    .context("Failed to split pane")?;
            }

            if !command.is_empty() {
                let pane_target = format!("{}.{}", window_target, pane_index);
                Command::new("tmux")
                    .args(["send-keys", "-t", &pane_target, command, "Enter"])
                    .status()
                    .context("Failed to send keys to pane")?;
            }
        }

        Ok(())
    }

    fn apply_layout(&self, window_target: &str, layout: &str) -> Result<()> {
        Command::new("tmux")
            .args(["select-layout", "-t", window_target, layout])
            .status()
            .with_context(|| format!("Failed to apply layout: {}", layout))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn should_replace_dots_with_underscores_in_session_name() {
        let session = TmuxSession::new(Path::new("/home/user/my.project"));
        assert_eq!(session.session_name, "my_project");
    }

    #[test]
    fn should_replace_spaces_with_dashes_in_session_name() {
        let session = TmuxSession::new(Path::new("/home/user/my project"));
        assert_eq!(session.session_name, "my-project");
    }

    #[test]
    fn should_use_project_fallback_when_path_has_no_filename() {
        let session = TmuxSession::new(Path::new("/"));
        assert_eq!(session.session_name, "project");
    }

    #[test]
    fn should_parse_window_from_toml() {
        let toml_str = r#"
            [[windows]]
            name = "editor"
            panes = ["nvim ."]
        "#;
        let config: SessionConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.windows[0].name, "editor");
        assert_eq!(config.windows[0].panes, vec!["nvim ."]);
    }

    #[test]
    fn should_parse_session_config_with_layout() {
        let toml_str = r#"
            [[windows]]
            name = "main"
            layout = "tiled"
            panes = ["vim", "bash"]
        "#;
        let config: SessionConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.windows[0].layout, Some("tiled".to_string()));
        assert_eq!(config.windows[0].panes.len(), 2);
    }
}
