use anyhow::{Context, Result};
use std::io::{self, Write};
use std::process::{Command, Stdio};

/// Required external tool dependencies.
#[derive(Debug, Clone, PartialEq)]
pub enum Dep {
    Fzf,
    Tmux,
}

/// Supported Linux package managers.
#[derive(Debug, Clone, PartialEq)]
pub enum PackageManager {
    Apt,
    Pacman,
    Dnf,
    Yum,
    Zypper,
    Emerge,
    Xbps,
    Apk,
}

/// Injectable binary availability checker — enables unit testing without hitting the real system.
pub trait BinaryChecker {
    fn is_available(&self, name: &str) -> bool;
}

/// Production implementation: delegates to the system `which` command.
pub struct SystemBinaryChecker;

impl BinaryChecker for SystemBinaryChecker {
    fn is_available(&self, name: &str) -> bool {
        Command::new("which")
            .arg(name)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

impl Dep {
    pub fn all() -> Vec<Self> {
        vec![Self::Fzf, Self::Tmux]
    }

    pub fn binary_name(&self) -> &str {
        match self {
            Self::Fzf => "fzf",
            Self::Tmux => "tmux",
        }
    }

    pub fn package_name(&self) -> &str {
        match self {
            Self::Fzf => "fzf",
            Self::Tmux => "tmux",
        }
    }
}

impl PackageManager {
    /// Ordered list for detection — more specific managers first.
    pub fn all_ordered() -> Vec<Self> {
        vec![
            Self::Apt,
            Self::Pacman,
            Self::Dnf,
            Self::Yum,
            Self::Zypper,
            Self::Emerge,
            Self::Xbps,
            Self::Apk,
        ]
    }

    /// Binary used to detect whether this package manager is installed.
    pub fn detection_binary(&self) -> &str {
        match self {
            Self::Apt => "apt",
            Self::Pacman => "pacman",
            Self::Dnf => "dnf",
            Self::Yum => "yum",
            Self::Zypper => "zypper",
            Self::Emerge => "emerge",
            Self::Xbps => "xbps-install",
            Self::Apk => "apk",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::Apt => "apt (Debian/Ubuntu)",
            Self::Pacman => "pacman (Arch Linux)",
            Self::Dnf => "dnf (Fedora)",
            Self::Yum => "yum (RHEL/CentOS)",
            Self::Zypper => "zypper (openSUSE)",
            Self::Emerge => "emerge (Gentoo)",
            Self::Xbps => "xbps-install (Void Linux)",
            Self::Apk => "apk (Alpine Linux)",
        }
    }

    /// Builds the full install command (including `sudo`) for the given packages.
    pub fn install_command(&self, packages: &[&str]) -> Vec<String> {
        let mut cmd = vec!["sudo".to_string()];
        match self {
            Self::Apt => cmd.extend(["apt", "install", "-y"].map(String::from)),
            Self::Pacman => cmd.extend(["pacman", "-S", "--noconfirm"].map(String::from)),
            Self::Dnf => cmd.extend(["dnf", "install", "-y"].map(String::from)),
            Self::Yum => cmd.extend(["yum", "install", "-y"].map(String::from)),
            Self::Zypper => cmd.extend(["zypper", "install", "-y"].map(String::from)),
            Self::Emerge => cmd.extend(["emerge"].map(String::from)),
            Self::Xbps => cmd.extend(["xbps-install", "-y"].map(String::from)),
            Self::Apk => cmd.extend(["apk", "add"].map(String::from)),
        }
        cmd.extend(packages.iter().map(|&s| s.to_string()));
        cmd
    }
}

/// Returns the required deps that are not currently installed.
pub fn check_missing<C: BinaryChecker>(checker: &C) -> Vec<Dep> {
    Dep::all()
        .into_iter()
        .filter(|dep| !checker.is_available(dep.binary_name()))
        .collect()
}

/// Returns the first supported package manager found on the system.
pub fn detect_package_manager<C: BinaryChecker>(checker: &C) -> Option<PackageManager> {
    PackageManager::all_ordered()
        .into_iter()
        .find(|pm| checker.is_available(pm.detection_binary()))
}

/// Checks for missing dependencies, informs the user, and offers to install them.
///
/// Returns `Ok(())` if all deps are available (or successfully installed).
pub fn ensure_dependencies() -> Result<()> {
    let checker = SystemBinaryChecker;
    let missing = check_missing(&checker);

    if missing.is_empty() {
        return Ok(());
    }

    eprintln!("The following required tools are not installed:");
    for dep in &missing {
        eprintln!("  ✗ {}", dep.binary_name());
    }
    eprintln!();

    let pm = detect_package_manager(&checker).ok_or_else(|| {
        anyhow::anyhow!(
            "No supported package manager found. Please install {} manually.",
            missing
                .iter()
                .map(|d| d.binary_name())
                .collect::<Vec<_>>()
                .join(" and ")
        )
    })?;

    let packages: Vec<&str> = missing.iter().map(|d| d.package_name()).collect();
    let cmd = pm.install_command(&packages);

    eprintln!("Detected package manager: {}", pm.display_name());
    eprintln!("Install command: {}", cmd.join(" "));
    eprint!("\nProceed with installation? [Y/n] ");
    io::stdout().flush().ok();

    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .context("Failed to read user input")?;

    let answer = answer.trim().to_lowercase();
    if answer == "n" || answer == "no" {
        anyhow::bail!(
            "Installation cancelled. Please install {} manually before running tmuxido.",
            missing
                .iter()
                .map(|d| d.binary_name())
                .collect::<Vec<_>>()
                .join(" and ")
        );
    }

    let (program, args) = cmd
        .split_first()
        .expect("install_command always returns at least one element");

    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("Failed to run: {}", cmd.join(" ")))?;

    if !status.success() {
        anyhow::bail!(
            "Installation failed. Please install {} manually.",
            missing
                .iter()
                .map(|d| d.binary_name())
                .collect::<Vec<_>>()
                .join(" and ")
        );
    }

    eprintln!("Installation complete!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockChecker {
        available: Vec<String>,
    }

    impl MockChecker {
        fn with(available: &[&str]) -> Self {
            Self {
                available: available.iter().map(|s| s.to_string()).collect(),
            }
        }
    }

    impl BinaryChecker for MockChecker {
        fn is_available(&self, name: &str) -> bool {
            self.available.iter().any(|s| s == name)
        }
    }

    // --- Dep ---

    #[test]
    fn should_return_fzf_binary_name() {
        assert_eq!(Dep::Fzf.binary_name(), "fzf");
    }

    #[test]
    fn should_return_tmux_binary_name() {
        assert_eq!(Dep::Tmux.binary_name(), "tmux");
    }

    #[test]
    fn should_include_fzf_and_tmux_in_all_deps() {
        let deps = Dep::all();
        assert!(deps.contains(&Dep::Fzf));
        assert!(deps.contains(&Dep::Tmux));
    }

    #[test]
    fn should_return_same_package_name_as_binary_for_fzf() {
        assert_eq!(Dep::Fzf.package_name(), "fzf");
    }

    #[test]
    fn should_return_same_package_name_as_binary_for_tmux() {
        assert_eq!(Dep::Tmux.package_name(), "tmux");
    }

    // --- check_missing ---

    #[test]
    fn should_return_empty_when_all_deps_present() {
        let checker = MockChecker::with(&["fzf", "tmux"]);
        assert!(check_missing(&checker).is_empty());
    }

    #[test]
    fn should_detect_fzf_as_missing_when_only_tmux_present() {
        let checker = MockChecker::with(&["tmux"]);
        let missing = check_missing(&checker);
        assert_eq!(missing, vec![Dep::Fzf]);
    }

    #[test]
    fn should_detect_tmux_as_missing_when_only_fzf_present() {
        let checker = MockChecker::with(&["fzf"]);
        let missing = check_missing(&checker);
        assert_eq!(missing, vec![Dep::Tmux]);
    }

    #[test]
    fn should_detect_both_missing_when_none_present() {
        let checker = MockChecker::with(&[]);
        let missing = check_missing(&checker);
        assert_eq!(missing.len(), 2);
        assert!(missing.contains(&Dep::Fzf));
        assert!(missing.contains(&Dep::Tmux));
    }

    // --- detect_package_manager ---

    #[test]
    fn should_detect_apt_when_available() {
        let checker = MockChecker::with(&["apt"]);
        assert_eq!(detect_package_manager(&checker), Some(PackageManager::Apt));
    }

    #[test]
    fn should_detect_pacman_when_available() {
        let checker = MockChecker::with(&["pacman"]);
        assert_eq!(
            detect_package_manager(&checker),
            Some(PackageManager::Pacman)
        );
    }

    #[test]
    fn should_detect_dnf_when_available() {
        let checker = MockChecker::with(&["dnf"]);
        assert_eq!(detect_package_manager(&checker), Some(PackageManager::Dnf));
    }

    #[test]
    fn should_detect_xbps_when_xbps_install_available() {
        let checker = MockChecker::with(&["xbps-install"]);
        assert_eq!(detect_package_manager(&checker), Some(PackageManager::Xbps));
    }

    #[test]
    fn should_detect_apk_when_available() {
        let checker = MockChecker::with(&["apk"]);
        assert_eq!(detect_package_manager(&checker), Some(PackageManager::Apk));
    }

    #[test]
    fn should_return_none_when_no_pm_detected() {
        let checker = MockChecker::with(&["ls", "sh"]);
        assert_eq!(detect_package_manager(&checker), None);
    }

    #[test]
    fn should_prefer_apt_over_pacman_when_both_available() {
        let checker = MockChecker::with(&["apt", "pacman"]);
        assert_eq!(detect_package_manager(&checker), Some(PackageManager::Apt));
    }

    // --- PackageManager::install_command ---

    #[test]
    fn should_build_apt_install_command() {
        let cmd = PackageManager::Apt.install_command(&["fzf", "tmux"]);
        assert_eq!(cmd, vec!["sudo", "apt", "install", "-y", "fzf", "tmux"]);
    }

    #[test]
    fn should_build_pacman_install_command() {
        let cmd = PackageManager::Pacman.install_command(&["fzf", "tmux"]);
        assert_eq!(
            cmd,
            vec!["sudo", "pacman", "-S", "--noconfirm", "fzf", "tmux"]
        );
    }

    #[test]
    fn should_build_dnf_install_command() {
        let cmd = PackageManager::Dnf.install_command(&["fzf"]);
        assert_eq!(cmd, vec!["sudo", "dnf", "install", "-y", "fzf"]);
    }

    #[test]
    fn should_build_yum_install_command() {
        let cmd = PackageManager::Yum.install_command(&["tmux"]);
        assert_eq!(cmd, vec!["sudo", "yum", "install", "-y", "tmux"]);
    }

    #[test]
    fn should_build_zypper_install_command() {
        let cmd = PackageManager::Zypper.install_command(&["fzf", "tmux"]);
        assert_eq!(cmd, vec!["sudo", "zypper", "install", "-y", "fzf", "tmux"]);
    }

    #[test]
    fn should_build_emerge_install_command() {
        let cmd = PackageManager::Emerge.install_command(&["fzf"]);
        assert_eq!(cmd, vec!["sudo", "emerge", "fzf"]);
    }

    #[test]
    fn should_build_xbps_install_command() {
        let cmd = PackageManager::Xbps.install_command(&["tmux"]);
        assert_eq!(cmd, vec!["sudo", "xbps-install", "-y", "tmux"]);
    }

    #[test]
    fn should_build_apk_install_command() {
        let cmd = PackageManager::Apk.install_command(&["fzf", "tmux"]);
        assert_eq!(cmd, vec!["sudo", "apk", "add", "fzf", "tmux"]);
    }

    #[test]
    fn should_build_command_for_single_package() {
        let cmd = PackageManager::Apt.install_command(&["fzf"]);
        assert_eq!(cmd, vec!["sudo", "apt", "install", "-y", "fzf"]);
    }

    #[test]
    fn should_include_sudo_for_all_package_managers() {
        for pm in PackageManager::all_ordered() {
            let cmd = pm.install_command(&["fzf"]);
            assert_eq!(
                cmd.first().map(String::as_str),
                Some("sudo"),
                "{} install command should start with sudo",
                pm.display_name()
            );
        }
    }

    #[test]
    fn should_include_all_packages_in_command() {
        let cmd = PackageManager::Apt.install_command(&["fzf", "tmux", "git"]);
        assert!(cmd.contains(&"fzf".to_string()));
        assert!(cmd.contains(&"tmux".to_string()));
        assert!(cmd.contains(&"git".to_string()));
    }
}
