pub mod cache;
pub mod config;
pub mod session;

use anyhow::Result;
use cache::ProjectCache;
use config::Config;
use session::{SessionConfig, TmuxSession};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use walkdir::WalkDir;

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
    if !config.cache_enabled || force_refresh {
        let (projects, fingerprints) = scan_all_roots(config)?;
        let cache = ProjectCache::new(projects.clone(), fingerprints);
        cache.save()?;
        eprintln!("Cache updated with {} projects", projects.len());
        return Ok(projects);
    }

    if let Some(mut cache) = ProjectCache::load()? {
        // Cache no formato antigo (sem dir_mtimes) → atualizar com rescan completo
        if cache.dir_mtimes.is_empty() {
            eprintln!("Upgrading cache, scanning for projects...");
            let (projects, fingerprints) = scan_all_roots(config)?;
            let new_cache = ProjectCache::new(projects.clone(), fingerprints);
            new_cache.save()?;
            eprintln!("Cache updated with {} projects", projects.len());
            return Ok(projects);
        }

        let changed = cache.validate_and_update(&|root| scan_from_root(root, config))?;
        if changed {
            cache.save()?;
            eprintln!(
                "Cache updated incrementally ({} projects)",
                cache.projects.len()
            );
        } else {
            eprintln!("Using cached projects ({} projects)", cache.projects.len());
        }
        return Ok(cache.projects);
    }

    // Sem cache ainda — scan completo inicial
    eprintln!("No cache found, scanning for projects...");
    let (projects, fingerprints) = scan_all_roots(config)?;
    let cache = ProjectCache::new(projects.clone(), fingerprints);
    cache.save()?;
    eprintln!("Cache updated with {} projects", projects.len());
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
