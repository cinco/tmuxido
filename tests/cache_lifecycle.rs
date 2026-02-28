use std::collections::HashMap;
use std::path::PathBuf;
use tmuxido::cache::ProjectCache;

#[test]
fn should_save_and_reload_cache() {
    let projects = vec![PathBuf::from("/tmp/test_tmuxido_project")];
    let cache = ProjectCache::new(projects.clone(), HashMap::new());
    cache.save().unwrap();

    let loaded = ProjectCache::load().unwrap().unwrap();
    assert_eq!(loaded.projects, projects);
}
