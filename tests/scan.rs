use std::fs;
use tempfile::tempdir;
use tmuxido::config::Config;
use tmuxido::scan_from_root;
use tmuxido::session::SessionConfig;

fn make_config(max_depth: usize) -> Config {
    Config {
        paths: vec![],
        max_depth,
        cache_enabled: true,
        cache_ttl_hours: 24,
        update_check_interval_hours: 24,
        default_session: SessionConfig { windows: vec![] },
    }
}

/// `tempfile::tempdir()` creates hidden dirs (e.g. `/tmp/.tmpXXXXXX`) on this
/// system, which `scan_from_root`'s `filter_entry` would skip. Create a
/// visible subdirectory to use as the actual scan root.
fn make_scan_root() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempdir().unwrap();
    let root = dir.path().join("scan_root");
    fs::create_dir_all(&root).unwrap();
    (dir, root)
}

#[test]
fn should_find_git_repos_in_temp_dir() {
    let (_dir, root) = make_scan_root();
    fs::create_dir_all(root.join("foo/.git")).unwrap();
    fs::create_dir_all(root.join("bar/.git")).unwrap();

    let config = make_config(5);
    let (projects, _) = scan_from_root(&root, &config).unwrap();

    assert_eq!(projects.len(), 2);
    assert!(projects.iter().any(|p| p.ends_with("foo")));
    assert!(projects.iter().any(|p| p.ends_with("bar")));
}

#[test]
fn should_not_descend_into_hidden_dirs() {
    let (_dir, root) = make_scan_root();
    fs::create_dir_all(root.join(".hidden/repo/.git")).unwrap();

    let config = make_config(5);
    let (projects, _) = scan_from_root(&root, &config).unwrap();

    assert!(projects.is_empty());
}

#[test]
fn should_respect_max_depth() {
    let (_dir, root) = make_scan_root();
    // Shallow: project/.git at depth 2 from root — found with max_depth=2
    fs::create_dir_all(root.join("project/.git")).unwrap();
    // Deep: nested/deep/project/.git at depth 4 — excluded with max_depth=2
    fs::create_dir_all(root.join("nested/deep/project/.git")).unwrap();

    let config = make_config(2);
    let (projects, _) = scan_from_root(&root, &config).unwrap();

    assert_eq!(projects.len(), 1);
    assert!(projects[0].ends_with("project"));
}
