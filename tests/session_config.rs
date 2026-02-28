use std::fs;
use tempfile::tempdir;
use tmuxido::session::SessionConfig;

#[test]
fn should_load_project_session_config() {
    let dir = tempdir().unwrap();
    let config_content = r#"
        [[windows]]
        name = "editor"
        panes = ["nvim ."]
    "#;
    fs::write(dir.path().join(".tmuxido.toml"), config_content).unwrap();

    let result = SessionConfig::load_from_project(dir.path()).unwrap();
    assert!(result.is_some());
    let config = result.unwrap();
    assert_eq!(config.windows[0].name, "editor");
}

#[test]
fn should_return_none_when_no_project_config() {
    let dir = tempdir().unwrap();
    let result = SessionConfig::load_from_project(dir.path()).unwrap();
    assert!(result.is_none());
}
