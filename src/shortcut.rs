use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Desktop environment variants we support
#[derive(Debug, PartialEq, Clone)]
pub enum DesktopEnv {
    Hyprland,
    Gnome,
    Kde,
    Unknown,
}

impl std::fmt::Display for DesktopEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DesktopEnv::Hyprland => write!(f, "Hyprland"),
            DesktopEnv::Gnome => write!(f, "GNOME"),
            DesktopEnv::Kde => write!(f, "KDE"),
            DesktopEnv::Unknown => write!(f, "Unknown"),
        }
    }
}

/// A keyboard shortcut combo (modifiers + key), stored in uppercase internally
#[derive(Debug, Clone, PartialEq)]
pub struct KeyCombo {
    pub modifiers: Vec<String>,
    pub key: String,
}

impl KeyCombo {
    /// Parse input like "Super+Shift+T", "super+shift+t", "SUPER+SHIFT+T"
    pub fn parse(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }
        let parts: Vec<&str> = trimmed.split('+').collect();
        if parts.len() < 2 {
            return None;
        }
        let key = parts.last()?.trim().to_uppercase();
        if key.is_empty() {
            return None;
        }
        let modifiers: Vec<String> = parts[..parts.len() - 1]
            .iter()
            .map(|s| s.trim().to_uppercase())
            .filter(|s| !s.is_empty())
            .collect();
        if modifiers.is_empty() {
            return None;
        }
        Some(KeyCombo { modifiers, key })
    }

    /// Format for Hyprland binding: "SUPER SHIFT, T"
    pub fn to_hyprland(&self) -> String {
        let mods = self.modifiers.join(" ");
        format!("{}, {}", mods, self.key)
    }

    /// Format for GNOME gsettings: "<Super><Shift>t"
    pub fn to_gnome(&self) -> String {
        let mods: String = self
            .modifiers
            .iter()
            .map(|m| {
                let mut chars = m.chars();
                let capitalized = match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().to_string() + &chars.as_str().to_lowercase(),
                };
                format!("<{}>", capitalized)
            })
            .collect();
        format!("{}{}", mods, self.key.to_lowercase())
    }

    /// Format for KDE kglobalshortcutsrc: "Meta+Shift+T"
    pub fn to_kde(&self) -> String {
        let mut parts: Vec<String> = self
            .modifiers
            .iter()
            .map(|m| match m.as_str() {
                "SUPER" | "WIN" | "META" => "Meta".to_string(),
                other => {
                    let mut chars = other.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(c) => c.to_uppercase().to_string() + &chars.as_str().to_lowercase(),
                    }
                }
            })
            .collect();
        parts.push(self.key.clone());
        parts.join("+")
    }

    /// Normalized string for dedup/comparison (uppercase, +separated)
    pub fn normalized(&self) -> String {
        let mut parts = self.modifiers.clone();
        parts.push(self.key.clone());
        parts.join("+")
    }
}

impl std::fmt::Display for KeyCombo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parts: Vec<String> = self
            .modifiers
            .iter()
            .map(|m| {
                let mut chars = m.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().to_string() + &chars.as_str().to_lowercase(),
                }
            })
            .chain(std::iter::once(self.key.clone()))
            .collect();
        write!(f, "{}", parts.join("+"))
    }
}

// ============================================================================
// Detection
// ============================================================================

/// Detect the current desktop environment from environment variables
pub fn detect_desktop() -> DesktopEnv {
    let xdg = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    let has_hyprland_sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok();
    detect_from(&xdg, has_hyprland_sig)
}

fn detect_from(xdg: &str, has_hyprland_sig: bool) -> DesktopEnv {
    let xdg_lower = xdg.to_lowercase();
    if xdg_lower.contains("hyprland") || has_hyprland_sig {
        DesktopEnv::Hyprland
    } else if xdg_lower.contains("gnome") {
        DesktopEnv::Gnome
    } else if xdg_lower.contains("kde") || xdg_lower.contains("plasma") {
        DesktopEnv::Kde
    } else {
        DesktopEnv::Unknown
    }
}

// ============================================================================
// Hyprland
// ============================================================================

/// Path to the Hyprland bindings config file
pub fn hyprland_bindings_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("Could not determine config directory")?;
    Ok(config_dir.join("hypr").join("bindings.conf"))
}

