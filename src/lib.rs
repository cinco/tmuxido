pub mod cache;
pub mod config;
pub mod deps;
pub mod self_update;
pub mod session;
pub mod shortcut;
pub mod ui;
pub mod update_check;

use anyhow::Result;
use cache::ProjectCache;
use config::Config;
use session::{SessionConfig, TmuxSession};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use walkdir::WalkDir;

pub fn setup_shortcut_wizard() -> Result<()> {
    shortcut::setup_shortcut_wizard()
}

pub fn setup_desktop_integration_wizard() -> Result<()> {
    shortcut::setup_desktop_integration_wizard()
}

pub fn show_cache_status(config: &Config) -> Result<()> {
    if !config.cache_enabled {
        println!("Cache is disabled in configuration");
        return Ok(());
    }

    if let Some(cache) = ProjectCache::load()? {
        let age_seconds = cache.age_in_seconds();
        let age_hours = age_seconds / 3600;
        let age_minutes = (age_seconds % 3600) / 60;

        println!("Cache status:");
        println!("  Location: {}", ProjectCache::cache_path()?.display());
        println!("  Projects cached: {}", cache.projects.len());
        println!("  Directories tracked: {}", cache.dir_mtimes.len());
        println!("  Last updated: {}h {}m ago", age_hours, age_minutes);
    } else {
        println!("No cache found");
        println!("  Run without --cache-status to create it");
    }

    Ok(())
}

pub fn get_projects(config: &Config, force_refresh: bool) -> Result<Vec<PathBuf>> {
    get_projects_internal(
        config,
        force_refresh,
        &ProjectCache::load,
        &|cache| cache.save(),
        &scan_all_roots,
        &spawn_background_refresh,
    )
}

/// Rebuilds the project cache incrementally. Intended to be called from a
/// background process spawned by `get_projects` via stale-while-revalidate.
pub fn refresh_cache(config: &Config) -> Result<()> {
    match ProjectCache::load()? {
        None => {
            let (projects, fingerprints) = scan_all_roots(config)?;
            ProjectCache::new(projects, fingerprints).save()?;
        }
        Some(mut cache) => {
            if cache.dir_mtimes.is_empty() {
                // Old cache format — full rescan
                let (projects, fingerprints) = scan_all_roots(config)?;
                ProjectCache::new(projects, fingerprints).save()?;
            } else {
                // Incremental rescan based on directory mtimes
                let changed = cache.validate_and_update(&|root| scan_from_root(root, config))?;
                if changed {
                    cache.save()?;
                }
            }
        }
    }
    Ok(())
}

fn spawn_background_refresh() {
    if let Ok(exe) = std::env::current_exe() {
        std::process::Command::new(exe)
            .arg("--background-refresh")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .ok();
    }
}

#[allow(clippy::type_complexity)]
fn get_projects_internal(
    config: &Config,
    force_refresh: bool,
    cache_loader: &dyn Fn() -> Result<Option<ProjectCache>>,
    cache_saver: &dyn Fn(&ProjectCache) -> Result<()>,
    scanner: &dyn Fn(&Config) -> Result<(Vec<PathBuf>, HashMap<PathBuf, u64>)>,
    refresh_spawner: &dyn Fn(),
) -> Result<Vec<PathBuf>> {
    if !config.cache_enabled || force_refresh {
        let (projects, fingerprints) = scanner(config)?;
        let cache = ProjectCache::new(projects.clone(), fingerprints);
        cache_saver(&cache)?;
        return Ok(projects);
    }

    if let Some(cache) = cache_loader()? {
        // Cache exists — return immediately (stale-while-revalidate).
        // Spawn a background refresh if the cache is stale or in old format.
        let is_stale =
            cache.dir_mtimes.is_empty() || cache.age_in_seconds() > config.cache_ttl_hours * 3600;
        if is_stale {
            refresh_spawner();
        }
        return Ok(cache.projects);
    }

    // No cache yet — first run, blocking scan is unavoidable.
    eprintln!("No cache found, scanning for projects...");
    let (projects, fingerprints) = scanner(config)?;
    let cache = ProjectCache::new(projects.clone(), fingerprints);
    cache_saver(&cache)?;
    Ok(projects)
}

