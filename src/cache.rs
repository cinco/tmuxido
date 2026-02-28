use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectCache {
    pub projects: Vec<PathBuf>,
    pub last_updated: u64,
    /// mtime de cada diretório visitado durante o scan.
    /// Usado para detectar mudanças incrementais sem precisar varrer tudo.
    #[serde(default)]
    pub dir_mtimes: HashMap<PathBuf, u64>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn mtime_secs(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Retorna o subconjunto mínimo de diretórios: aqueles que não têm nenhum
/// ancestral também na lista. Evita rescanear a mesma subárvore duas vezes.
pub(crate) fn minimal_roots(dirs: &[PathBuf]) -> Vec<PathBuf> {
    dirs.iter()
        .filter(|dir| {
            !dirs
                .iter()
                .any(|other| other != *dir && dir.starts_with(other))
        })
        .cloned()
        .collect()
}

impl ProjectCache {
    pub fn new(projects: Vec<PathBuf>, dir_mtimes: HashMap<PathBuf, u64>) -> Self {
        Self {
            projects,
            last_updated: now_secs(),
            dir_mtimes,
        }
    }

    pub fn cache_path() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .context("Could not determine cache directory")?
            .join("tmuxido");

        fs::create_dir_all(&cache_dir).with_context(|| {
            format!("Failed to create cache directory: {}", cache_dir.display())
        })?;

        Ok(cache_dir.join("projects.json"))
    }

    pub fn load() -> Result<Option<Self>> {
        let cache_path = Self::cache_path()?;

        if !cache_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&cache_path)
            .with_context(|| format!("Failed to read cache file: {}", cache_path.display()))?;

        let cache: ProjectCache = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse cache file: {}", cache_path.display()))?;

        Ok(Some(cache))
    }

    pub fn save(&self) -> Result<()> {
        let cache_path = Self::cache_path()?;

        let content = serde_json::to_string_pretty(self).context("Failed to serialize cache")?;

        fs::write(&cache_path, content)
            .with_context(|| format!("Failed to write cache file: {}", cache_path.display()))?;

        Ok(())
    }

    /// Valida e atualiza o cache de forma incremental.
    ///
    /// 1. Remove projetos cujo `.git` não existe mais.
    /// 2. Detecta diretórios com mtime alterado.
    /// 3. Resscaneia apenas as subárvores mínimas que mudaram.
    ///
    /// Retorna `true` se o cache foi modificado.
    /// Retorna `false` com `dir_mtimes` vazio (cache antigo) — chamador deve fazer rescan completo.
    #[allow(clippy::type_complexity)]
    pub fn validate_and_update(
        &mut self,
        scan_fn: &dyn Fn(&Path) -> Result<(Vec<PathBuf>, HashMap<PathBuf, u64>)>,
    ) -> Result<bool> {
        let mut changed = false;

        // Passo 1: remover projetos cujo .git não existe mais
        let before = self.projects.len();
        self.projects.retain(|p| p.join(".git").exists());
        if self.projects.len() != before {
            changed = true;
        }

        // Sem fingerprints = cache no formato antigo; sinaliza ao chamador
        if self.dir_mtimes.is_empty() {
            return Ok(changed);
        }

        // Passo 2: encontrar diretórios com mtime diferente do armazenado
        let changed_dirs: Vec<PathBuf> = self
            .dir_mtimes
            .iter()
            .filter(|(dir, stored_mtime)| {
                fs::metadata(dir)
                    .and_then(|m| m.modified())
                    .map(|t| mtime_secs(t) != **stored_mtime)
                    .unwrap_or(true) // diretório sumiu = tratar como mudança
            })
            .map(|(dir, _)| dir.clone())
            .collect();

        if changed_dirs.is_empty() {
            return Ok(changed);
        }

        // Passo 3: resscanear apenas as raízes mínimas das subárvores alteradas
        for root in minimal_roots(&changed_dirs) {
            eprintln!("Rescanning: {}", root.display());

            // Remover entradas antigas desta subárvore
            self.projects.retain(|p| !p.starts_with(&root));
            self.dir_mtimes.retain(|d, _| !d.starts_with(&root));

            // Resscanear e mesclar
            let (new_projects, new_fingerprints) = scan_fn(&root)?;
            self.projects.extend(new_projects);
            self.dir_mtimes.extend(new_fingerprints);
            changed = true;
        }

        if changed {
            self.projects.sort();
            self.projects.dedup();
            self.last_updated = now_secs();
        }

        Ok(changed)
    }

    pub fn age_in_seconds(&self) -> u64 {
        now_secs().saturating_sub(self.last_updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn should_return_empty_when_input_is_empty() {
        let result = minimal_roots(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn should_return_single_dir_as_root() {
        let dirs = vec![PathBuf::from("/home/user/projects")];
        let result = minimal_roots(&dirs);
        assert_eq!(result, dirs);
    }

    #[test]
    fn should_exclude_nested_dirs_when_parent_is_present() {
        let dirs = vec![
            PathBuf::from("/home/user"),
            PathBuf::from("/home/user/projects"),
        ];
        let result = minimal_roots(&dirs);
        assert_eq!(result.len(), 1);
        assert!(result.contains(&PathBuf::from("/home/user")));
    }

    #[test]
    fn should_keep_sibling_dirs_that_are_not_nested() {
        let dirs = vec![
            PathBuf::from("/home/user/projects"),
            PathBuf::from("/home/user/work"),
        ];
        let result = minimal_roots(&dirs);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn should_remove_stale_projects_when_git_dir_missing() {
        let dir = tempdir().unwrap();
        let project = dir.path().join("myproject");
        fs::create_dir_all(project.join(".git")).unwrap();

        let mut cache = ProjectCache::new(vec![project.clone()], HashMap::new());
        assert_eq!(cache.projects.len(), 1);

        fs::remove_dir_all(project.join(".git")).unwrap();

        let result = cache.validate_and_update(&|_| Ok((vec![], HashMap::new())));
        assert_eq!(result.unwrap(), true);
        assert!(cache.projects.is_empty());
    }

    #[test]
    fn should_return_false_when_nothing_changed() {
        let dir = tempdir().unwrap();
        let actual_mtime = fs::metadata(dir.path())
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut dir_mtimes = HashMap::new();
        dir_mtimes.insert(dir.path().to_path_buf(), actual_mtime);
        let mut cache = ProjectCache::new(vec![], dir_mtimes);

        let result = cache.validate_and_update(&|_| Ok((vec![], HashMap::new())));
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn should_rescan_dirs_when_mtime_changed() {
        let dir = tempdir().unwrap();
        let tracked = dir.path().to_path_buf();

        // Store mtime 0 — guaranteed to differ from the actual mtime
        let mut dir_mtimes = HashMap::new();
        dir_mtimes.insert(tracked, 0u64);
        let mut cache = ProjectCache::new(vec![], dir_mtimes);

        let new_project = dir.path().join("discovered");
        let scan_called = std::cell::Cell::new(false);
        let result = cache.validate_and_update(&|_root| {
            scan_called.set(true);
            Ok((vec![new_project.clone()], HashMap::new()))
        });

        assert_eq!(result.unwrap(), true);
        assert!(scan_called.get());
        assert!(cache.projects.contains(&new_project));
    }

    #[test]
    fn should_return_false_when_dir_mtimes_empty() {
        let mut cache = ProjectCache::new(vec![], HashMap::new());
        let result = cache.validate_and_update(&|_| Ok((vec![], HashMap::new())));
        assert_eq!(result.unwrap(), false);
    }
}
