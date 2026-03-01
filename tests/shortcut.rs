use std::fs;
use tempfile::tempdir;
use tmuxido::shortcut::{KeyCombo, check_kde_conflict, write_hyprland_binding, write_kde_shortcut};

#[test]
fn writes_hyprland_binding_to_new_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("bindings.conf");
    let combo = KeyCombo::parse("Super+Shift+T").unwrap();

    write_hyprland_binding(&path, &combo).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("SUPER SHIFT, T"),
        "should contain Hyprland combo"
    );
    assert!(content.contains("tmuxido"), "should mention tmuxido");
    assert!(
        content.starts_with("bindd"),
        "should start with bindd directive"
    );
}

#[test]
fn write_hyprland_binding_skips_when_tmuxido_already_present() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("bindings.conf");
    fs::write(&path, "bindd = SUPER SHIFT, T, Tmuxido, exec, tmuxido\n").unwrap();

    let combo = KeyCombo::parse("Super+Shift+T").unwrap();
    write_hyprland_binding(&path, &combo).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    let count = content.lines().filter(|l| l.contains("tmuxido")).count();
    assert_eq!(count, 1, "should not add a duplicate line");
}

#[test]
fn write_hyprland_binding_creates_parent_dirs() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("nested").join("hypr").join("bindings.conf");

    let combo = KeyCombo::parse("Super+Ctrl+T").unwrap();
    write_hyprland_binding(&path, &combo).unwrap();

    assert!(
        path.exists(),
        "file should be created even when parent dirs are missing"
    );
}

#[test]
fn writes_kde_shortcut_to_new_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("kglobalshortcutsrc");
    let combo = KeyCombo::parse("Super+Shift+T").unwrap();

    write_kde_shortcut(&path, &combo).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("[tmuxido]"),
        "should contain [tmuxido] section"
    );
    assert!(
        content.contains("Meta+Shift+T"),
        "should use Meta notation for KDE"
    );
    assert!(
        content.contains("Launch Tmuxido"),
        "should include action description"
    );
}

#[test]
fn write_kde_shortcut_skips_when_section_already_exists() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("kglobalshortcutsrc");
    fs::write(
        &path,
        "[tmuxido]\nLaunch Tmuxido=Meta+Shift+T,none,Launch Tmuxido\n",
    )
    .unwrap();

    let combo = KeyCombo::parse("Super+Shift+P").unwrap();
    write_kde_shortcut(&path, &combo).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    let count = content.matches("[tmuxido]").count();
    assert_eq!(count, 1, "should not add a duplicate section");
}

#[test]
fn check_kde_conflict_finds_existing_binding() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("kglobalshortcutsrc");
    fs::write(
        &path,
        "[myapp]\nLaunch Something=Meta+Shift+T,none,Launch Something\n",
    )
    .unwrap();

    let combo = KeyCombo::parse("Super+Shift+T").unwrap();
    let conflict = check_kde_conflict(&path, &combo);

    assert_eq!(conflict, Some("myapp".to_string()));
}

#[test]
fn check_kde_conflict_returns_none_for_free_binding() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("kglobalshortcutsrc");
    fs::write(
        &path,
        "[myapp]\nLaunch Something=Meta+Ctrl+T,none,Launch Something\n",
    )
    .unwrap();

    let combo = KeyCombo::parse("Super+Shift+T").unwrap();
    assert!(check_kde_conflict(&path, &combo).is_none());
}

#[test]
fn check_kde_conflict_returns_none_when_file_missing() {
    let combo = KeyCombo::parse("Super+Shift+T").unwrap();
    assert!(check_kde_conflict(std::path::Path::new("/nonexistent/path"), &combo).is_none());
}