pub fn scan_all_roots(config: &Config) -> Result<(Vec<PathBuf>, HashMap<PathBuf, u64>)> {
    let mut all_projects = Vec::new();
    let mut all_fingerprints = HashMap::new();

    for path_str in &config.paths {
        let path = PathBuf::from(shellexpand::tilde(path_str).to_string());

        if !path.exists() {
            eprintln!("Warning: Path does not exist: {}", path.display());
            continue;
        }

        eprintln!("Scanning: {}", path.display());

        let (projects, fingerprints) = scan_from_root(&path, config)?;
        all_projects.extend(projects);
        all_fingerprints.extend(fingerprints);
    }

    all_projects.sort();
    all_projects.dedup();

    Ok((all_projects, all_fingerprints))
}

pub fn scan_from_root(
    root: &Path,
    config: &Config,
) -> Result<(Vec<PathBuf>, HashMap<PathBuf, u64>)> {
    let mut projects = Vec::new();
    let mut fingerprints = HashMap::new();

    for entry in WalkDir::new(root)
        .max_depth(config.max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            e.file_name()
                .to_str()
                .map(|s| !s.starts_with('.') || s == ".git")
                .unwrap_or(false)
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if entry.file_type().is_dir() {
            if entry.file_name() == ".git" {
                // Projeto encontrado
                if let Some(parent) = entry.path().parent() {
                    projects.push(parent.to_path_buf());
                }
            } else {
                // Registrar mtime para detecção de mudanças futuras
                if let Ok(metadata) = entry.metadata()
                    && let Ok(modified) = metadata.modified()
                {
                    let mtime = modified
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    fingerprints.insert(entry.path().to_path_buf(), mtime);
                }
            }
        }
    }

    Ok((projects, fingerprints))
}

