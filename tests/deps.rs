use tmuxido::deps::{
    BinaryChecker, Dep, PackageManager, SystemBinaryChecker, check_missing, detect_package_manager,
};

// --- SystemBinaryChecker (real system calls) ---

#[test]
fn system_checker_finds_sh_binary() {
    let checker = SystemBinaryChecker;
    assert!(
        checker.is_available("sh"),
        "`sh` must be present on any Unix system"
    );
}

#[test]
fn system_checker_returns_false_for_nonexistent_binary() {
    let checker = SystemBinaryChecker;
    assert!(!checker.is_available("tmuxido_nonexistent_xyz_42"));
}

// --- detect_package_manager on real system ---

#[test]
fn should_detect_some_package_manager_on_linux() {
    let checker = SystemBinaryChecker;
    let pm = detect_package_manager(&checker);
    assert!(
        pm.is_some(),
        "Expected to detect at least one package manager on this Linux system"
    );
}

// --- PackageManager metadata completeness ---

#[test]
fn all_package_managers_have_non_empty_detection_binary() {
    for pm in PackageManager::all_ordered() {
        assert!(
            !pm.detection_binary().is_empty(),
            "{:?} has empty detection binary",
            pm
        );
    }
}

#[test]
fn all_package_managers_have_non_empty_display_name() {
    for pm in PackageManager::all_ordered() {
        assert!(
            !pm.display_name().is_empty(),
            "{:?} has empty display name",
            pm
        );
    }
}

#[test]
fn install_command_always_starts_with_sudo() {
    let packages = &["fzf", "tmux"];
    for pm in PackageManager::all_ordered() {
        let cmd = pm.install_command(packages);
        assert_eq!(
            cmd.first().map(String::as_str),
            Some("sudo"),
            "{} install command should start with sudo",
            pm.display_name()
        );
    }
}

#[test]
fn install_command_always_contains_requested_packages() {
    let packages = &["fzf", "tmux"];
    for pm in PackageManager::all_ordered() {
        let cmd = pm.install_command(packages);
        assert!(
            cmd.contains(&"fzf".to_string()),
            "{} command missing 'fzf'",
            pm.display_name()
        );
        assert!(
            cmd.contains(&"tmux".to_string()),
            "{} command missing 'tmux'",
            pm.display_name()
        );
    }
}

// --- Dep completeness ---

#[test]
fn dep_package_names_are_standard() {
    assert_eq!(Dep::Fzf.package_name(), "fzf");
    assert_eq!(Dep::Tmux.package_name(), "tmux");
}

#[test]
fn all_deps_have_matching_binary_and_package_names() {
    for dep in Dep::all() {
        assert!(!dep.binary_name().is_empty());
        assert!(!dep.package_name().is_empty());
    }
}

// --- check_missing on real system ---

#[test]
fn check_missing_returns_only_actually_missing_tools() {
    let checker = SystemBinaryChecker;
    let missing = check_missing(&checker);
    // Every item reported as missing must NOT be findable via `which`
    for dep in &missing {
        assert!(
            !checker.is_available(dep.binary_name()),
            "{} reported as missing but `which` finds it",
            dep.binary_name()
        );
    }
}

#[test]
fn check_missing_does_not_report_present_tools_as_missing() {
    let checker = SystemBinaryChecker;
    let missing = check_missing(&checker);
    // Every dep NOT in missing list must be available
    let missing_names: Vec<&str> = missing.iter().map(|d| d.binary_name()).collect();
    for dep in Dep::all() {
        if !missing_names.contains(&dep.binary_name()) {
            assert!(
                checker.is_available(dep.binary_name()),
                "{} not in missing list but `which` can't find it",
                dep.binary_name()
            );
        }
    }
}