/// Calculate Hyprland modmask bitmask for a key combo
fn hyprland_modmask(combo: &KeyCombo) -> u32 {
    let mut mask = 0u32;
    for modifier in &combo.modifiers {
        mask |= match modifier.as_str() {
            "SHIFT" => 1,
            "CAPS" => 2,
            "CTRL" | "CONTROL" => 4,
            "ALT" => 8,
            "MOD2" => 16,
            "MOD3" => 32,
            "SUPER" | "WIN" | "META" => 64,
            "MOD5" => 128,
            _ => 0,
        };
    }
    mask
}

/// Check if a key combo is already bound in Hyprland via `hyprctl binds -j`.
/// Returns `Some(description)` if a conflict is found, `None` otherwise.
pub fn check_hyprland_conflict(combo: &KeyCombo) -> Option<String> {
    let output = std::process::Command::new("hyprctl")
        .args(["binds", "-j"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let json_str = String::from_utf8(output.stdout).ok()?;
    let binds: Vec<serde_json::Value> = serde_json::from_str(&json_str).ok()?;

    let target_modmask = hyprland_modmask(combo);
    let target_key = combo.key.to_lowercase();

    for bind in &binds {
        let modmask = bind["modmask"].as_u64()? as u32;
        let key = bind["key"].as_str()?.to_lowercase();
        if modmask == target_modmask && key == target_key {
            let description = if bind["has_description"].as_bool().unwrap_or(false) {
                bind["description"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string()
            } else {
                bind["dispatcher"].as_str().unwrap_or("unknown").to_string()
            };
            return Some(description);
        }
    }
    None
}

/// Determine the best launch command for Hyprland (prefers omarchy if available)
fn hyprland_launch_command() -> String {
    let available = std::process::Command::new("sh")
        .args(["-c", "command -v omarchy-launch-tui"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if available {
        "omarchy-launch-tui tmuxido".to_string()
    } else {
        "xdg-terminal-exec -e tmuxido".to_string()
    }
}

/// Write a `bindd` entry to the Hyprland bindings file.
/// Skips if any line already contains "tmuxido".
pub fn write_hyprland_binding(path: &Path, combo: &KeyCombo) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    if path.exists() {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        if content.lines().any(|l| l.contains("tmuxido")) {
            return Ok(());
        }
    }

    let launch_cmd = hyprland_launch_command();
    let line = format!(
        "bindd = {}, Tmuxido, exec, {}\n",
        combo.to_hyprland(),
        launch_cmd
    );

    use std::fs::OpenOptions;
    use std::io::Write;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?;
    file.write_all(line.as_bytes())
        .with_context(|| format!("Failed to write to {}", path.display()))?;
    Ok(())
}

// ============================================================================
// GNOME
// ============================================================================

/// Check if a combo conflicts with existing GNOME custom keybindings.
/// Returns `Some(name)` on conflict, `None` otherwise.
pub fn check_gnome_conflict(combo: &KeyCombo) -> Option<String> {
    let gnome_binding = combo.to_gnome();
    let output = std::process::Command::new("gsettings")
        .args([
            "get",
            "org.gnome.settings-daemon.plugins.media-keys",
            "custom-keybindings",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let list_str = String::from_utf8(output.stdout).ok()?;
    let paths = parse_gsettings_list(&list_str);

    for path in &paths {
        let binding = run_gsettings_custom(path, "binding")?;
        if binding.trim_matches('\'') == gnome_binding {
            let name = run_gsettings_custom(path, "name").unwrap_or_else(|| "unknown".to_string());
            return Some(name.trim_matches('\'').to_string());
        }
    }
    None
}

fn run_gsettings_custom(path: &str, key: &str) -> Option<String> {
    let schema = format!(
        "org.gnome.settings-daemon.plugins.media-keys.custom-keybindings:{}",
        path
    );
    let output = std::process::Command::new("gsettings")
        .args(["get", &schema, key])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8(output.stdout).ok()?.trim().to_string())
}

/// Parse gsettings list format `['path1', 'path2']` into a vec of path strings.
/// Also handles the GVariant empty-array notation `@as []`.
fn parse_gsettings_list(input: &str) -> Vec<String> {
    let s = input.trim();
    // Strip GVariant type hint if present: "@as [...]" → "[...]"
    let s = s.strip_prefix("@as").map(|r| r.trim()).unwrap_or(s);
    let inner = s.trim_start_matches('[').trim_end_matches(']').trim();
    if inner.is_empty() {
        return Vec::new();
    }
    inner
        .split(',')
        .map(|s| s.trim().trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Write a GNOME custom keybinding using `gsettings`
pub fn write_gnome_shortcut(combo: &KeyCombo) -> Result<()> {
    let base_schema = "org.gnome.settings-daemon.plugins.media-keys";
    let base_path = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings";

    let output = std::process::Command::new("gsettings")
        .args(["get", base_schema, "custom-keybindings"])
        .output()
        .context("Failed to run gsettings")?;

    let current_list = if output.status.success() {
        String::from_utf8(output.stdout)?.trim().to_string()
    } else {
        "@as []".to_string()
    };
    let existing = parse_gsettings_list(&current_list);

    // Find next available slot number
    let slot = (0..)
        .find(|n| {
            let candidate = format!("{}/custom{}/", base_path, n);
            !existing.contains(&candidate)
        })
        .expect("slot number is always findable");

    let slot_path = format!("{}/custom{}/", base_path, slot);
    let slot_schema = format!(
        "org.gnome.settings-daemon.plugins.media-keys.custom-keybindings:{}",
        slot_path
    );

    let mut new_list = existing.clone();
    new_list.push(slot_path.clone());
    let list_value = format!(
        "[{}]",
        new_list
            .iter()
            .map(|s| format!("'{}'", s))
            .collect::<Vec<_>>()
            .join(", ")
    );

    std::process::Command::new("gsettings")
        .args(["set", &slot_schema, "name", "Tmuxido"])
        .status()
        .context("Failed to set GNOME shortcut name")?;

    std::process::Command::new("gsettings")
        .args(["set", &slot_schema, "binding", &combo.to_gnome()])
        .status()
        .context("Failed to set GNOME shortcut binding")?;

    std::process::Command::new("gsettings")
        .args([
            "set",
            &slot_schema,
            "command",
            "xdg-terminal-exec -e tmuxido",
        ])
        .status()
        .context("Failed to set GNOME shortcut command")?;

    std::process::Command::new("gsettings")
        .args(["set", base_schema, "custom-keybindings", &list_value])
        .status()
        .context("Failed to update GNOME custom keybindings list")?;

    Ok(())
}

// ============================================================================
// KDE
// ============================================================================

/// Path to the KDE global shortcuts config file
pub fn kde_shortcuts_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("Could not determine config directory")?;
    Ok(config_dir.join("kglobalshortcutsrc"))
}

/// Check if a key combo is already bound in `kglobalshortcutsrc`.
/// Returns `Some(section_name)` on conflict, `None` otherwise.
pub fn check_kde_conflict(path: &Path, combo: &KeyCombo) -> Option<String> {
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    let kde_combo = combo.to_kde();

    let mut current_section = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed[1..trimmed.len() - 1].to_string();
            continue;
        }
        if let Some(eq_pos) = trimmed.find('=') {
            let value = &trimmed[eq_pos + 1..];
            // Format: Action=Binding,AlternativeKey,Description
            if let Some(binding) = value.split(',').next()
                && binding == kde_combo
            {
                return Some(current_section.clone());
            }
        }
    }
    None
}

/// Write a KDE global shortcut entry to `kglobalshortcutsrc`.
/// Skips if `[tmuxido]` section already exists.
pub fn write_kde_shortcut(path: &Path, combo: &KeyCombo) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    if path.exists() {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        if content.contains("[tmuxido]") {
            return Ok(());
        }
    }

    let entry = format!(
        "\n[tmuxido]\nLaunch Tmuxido={},none,Launch Tmuxido\n",
        combo.to_kde()
    );

    use std::fs::OpenOptions;
    use std::io::Write;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?;
    file.write_all(entry.as_bytes())
        .with_context(|| format!("Failed to write to {}", path.display()))?;
    Ok(())
}

// ============================================================================
// Fallback combos and conflict resolution
// ============================================================================

const FALLBACK_COMBOS: &[&str] = &[
    "Super+Shift+T",
    "Super+Shift+P",
    "Super+Ctrl+T",
    "Super+Alt+T",
    "Super+Shift+M",
    "Super+Ctrl+P",
];

/// Find the first free combo from the fallback list, skipping those in `taken`.
/// `taken` should contain normalized combo strings (uppercase, `+`-separated).
pub fn find_free_combo(taken: &[String]) -> Option<KeyCombo> {
    FALLBACK_COMBOS.iter().find_map(|s| {
        let combo = KeyCombo::parse(s)?;
        if taken.contains(&combo.normalized()) {
            None
        } else {
            Some(combo)
        }
    })
}

// ============================================================================
// Main wizard
// ============================================================================

pub fn setup_shortcut_wizard() -> Result<()> {
    let de = detect_desktop();
    crate::ui::render_section_header("Keyboard Shortcut");

    if de == DesktopEnv::Unknown {
        crate::ui::render_shortcut_unknown_de();
        return Ok(());
    }

    println!("  Detected desktop environment: {}", de);

    if !crate::ui::render_shortcut_setup_prompt()? {
        return Ok(());
    }

    let combo = loop {
        let input = crate::ui::render_key_combo_prompt("Super+Shift+T")?;
        let raw = if input.is_empty() {
            "Super+Shift+T".to_string()
        } else {
            input
        };
        if let Some(c) = KeyCombo::parse(&raw) {
            break c;
        }
        println!("  Invalid key combo. Use format like 'Super+Shift+T'");
    };

    let conflict = match &de {
        DesktopEnv::Hyprland => check_hyprland_conflict(&combo),
        DesktopEnv::Gnome => check_gnome_conflict(&combo),
        DesktopEnv::Kde => {
            let path = kde_shortcuts_path()?;
            check_kde_conflict(&path, &combo)
        }
        DesktopEnv::Unknown => unreachable!(),
    };

    let final_combo = if let Some(taken_by) = conflict {
        let taken_normalized = vec![combo.normalized()];
        if let Some(suggestion) = find_free_combo(&taken_normalized) {
            let use_suggestion = crate::ui::render_shortcut_conflict_prompt(
                &combo.to_string(),
                &taken_by,
                &suggestion.to_string(),
            )?;
            if use_suggestion {
                suggestion
            } else {
                println!("  Run 'tmuxido --setup-shortcut' again to choose a different combo.");
                return Ok(());
            }
        } else {
            println!(
                "  All fallback combos are taken. Run 'tmuxido --setup-shortcut' with a custom combo."
            );
            return Ok(());
        }
    } else {
        combo
    };

    let (details, reload_hint) = match &de {
        DesktopEnv::Hyprland => {
            let path = hyprland_bindings_path()?;
            write_hyprland_binding(&path, &final_combo)?;
            (
                format!("Added to {}", path.display()),
                "Reload Hyprland with Super+Shift+R to activate.".to_string(),
            )
        }
        DesktopEnv::Gnome => {
            write_gnome_shortcut(&final_combo)?;
            (
                "Added to GNOME custom keybindings.".to_string(),
                "The shortcut is active immediately.".to_string(),
            )
        }
        DesktopEnv::Kde => {
            let path = kde_shortcuts_path()?;
            write_kde_shortcut(&path, &final_combo)?;
            (
                format!("Added to {}", path.display()),
                "Log out and back in to activate the shortcut.".to_string(),
            )
        }
        DesktopEnv::Unknown => unreachable!(),
    };

    crate::ui::render_shortcut_success(
        &de.to_string(),
        &final_combo.to_string(),
        &details,
        &reload_hint,
    );
    Ok(())
}

// ============================================================================
// Desktop integration (.desktop file + icon)
// ============================================================================

const ICON_URL: &str = "https://raw.githubusercontent.com/cinco/tmuxido/refs/heads/main/docs/assets/tmuxido-icon_96.png";

const DESKTOP_CONTENT: &str = "[Desktop Entry]
Name=Tmuxido
Comment=Quickly find and open projects in tmux
Exec=tmuxido
Icon=tmuxido
Type=Application
Categories=Development;Utility;
Terminal=true
Keywords=tmux;project;fzf;dev;
StartupWMClass=tmuxido
";

/// Path where the .desktop entry will be installed
pub fn desktop_file_path() -> Result<PathBuf> {
    let data_dir = dirs::data_dir().context("Could not determine data directory")?;
    Ok(data_dir.join("applications").join("tmuxido.desktop"))
}

/// Path where the 96×96 icon will be installed
pub fn icon_install_path() -> Result<PathBuf> {
    let data_dir = dirs::data_dir().context("Could not determine data directory")?;
    Ok(data_dir
        .join("icons")
        .join("hicolor")
        .join("96x96")
        .join("apps")
        .join("tmuxido.png"))
}

/// Result of a desktop integration install
pub struct DesktopInstallResult {
    pub desktop_path: PathBuf,
    pub icon_path: PathBuf,
    pub icon_downloaded: bool,
}

/// Write the .desktop file and download the icon to the given paths.
/// Icon download is best-effort — does not fail if curl or network is unavailable.
pub fn install_desktop_integration_to(
    desktop_path: &Path,
    icon_path: &Path,
) -> Result<DesktopInstallResult> {
    // Write .desktop
    if let Some(parent) = desktop_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    std::fs::write(desktop_path, DESKTOP_CONTENT)
        .with_context(|| format!("Failed to write {}", desktop_path.display()))?;

    // Download icon (best-effort via curl)
    let icon_downloaded = (|| -> Option<()> {
        if let Some(parent) = icon_path.parent() {
            std::fs::create_dir_all(parent).ok()?;
        }
        std::process::Command::new("curl")
            .args(["-fsSL", ICON_URL, "-o", &icon_path.to_string_lossy()])
            .status()
            .ok()?
            .success()
            .then_some(())
    })()
    .is_some();

    // Refresh desktop database (best-effort)
    if let Some(apps_dir) = desktop_path.parent() {
        let _ = std::process::Command::new("update-desktop-database")
            .arg(apps_dir)
            .status();
    }

    // Refresh icon cache (best-effort)
    if icon_downloaded {
        // Navigate up from …/96x96/apps → …/icons/hicolor
        let hicolor_dir = icon_path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent());
        if let Some(dir) = hicolor_dir {
            let _ = std::process::Command::new("gtk-update-icon-cache")
                .args(["-f", "-t", &dir.to_string_lossy()])
                .status();
        }
    }

    Ok(DesktopInstallResult {
        desktop_path: desktop_path.to_path_buf(),
        icon_path: icon_path.to_path_buf(),
        icon_downloaded,
    })
}

/// Install .desktop and icon to the standard XDG locations
pub fn install_desktop_integration() -> Result<DesktopInstallResult> {
    install_desktop_integration_to(&desktop_file_path()?, &icon_install_path()?)
}

/// Interactive wizard that asks the user and then installs desktop integration
pub fn setup_desktop_integration_wizard() -> Result<()> {
    crate::ui::render_section_header("Desktop Integration");

    if !crate::ui::render_desktop_integration_prompt()? {
        return Ok(());
    }

    let result = install_desktop_integration()?;
    crate::ui::render_desktop_integration_success(&result);
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- detect_desktop ---

    #[test]
    fn should_detect_hyprland_from_xdg_var() {
        assert_eq!(detect_from("Hyprland", false), DesktopEnv::Hyprland);
        assert_eq!(detect_from("hyprland", false), DesktopEnv::Hyprland);
        assert_eq!(detect_from("HYPRLAND", false), DesktopEnv::Hyprland);
    }

    #[test]
    fn should_detect_hyprland_from_signature_even_without_xdg() {
        assert_eq!(detect_from("", true), DesktopEnv::Hyprland);
        assert_eq!(detect_from("somethingelse", true), DesktopEnv::Hyprland);
    }

    #[test]
    fn should_detect_gnome() {
        assert_eq!(detect_from("GNOME", false), DesktopEnv::Gnome);
        assert_eq!(detect_from("gnome", false), DesktopEnv::Gnome);
        assert_eq!(detect_from("ubuntu:GNOME", false), DesktopEnv::Gnome);
    }

    #[test]
    fn should_detect_kde_from_kde_xdg() {
        assert_eq!(detect_from("KDE", false), DesktopEnv::Kde);
        assert_eq!(detect_from("kde", false), DesktopEnv::Kde);
    }

    #[test]
    fn should_detect_kde_from_plasma_xdg() {
        assert_eq!(detect_from("Plasma", false), DesktopEnv::Kde);
        assert_eq!(detect_from("plasma", false), DesktopEnv::Kde);
    }

    #[test]
    fn should_return_unknown_for_unrecognized_de() {
        assert_eq!(detect_from("", false), DesktopEnv::Unknown);
        assert_eq!(detect_from("i3", false), DesktopEnv::Unknown);
        assert_eq!(detect_from("sway", false), DesktopEnv::Unknown);
    }

    // --- KeyCombo::parse ---

    #[test]
    fn should_parse_title_case_combo() {
        let c = KeyCombo::parse("Super+Shift+T").unwrap();
        assert_eq!(c.modifiers, vec!["SUPER", "SHIFT"]);
        assert_eq!(c.key, "T");
    }

    #[test]
    fn should_parse_lowercase_combo() {
        let c = KeyCombo::parse("super+shift+t").unwrap();
        assert_eq!(c.modifiers, vec!["SUPER", "SHIFT"]);
        assert_eq!(c.key, "T");
    }

    #[test]
    fn should_parse_uppercase_combo() {
        let c = KeyCombo::parse("SUPER+SHIFT+T").unwrap();
        assert_eq!(c.modifiers, vec!["SUPER", "SHIFT"]);
        assert_eq!(c.key, "T");
    }

    #[test]
    fn should_parse_three_modifier_combo() {
        let c = KeyCombo::parse("Super+Ctrl+Alt+F").unwrap();
        assert_eq!(c.modifiers, vec!["SUPER", "CTRL", "ALT"]);
        assert_eq!(c.key, "F");
    }

    #[test]
    fn should_return_none_for_key_only() {
        assert!(KeyCombo::parse("T").is_none());
    }

    #[test]
    fn should_return_none_for_empty_input() {
        assert!(KeyCombo::parse("").is_none());
        assert!(KeyCombo::parse("   ").is_none());
    }

    #[test]
    fn should_trim_whitespace_in_parts() {
        let c = KeyCombo::parse(" Super + Shift + T ").unwrap();
        assert_eq!(c.modifiers, vec!["SUPER", "SHIFT"]);
        assert_eq!(c.key, "T");
    }

    // --- KeyCombo formatting ---

    #[test]
    fn should_format_for_hyprland() {
        let c = KeyCombo::parse("Super+Shift+T").unwrap();
        assert_eq!(c.to_hyprland(), "SUPER SHIFT, T");
    }

    #[test]
    fn should_format_single_modifier_for_hyprland() {
        let c = KeyCombo::parse("Super+T").unwrap();
        assert_eq!(c.to_hyprland(), "SUPER, T");
    }

    #[test]
    fn should_format_for_gnome() {
        let c = KeyCombo::parse("Super+Shift+T").unwrap();
        assert_eq!(c.to_gnome(), "<Super><Shift>t");
    }

    #[test]
    fn should_format_ctrl_for_gnome() {
        let c = KeyCombo::parse("Super+Ctrl+P").unwrap();
        assert_eq!(c.to_gnome(), "<Super><Ctrl>p");
    }

    #[test]
    fn should_format_for_kde() {
        let c = KeyCombo::parse("Super+Shift+T").unwrap();
        assert_eq!(c.to_kde(), "Meta+Shift+T");
    }

    #[test]
    fn should_map_super_to_meta_for_kde() {
        let c = KeyCombo::parse("Super+Ctrl+P").unwrap();
        assert_eq!(c.to_kde(), "Meta+Ctrl+P");
    }

    #[test]
    fn should_display_in_title_case() {
        let c = KeyCombo::parse("SUPER+SHIFT+T").unwrap();
        assert_eq!(c.to_string(), "Super+Shift+T");
    }

    // --- hyprland_modmask ---

    #[test]
    fn should_calculate_modmask_for_super_shift() {
        let c = KeyCombo::parse("Super+Shift+T").unwrap();
        assert_eq!(hyprland_modmask(&c), 64 + 1); // SUPER=64, SHIFT=1
    }

    #[test]
    fn should_calculate_modmask_for_super_only() {
        let c = KeyCombo::parse("Super+T").unwrap();
        assert_eq!(hyprland_modmask(&c), 64);
    }

    #[test]
    fn should_calculate_modmask_for_ctrl_alt() {
        let c = KeyCombo::parse("Ctrl+Alt+T").unwrap();
        assert_eq!(hyprland_modmask(&c), 4 + 8); // CTRL=4, ALT=8
    }

    // --- find_free_combo ---

    #[test]
    fn should_return_first_fallback_when_nothing_taken() {
        let combo = find_free_combo(&[]).unwrap();
        assert_eq!(combo.normalized(), "SUPER+SHIFT+T");
    }

    #[test]
    fn should_skip_taken_combos() {
        let taken = vec!["SUPER+SHIFT+T".to_string(), "SUPER+SHIFT+P".to_string()];
        let combo = find_free_combo(&taken).unwrap();
        assert_eq!(combo.normalized(), "SUPER+CTRL+T");
    }

    #[test]
    fn should_return_none_when_all_fallbacks_taken() {
        let taken: Vec<String> = FALLBACK_COMBOS
            .iter()
            .map(|s| KeyCombo::parse(s).unwrap().normalized())
            .collect();
        assert!(find_free_combo(&taken).is_none());
    }

    // --- parse_gsettings_list ---

    #[test]
    fn should_parse_empty_gsettings_list() {
        assert!(parse_gsettings_list("[]").is_empty());
        assert!(parse_gsettings_list("@as []").is_empty());
        assert!(parse_gsettings_list("  [ ]  ").is_empty());
    }

    #[test]
    fn should_parse_gsettings_list_with_one_entry() {
        let result =
            parse_gsettings_list("['/org/gnome/settings-daemon/plugins/media-keys/custom0/']");
        assert_eq!(
            result,
            vec!["/org/gnome/settings-daemon/plugins/media-keys/custom0/"]
        );
    }

    #[test]
    fn should_parse_gsettings_list_with_multiple_entries() {
        let result = parse_gsettings_list("['/org/gnome/.../custom0/', '/org/gnome/.../custom1/']");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "/org/gnome/.../custom0/");
        assert_eq!(result[1], "/org/gnome/.../custom1/");
    }

    // --- check_kde_conflict ---

    #[test]
    fn should_return_none_when_kde_file_missing() {
        let combo = KeyCombo::parse("Super+Shift+T").unwrap();
        assert!(check_kde_conflict(Path::new("/nonexistent/path"), &combo).is_none());
    }

    // --- normalized ---

    #[test]
    fn should_normalize_to_uppercase_plus_separated() {
        let c = KeyCombo::parse("super+shift+t").unwrap();
        assert_eq!(c.normalized(), "SUPER+SHIFT+T");
    }

    // --- desktop integration ---

    #[test]
    fn should_write_desktop_file_to_given_path() {
        let dir = tempfile::tempdir().unwrap();
        let desktop = dir.path().join("apps").join("tmuxido.desktop");
        let icon = dir.path().join("icons").join("tmuxido.png");

        let result = install_desktop_integration_to(&desktop, &icon).unwrap();

        assert!(result.desktop_path.exists());
        let content = std::fs::read_to_string(&result.desktop_path).unwrap();
        assert!(content.contains("[Desktop Entry]"));
        assert!(content.contains("Exec=tmuxido"));
        assert!(content.contains("Icon=tmuxido"));
        assert!(content.contains("Terminal=true"));
    }

    #[test]
    fn should_create_parent_directories_for_desktop_file() {
        let dir = tempfile::tempdir().unwrap();
        let desktop = dir
            .path()
            .join("nested")
            .join("apps")
            .join("tmuxido.desktop");
        let icon = dir.path().join("icons").join("tmuxido.png");

        install_desktop_integration_to(&desktop, &icon).unwrap();

        assert!(desktop.exists());
    }

    #[test]
    fn desktop_content_contains_required_fields() {
        assert!(DESKTOP_CONTENT.contains("[Desktop Entry]"));
        assert!(DESKTOP_CONTENT.contains("Name=Tmuxido"));
        assert!(DESKTOP_CONTENT.contains("Exec=tmuxido"));
        assert!(DESKTOP_CONTENT.contains("Icon=tmuxido"));
        assert!(DESKTOP_CONTENT.contains("Type=Application"));
        assert!(DESKTOP_CONTENT.contains("Terminal=true"));
        assert!(DESKTOP_CONTENT.contains("StartupWMClass=tmuxido"));
    }
}
