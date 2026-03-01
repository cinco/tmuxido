use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::session::SessionConfig;
use crate::ui;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub paths: Vec<String>,
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    #[serde(default = "default_cache_enabled")]
    pub cache_enabled: bool,
    #[serde(default = "default_cache_ttl_hours")]
    pub cache_ttl_hours: u64,
    #[serde(default = "default_session_config")]
    pub default_session: SessionConfig,
}

fn default_max_depth() -> usize {
    5
}

fn default_cache_enabled() -> bool {
    true
}

fn default_cache_ttl_hours() -> u64 {
    24
}

fn default_session_config() -> SessionConfig {
    use crate::session::Window;

    SessionConfig {
        windows: vec![
            Window {
                name: "editor".to_string(),
                panes: vec![],
                layout: None,
            },
            Window {
                name: "terminal".to_string(),
                panes: vec![],
                layout: None,
            },
        ],
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Self::default_config());
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

        Ok(config)
    }

    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("tmuxido");

        Ok(config_dir.join("tmuxido.toml"))
    }

    pub fn ensure_config_exists() -> Result<PathBuf> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            let config_dir = config_path
                .parent()
                .context("Could not get parent directory")?;

            fs::create_dir_all(config_dir).with_context(|| {
                format!(
                    "Failed to create config directory: {}",
                    config_dir.display()
                )
            })?;

            // Run interactive configuration wizard
            let paths = Self::prompt_for_paths()?;
            let max_depth = Self::prompt_for_max_depth()?;
            let cache_enabled = Self::prompt_for_cache_enabled()?;
            let cache_ttl_hours = if cache_enabled {
                Self::prompt_for_cache_ttl()?
            } else {
                24
            };
            let windows = Self::prompt_for_windows()?;

            // Render styled success message before moving windows
            ui::render_config_created(&paths, max_depth, cache_enabled, cache_ttl_hours, &windows);

            let config = Config {
                paths: paths.clone(),
                max_depth,
                cache_enabled,
                cache_ttl_hours,
                default_session: SessionConfig { windows },
            };

            let toml_string =
                toml::to_string_pretty(&config).context("Failed to serialize config")?;

            fs::write(&config_path, toml_string).with_context(|| {
                format!("Failed to write config file: {}", config_path.display())
            })?;
        }

        Ok(config_path)
    }

    fn prompt_for_paths() -> Result<Vec<String>> {
        // Render styled welcome banner
        ui::render_welcome_banner();

        // Get input with styled prompt
        let input = ui::render_paths_prompt()?;
        let paths = Self::parse_paths_input(&input);

        if paths.is_empty() {
            ui::render_fallback_message();
            Ok(vec![
                dirs::home_dir()
                    .unwrap_or_default()
                    .join("Projects")
                    .to_string_lossy()
                    .to_string(),
            ])
        } else {
            Ok(paths)
        }
    }

    fn prompt_for_max_depth() -> Result<usize> {
        ui::render_section_header("Scan Settings");
        let input = ui::render_max_depth_prompt()?;

        if input.is_empty() {
            return Ok(5);
        }

        match input.parse::<usize>() {
            Ok(n) if n > 0 => Ok(n),
            _ => {
                eprintln!("Invalid value, using default: 5");
                Ok(5)
            }
        }
    }

    fn prompt_for_cache_enabled() -> Result<bool> {
        ui::render_section_header("Cache Settings");
        let input = ui::render_cache_enabled_prompt()?;

        if input.is_empty() || input == "y" || input == "yes" {
            Ok(true)
        } else if input == "n" || input == "no" {
            Ok(false)
        } else {
            eprintln!("Invalid value, using default: yes");
            Ok(true)
        }
    }

    fn prompt_for_cache_ttl() -> Result<u64> {
        let input = ui::render_cache_ttl_prompt()?;

        if input.is_empty() {
            return Ok(24);
        }

        match input.parse::<u64>() {
            Ok(n) if n > 0 => Ok(n),
            _ => {
                eprintln!("Invalid value, using default: 24");
                Ok(24)
            }
        }
    }

    fn prompt_for_windows() -> Result<Vec<crate::session::Window>> {
        ui::render_section_header("Default Session");
        let input = ui::render_windows_prompt()?;

        let window_names: Vec<String> = input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let names = if window_names.is_empty() {
            vec!["editor".to_string(), "terminal".to_string()]
        } else {
            window_names
        };

        // Configure panes for each window
        let mut windows = Vec::new();
        for name in names {
            let panes = Self::prompt_for_panes(&name)?;
            windows.push(crate::session::Window {
                name,
                panes,
                layout: None,
            });
        }

        Ok(windows)
    }

    fn prompt_for_panes(window_name: &str) -> Result<Vec<String>> {
        let input = ui::render_panes_prompt(window_name)?;

        let pane_names: Vec<String> = input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if pane_names.is_empty() {
            // Single pane, no commands
            return Ok(vec![]);
        }

        // Ask for commands for each pane
        let mut panes = Vec::new();
        for pane_name in pane_names {
            let command = ui::render_pane_command_prompt(&pane_name)?;
            panes.push(command);
        }

        Ok(panes)
    }

    fn parse_paths_input(input: &str) -> Vec<String> {
        input
            .trim()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    fn default_config() -> Self {
        Config {
            paths: vec![
                dirs::home_dir()
                    .unwrap_or_default()
                    .join("Projects")
                    .to_string_lossy()
                    .to_string(),
            ],
            max_depth: 5,
            cache_enabled: true,
            cache_ttl_hours: 24,
            default_session: default_session_config(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_use_defaults_when_optional_fields_missing() {
        let toml_str = r#"paths = ["/home/user/projects"]"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.max_depth, 5);
        assert!(config.cache_enabled);
        assert_eq!(config.cache_ttl_hours, 24);
    }

    #[test]
    fn should_parse_full_config_correctly() {
        let toml_str = r#"
            paths = ["/foo", "/bar"]
            max_depth = 3
            cache_enabled = false
            cache_ttl_hours = 12
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.paths, vec!["/foo", "/bar"]);
        assert_eq!(config.max_depth, 3);
        assert!(!config.cache_enabled);
        assert_eq!(config.cache_ttl_hours, 12);
    }

    #[test]
    fn should_reject_invalid_toml() {
        let result: Result<Config, _> = toml::from_str("not valid toml ]][[");
        assert!(result.is_err());
    }

    #[test]
    fn should_parse_single_path() {
        let input = "~/Projects";
        let paths = Config::parse_paths_input(input);
        assert_eq!(paths, vec!["~/Projects"]);
    }

    #[test]
    fn should_parse_multiple_paths_with_commas() {
        let input = "~/Projects, ~/work, ~/repos";
        let paths = Config::parse_paths_input(input);
        assert_eq!(paths, vec!["~/Projects", "~/work", "~/repos"]);
    }

    #[test]
    fn should_trim_whitespace_from_paths() {
        let input = "  ~/Projects  ,  ~/work  ";
        let paths = Config::parse_paths_input(input);
        assert_eq!(paths, vec!["~/Projects", "~/work"]);
    }

    #[test]
    fn should_return_empty_vec_for_empty_input() {
        let input = "";
        let paths = Config::parse_paths_input(input);
        assert!(paths.is_empty());
    }

    #[test]
    fn should_return_empty_vec_for_whitespace_only() {
        let input = "   ";
        let paths = Config::parse_paths_input(input);
        assert!(paths.is_empty());
    }

    #[test]
    fn should_handle_empty_parts_between_commas() {
        let input = "~/Projects,,~/work";
        let paths = Config::parse_paths_input(input);
        assert_eq!(paths, vec!["~/Projects", "~/work"]);
    }
}