pub fn launch_tmux_session(selected: &Path, config: &Config) -> Result<()> {
    // Try to load project-specific config, fallback to global default
    let session_config = SessionConfig::load_from_project(selected)?
        .unwrap_or_else(|| config.default_session.clone());

    // Create tmux session
    let tmux_session = TmuxSession::new(selected);
    tmux_session.create(&session_config)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn make_config(cache_enabled: bool, cache_ttl_hours: u64) -> Config {
        Config {
            paths: vec!["/tmp/test".to_string()],
            max_depth: 3,
            cache_enabled,
            cache_ttl_hours,
            update_check_interval_hours: 24,
            default_session: session::SessionConfig { windows: vec![] },
        }
    }

    fn fresh_cache(projects: Vec<PathBuf>) -> ProjectCache {
        let fingerprints = HashMap::from([(PathBuf::from("/tracked"), 1u64)]);
        ProjectCache::new(projects, fingerprints)
        // last_updated = now_secs() — within any reasonable TTL
    }

    fn stale_cache(projects: Vec<PathBuf>) -> ProjectCache {
        let fingerprints = HashMap::from([(PathBuf::from("/tracked"), 1u64)]);
        let mut c = ProjectCache::new(projects, fingerprints);
        c.last_updated = 0; // epoch — always older than TTL
        c
    }

    fn call_internal(
        config: &Config,
        force_refresh: bool,
        cache_loader: &dyn Fn() -> Result<Option<ProjectCache>>,
        cache_saver: &dyn Fn(&ProjectCache) -> Result<()>,
        scanner: &dyn Fn(&Config) -> Result<(Vec<PathBuf>, HashMap<PathBuf, u64>)>,
        refresh_spawner: &dyn Fn(),
    ) -> Result<Vec<PathBuf>> {
        get_projects_internal(
            config,
            force_refresh,
            cache_loader,
            cache_saver,
            scanner,
            refresh_spawner,
        )
    }

    #[test]
    fn should_scan_when_cache_disabled() {
        let config = make_config(false, 24);
        let expected = vec![PathBuf::from("/p1")];
        let scanner_called = RefCell::new(false);
        let saver_called = RefCell::new(false);
        let spawner_called = RefCell::new(false);

        let result = call_internal(
            &config,
            false,
            &|| panic!("loader must not be called when cache disabled"),
            &|_| {
                *saver_called.borrow_mut() = true;
                Ok(())
            },
            &|_| {
                *scanner_called.borrow_mut() = true;
                Ok((expected.clone(), HashMap::new()))
            },
            &|| *spawner_called.borrow_mut() = true,
        );

        assert!(result.is_ok());
        assert!(scanner_called.into_inner());
        assert!(saver_called.into_inner());
        assert!(!spawner_called.into_inner());
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn should_scan_when_force_refresh() {
        let config = make_config(true, 24);
        let expected = vec![PathBuf::from("/p1")];
        let scanner_called = RefCell::new(false);
        let saver_called = RefCell::new(false);
        let spawner_called = RefCell::new(false);

        let result = call_internal(
            &config,
            true,
            &|| panic!("loader must not be called on force refresh"),
            &|_| {
                *saver_called.borrow_mut() = true;
                Ok(())
            },
            &|_| {
                *scanner_called.borrow_mut() = true;
                Ok((expected.clone(), HashMap::new()))
            },
            &|| *spawner_called.borrow_mut() = true,
        );

        assert!(result.is_ok());
        assert!(scanner_called.into_inner());
        assert!(saver_called.into_inner());
        assert!(!spawner_called.into_inner());
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn should_do_blocking_scan_when_no_cache_exists() {
        let config = make_config(true, 24);
        let expected = vec![PathBuf::from("/p1")];
        let scanner_called = RefCell::new(false);
        let saver_called = RefCell::new(false);
        let spawner_called = RefCell::new(false);

        let result = call_internal(
            &config,
            false,
            &|| Ok(None),
            &|_| {
                *saver_called.borrow_mut() = true;
                Ok(())
            },
            &|_| {
                *scanner_called.borrow_mut() = true;
                Ok((expected.clone(), HashMap::new()))
            },
            &|| *spawner_called.borrow_mut() = true,
        );

        assert!(result.is_ok());
        assert!(scanner_called.into_inner());
        assert!(saver_called.into_inner());
        assert!(!spawner_called.into_inner());
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn should_return_cached_projects_immediately_when_cache_is_fresh() {
        let config = make_config(true, 24);
        let cached = vec![PathBuf::from("/cached/project")];
        let cache = RefCell::new(Some(fresh_cache(cached.clone())));
        let spawner_called = RefCell::new(false);

        let result = call_internal(
            &config,
            false,
            &|| Ok(cache.borrow_mut().take()),
            &|_| panic!("saver must not be called in foreground"),
            &|_| panic!("scanner must not be called when cache is fresh"),
            &|| *spawner_called.borrow_mut() = true,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), cached);
        assert!(
            !spawner_called.into_inner(),
            "fresh cache should not trigger background refresh"
        );
    }

    #[test]
    fn should_return_stale_cache_immediately_and_spawn_background_refresh() {
        let config = make_config(true, 24);
        let cached = vec![PathBuf::from("/cached/project")];
        let cache = RefCell::new(Some(stale_cache(cached.clone())));
        let spawner_called = RefCell::new(false);

        let result = call_internal(
            &config,
            false,
            &|| Ok(cache.borrow_mut().take()),
            &|_| panic!("saver must not be called in foreground"),
            &|_| panic!("scanner must not be called in foreground"),
            &|| *spawner_called.borrow_mut() = true,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), cached);
        assert!(
            spawner_called.into_inner(),
            "stale cache must trigger background refresh"
        );
    }

    #[test]
    fn should_spawn_background_refresh_when_cache_has_no_fingerprints() {
        let config = make_config(true, 24);
        let cached = vec![PathBuf::from("/old/project")];
        // Old cache format: no dir_mtimes
        let old_cache = RefCell::new(Some(ProjectCache::new(cached.clone(), HashMap::new())));
        let spawner_called = RefCell::new(false);

        let result = call_internal(
            &config,
            false,
            &|| Ok(old_cache.borrow_mut().take()),
            &|_| panic!("saver must not be called in foreground"),
            &|_| panic!("scanner must not be called in foreground"),
            &|| *spawner_called.borrow_mut() = true,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), cached);
        assert!(
            spawner_called.into_inner(),
            "old cache format must trigger background refresh"
        );
    }
}
