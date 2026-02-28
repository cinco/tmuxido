use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::session::SessionConfig;

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

            let default_config = Self::default_config();
            let toml_string = toml::to_string_pretty(&default_config)
                .context("Failed to serialize default config")?;

            fs::write(&config_path, toml_string).with_context(|| {
                format!("Failed to write config file: {}", config_path.display())
            })?;

            eprintln!("Created default config at: {}", config_path.display());
        }

        Ok(config_path)
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
}
