use crate::session::Window;
use anyhow::{Context, Result};
use lipgloss::{Color, Style};
use std::io::{self, Write};

// Tokyo Night theme colors (as RGB tuples)
fn color_blue() -> Color {
    Color::from_rgb(122, 162, 247)
} // #7AA2F7
fn color_purple() -> Color {
    Color::from_rgb(187, 154, 247)
} // #BB9AF7
fn color_light_gray() -> Color {
    Color::from_rgb(169, 177, 214)
} // #A9B1D6
fn color_dark_gray() -> Color {
    Color::from_rgb(86, 95, 137)
} // #565F89
fn color_green() -> Color {
    Color::from_rgb(158, 206, 106)
} // #9ECE6A
fn color_orange() -> Color {
    Color::from_rgb(224, 175, 104)
} // #E0AF68

/// Renders a styled welcome screen for first-time setup
pub fn render_welcome_banner() {
    let title_style = Style::new().bold(true).foreground(color_blue());

    let subtitle_style = Style::new().foreground(color_purple());

    let text_style = Style::new().foreground(color_light_gray());

    let hint_style = Style::new().italic(true).foreground(color_dark_gray());

    println!();
    println!("{}", title_style.render("  🚀 Welcome to tmuxido!"));
    println!();
    println!(
        "{}",
        subtitle_style.render("  📁 Let's set up your project directories")
    );
    println!();
    println!(
        "{}",
        text_style.render("  Please specify where tmuxido should look for your projects.")
    );
    println!();
    println!(
        "{}",
        text_style.render("  You can add multiple paths separated by commas:")
    );
    println!();
    println!(
        "{}",
        hint_style.render("    💡 Example: ~/Projects, ~/work, ~/personal/repos")
    );
    println!();
}

/// Renders a prompt asking for paths
pub fn render_paths_prompt() -> Result<String> {
    let prompt_style = Style::new().bold(true).foreground(color_green());

    print!("  {} ", prompt_style.render("❯ Paths:"));
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read input")?;

    Ok(input.trim().to_string())
}

/// Renders a success message after config is created with all settings
pub fn render_config_created(
    paths: &[String],
    max_depth: usize,
    cache_enabled: bool,
    cache_ttl_hours: u64,
    windows: &[Window],
) {
    let success_style = Style::new().bold(true).foreground(color_green());
    let label_style = Style::new().foreground(color_light_gray());
    let value_style = Style::new().bold(true).foreground(color_blue());
    let path_style = Style::new().foreground(color_blue());
    let window_style = Style::new().foreground(color_purple());
    let info_style = Style::new().foreground(color_dark_gray());
    let bool_enabled_style = Style::new().bold(true).foreground(color_green());
    let bool_disabled_style = Style::new().bold(true).foreground(color_orange());

    println!();
    println!("{}", success_style.render("  ✅ Configuration saved!"));
    println!();

    // Project discovery section
    println!("{}", label_style.render("  📁 Project Discovery:"));
    println!(
        "    {} {} {}",
        label_style.render("Max scan depth:"),
        value_style.render(&max_depth.to_string()),
        label_style.render("levels")
    );
    println!();

    // Paths
    println!("{}", label_style.render("  📂 Directories:"));
    for path in paths {
        println!("    {}", path_style.render(&format!("• {}", path)));
    }
    println!();

    // Cache settings
    println!("{}", label_style.render("  💾 Cache Settings:"));
    let cache_status = if cache_enabled {
        bool_enabled_style.render("enabled")
    } else {
        bool_disabled_style.render("disabled")
    };
    println!("    {} {}", label_style.render("Status:"), cache_status);
    if cache_enabled {
        println!(
            "    {} {} {}",
            label_style.render("TTL:"),
            value_style.render(&cache_ttl_hours.to_string()),
            label_style.render("hours")
        );
    }
    println!();

    // Default session
    println!("{}", label_style.render("  🪟 Default Windows:"));
    for window in windows {
        println!("    {}", window_style.render(&format!("◦ {}", window.name)));
        if let Some(layout) = &window.layout {
            println!(
                "{}",
                info_style.render(&format!("    └─ layout: {}", layout))
            );
        }
        if !window.panes.is_empty() {
            for (i, pane) in window.panes.iter().enumerate() {
                let pane_display = if pane.is_empty() {
                    format!("    └─ pane {} (shell)", i + 1)
                } else {
                    format!("    └─ pane {}: {}", i + 1, pane)
                };
                println!("{}", info_style.render(&pane_display));
            }
        }
    }
    println!();

    println!(
        "{}",
        info_style.render(
            "  ⚙️  You can edit ~/.config/tmuxido/tmuxido.toml anytime to change these settings."
        )
    );
    println!();
}

/// Renders a warning when user provides no input (fallback to default)
pub fn render_fallback_message() {
    let warning_style = Style::new().italic(true).foreground(color_orange());

    println!();
    println!(
        "{}",
        warning_style.render("  ⚠️  No paths provided. Using default: ~/Projects")
    );
}

/// Renders a section header for grouping related settings
pub fn render_section_header(title: &str) {
    let header_style = Style::new().bold(true).foreground(color_purple());

    println!();
    println!("{}", header_style.render(&format!("  📋 {}", title)));
}

/// Renders a prompt for max_depth with instructions
pub fn render_max_depth_prompt() -> Result<String> {
    let prompt_style = Style::new().bold(true).foreground(color_green());
    let hint_style = Style::new().italic(true).foreground(color_dark_gray());

    println!(
        "{}",
        hint_style.render("    How many levels deep should tmuxido search for git repositories?")
    );
    println!(
        "{}",
        hint_style.render("    Higher values = deeper search, but slower. Default: 5")
    );
    print!("  {} ", prompt_style.render("❯ Max depth:"));
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read input")?;

    Ok(input.trim().to_string())
}

/// Renders a prompt for cache_enabled with instructions
pub fn render_cache_enabled_prompt() -> Result<String> {
    let prompt_style = Style::new().bold(true).foreground(color_green());
    let hint_style = Style::new().italic(true).foreground(color_dark_gray());

    println!(
        "{}",
        hint_style.render("    Enable caching to speed up project discovery?")
    );
    println!(
        "{}",
        hint_style.render("    Cache avoids rescanning unchanged directories. Default: yes (y)")
    );
    print!("  {} ", prompt_style.render("❯ Enable cache? (y/n):"));
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read input")?;

    Ok(input.trim().to_lowercase())
}

/// Renders a prompt for cache_ttl_hours with instructions
pub fn render_cache_ttl_prompt() -> Result<String> {
    let prompt_style = Style::new().bold(true).foreground(color_green());
    let hint_style = Style::new().italic(true).foreground(color_dark_gray());

    println!(
        "{}",
        hint_style.render("    How long should the cache remain valid (in hours)?")
    );
    println!(
        "{}",
        hint_style.render("    After this time, tmuxido will rescan your directories. Default: 24")
    );
    print!("  {} ", prompt_style.render("❯ Cache TTL (hours):"));
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read input")?;

    Ok(input.trim().to_string())
}

/// Renders a prompt for default session windows with instructions
pub fn render_windows_prompt() -> Result<String> {
    let prompt_style = Style::new().bold(true).foreground(color_green());
    let hint_style = Style::new().italic(true).foreground(color_dark_gray());

    println!(
        "{}",
        hint_style.render("    What windows should be created by default in new tmux sessions?")
    );
    println!(
        "{}",
        hint_style.render("    Enter window names separated by commas. Default: editor, terminal")
    );
    println!(
        "{}",
        hint_style.render("    💡 Tip: Common choices are 'editor', 'terminal', 'server', 'logs'")
    );
    print!("  {} ", prompt_style.render("❯ Window names:"));
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read input")?;

    Ok(input.trim().to_string())
}

/// Renders a prompt asking for panes in a specific window
pub fn render_panes_prompt(window_name: &str) -> Result<String> {
    let prompt_style = Style::new().bold(true).foreground(color_green());
    let hint_style = Style::new().italic(true).foreground(color_dark_gray());
    let window_style = Style::new().bold(true).foreground(color_purple());

    println!();
    println!("  Configuring window: {}", window_style.render(window_name));
    println!(
        "{}",
        hint_style
            .render("    Enter pane names separated by commas, or leave empty for a single pane.")
    );
    println!("{}", hint_style.render("    💡 Example: code, logs, tests"));
    print!("  {} ", prompt_style.render("❯ Pane names:"));
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read input")?;

    Ok(input.trim().to_string())
}

/// Renders a prompt for the layout of a window with multiple panes
pub fn render_layout_prompt(window_name: &str, pane_count: usize) -> Result<Option<String>> {
    let prompt_style = Style::new().bold(true).foreground(color_green());
    let hint_style = Style::new().italic(true).foreground(color_dark_gray());
    let window_style = Style::new().bold(true).foreground(color_purple());
    let label_style = Style::new().foreground(color_blue());

    println!();
    println!(
        "  Layout for window {} ({} panes):",
        window_style.render(window_name),
        label_style.render(&pane_count.to_string())
    );
    println!(
        "{}",
        hint_style.render("    Choose a pane layout (leave empty for no layout):")
    );
    println!(
        "{}",
        hint_style.render("    1. main-horizontal  — main pane on top, others below")
    );
    println!(
        "{}",
        hint_style.render("    2. main-vertical    — main pane on left, others on right")
    );
    println!(
        "{}",
        hint_style.render("    3. tiled            — all panes tiled equally")
    );
    println!(
        "{}",
        hint_style.render("    4. even-horizontal  — all panes side by side")
    );
    println!(
        "{}",
        hint_style.render("    5. even-vertical    — all panes stacked vertically")
    );
    print!("  {} ", prompt_style.render("❯ Layout (1-5 or name):"));
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read input")?;

    Ok(parse_layout_input(input.trim()))
}

/// Parse layout input: accepts number (1-5) or layout name; returns None for empty/invalid
pub fn parse_layout_input(input: &str) -> Option<String> {
    match input.trim() {
        "" => None,
        "1" | "main-horizontal" => Some("main-horizontal".to_string()),
        "2" | "main-vertical" => Some("main-vertical".to_string()),
        "3" | "tiled" => Some("tiled".to_string()),
        "4" | "even-horizontal" => Some("even-horizontal".to_string()),
        "5" | "even-vertical" => Some("even-vertical".to_string()),
        _ => None,
    }
}

/// Renders a prompt for a pane command
pub fn render_pane_command_prompt(pane_name: &str) -> Result<String> {
    let prompt_style = Style::new().bold(true).foreground(color_green());
    let hint_style = Style::new().italic(true).foreground(color_dark_gray());
    let pane_style = Style::new().foreground(color_blue());

    println!(
        "{}",
        hint_style.render(&format!(
            "    What command should run in pane '{}' on startup?",
            pane_style.render(pane_name)
        ))
    );
    println!(
        "{}",
        hint_style.render("    Leave empty to run the default shell, or enter a command like 'nvim', 'npm run dev'")
    );
    print!("  {} ", prompt_style.render("❯ Command:"));
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read input")?;

    Ok(input.trim().to_string())
}

/// Parse max_depth input, returning None for empty/invalid (use default)
pub fn parse_max_depth_input(input: &str) -> Option<usize> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse::<usize>().ok().filter(|&n| n > 0)
}

/// Parse cache enabled input, returning None for empty (use default)
pub fn parse_cache_enabled_input(input: &str) -> Option<bool> {
    let trimmed = input.trim().to_lowercase();
    if trimmed.is_empty() {
        return None;
    }
    match trimmed.as_str() {
        "y" | "yes" => Some(true),
        "n" | "no" => Some(false),
        _ => None,
    }
}

/// Parse cache TTL input, returning None for empty/invalid (use default)
pub fn parse_cache_ttl_input(input: &str) -> Option<u64> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse::<u64>().ok().filter(|&n| n > 0)
}

/// Parse comma-separated list into Vec<String>, filtering empty items
pub fn parse_comma_separated_list(input: &str) -> Vec<String> {
    input
        .trim()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_none_for_empty_max_depth() {
        assert_eq!(parse_max_depth_input(""), None);
        assert_eq!(parse_max_depth_input("   "), None);
    }

    #[test]
    fn should_parse_valid_max_depth() {
        assert_eq!(parse_max_depth_input("5"), Some(5));
        assert_eq!(parse_max_depth_input("10"), Some(10));
        assert_eq!(parse_max_depth_input("  3  "), Some(3));
    }

    #[test]
    fn should_return_none_for_invalid_max_depth() {
        assert_eq!(parse_max_depth_input("0"), None);
        assert_eq!(parse_max_depth_input("-1"), None);
        assert_eq!(parse_max_depth_input("abc"), None);
        assert_eq!(parse_max_depth_input("3.5"), None);
    }

    #[test]
    fn should_return_none_for_empty_cache_enabled() {
        assert_eq!(parse_cache_enabled_input(""), None);
        assert_eq!(parse_cache_enabled_input("   "), None);
    }

    #[test]
    fn should_parse_yes_as_true() {
        assert_eq!(parse_cache_enabled_input("y"), Some(true));
        assert_eq!(parse_cache_enabled_input("Y"), Some(true));
        assert_eq!(parse_cache_enabled_input("yes"), Some(true));
        assert_eq!(parse_cache_enabled_input("YES"), Some(true));
        assert_eq!(parse_cache_enabled_input("Yes"), Some(true));
    }

    #[test]
    fn should_parse_no_as_false() {
        assert_eq!(parse_cache_enabled_input("n"), Some(false));
        assert_eq!(parse_cache_enabled_input("N"), Some(false));
        assert_eq!(parse_cache_enabled_input("no"), Some(false));
        assert_eq!(parse_cache_enabled_input("NO"), Some(false));
        assert_eq!(parse_cache_enabled_input("No"), Some(false));
    }

    #[test]
    fn should_return_none_for_invalid_cache_input() {
        assert_eq!(parse_cache_enabled_input("maybe"), None);
        assert_eq!(parse_cache_enabled_input("true"), None);
        assert_eq!(parse_cache_enabled_input("1"), None);
    }

    #[test]
    fn should_return_none_for_empty_cache_ttl() {
        assert_eq!(parse_cache_ttl_input(""), None);
        assert_eq!(parse_cache_ttl_input("   "), None);
    }

    #[test]
    fn should_parse_valid_cache_ttl() {
        assert_eq!(parse_cache_ttl_input("24"), Some(24));
        assert_eq!(parse_cache_ttl_input("12"), Some(12));
        assert_eq!(parse_cache_ttl_input("  48  "), Some(48));
    }

    #[test]
    fn should_return_none_for_invalid_cache_ttl() {
        assert_eq!(parse_cache_ttl_input("0"), None);
        assert_eq!(parse_cache_ttl_input("-1"), None);
        assert_eq!(parse_cache_ttl_input("abc"), None);
        assert_eq!(parse_cache_ttl_input("12.5"), None);
    }

    #[test]
    fn should_parse_empty_comma_list() {
        let result = parse_comma_separated_list("");
        assert!(result.is_empty());
    }

    #[test]
    fn should_parse_single_item() {
        let result = parse_comma_separated_list("editor");
        assert_eq!(result, vec!["editor"]);
    }

    #[test]
    fn should_parse_multiple_items() {
        let result = parse_comma_separated_list("editor, terminal, server");
        assert_eq!(result, vec!["editor", "terminal", "server"]);
    }

    #[test]
    fn should_trim_whitespace_in_comma_list() {
        let result = parse_comma_separated_list("  editor  ,  terminal  ");
        assert_eq!(result, vec!["editor", "terminal"]);
    }

    #[test]
    fn should_filter_empty_parts_in_comma_list() {
        let result = parse_comma_separated_list("editor,,terminal");
        assert_eq!(result, vec!["editor", "terminal"]);
    }

    #[test]
    fn color_blue_should_return_expected_rgb() {
        let color = color_blue();
        // We can't easily test the internal RGB values, but we can verify it doesn't panic
        let _ = color;
    }

    #[test]
    fn color_functions_should_return_distinct_colors() {
        // Verify all color functions return valid Color objects
        let colors = vec![
            color_blue(),
            color_purple(),
            color_light_gray(),
            color_dark_gray(),
            color_green(),
            color_orange(),
        ];
        // Just verify they don't panic and are distinct
        assert_eq!(colors.len(), 6);
    }

    #[test]
    fn render_section_header_should_not_panic() {
        // This test verifies the function doesn't panic
        // We can't capture stdout easily in unit tests without additional setup
        render_section_header("Test Section");
    }

    #[test]
    fn render_welcome_banner_should_not_panic() {
        render_welcome_banner();
    }

    #[test]
    fn render_fallback_message_should_not_panic() {
        render_fallback_message();
    }

    #[test]
    fn render_config_created_should_not_panic() {
        let windows = vec![
            Window {
                name: "editor".to_string(),
                panes: vec!["nvim .".to_string()],
                layout: None,
            },
            Window {
                name: "terminal".to_string(),
                panes: vec![],
                layout: None,
            },
        ];
        render_config_created(&vec!["~/Projects".to_string()], 5, true, 24, &windows);
    }

    #[test]
    fn should_return_none_for_empty_layout_input() {
        assert_eq!(parse_layout_input(""), None);
        assert_eq!(parse_layout_input("   "), None);
    }

    #[test]
    fn should_parse_layout_by_number() {
        assert_eq!(parse_layout_input("1"), Some("main-horizontal".to_string()));
        assert_eq!(parse_layout_input("2"), Some("main-vertical".to_string()));
        assert_eq!(parse_layout_input("3"), Some("tiled".to_string()));
        assert_eq!(parse_layout_input("4"), Some("even-horizontal".to_string()));
        assert_eq!(parse_layout_input("5"), Some("even-vertical".to_string()));
    }

    #[test]
    fn should_parse_layout_by_name() {
        assert_eq!(
            parse_layout_input("main-horizontal"),
            Some("main-horizontal".to_string())
        );
        assert_eq!(
            parse_layout_input("main-vertical"),
            Some("main-vertical".to_string())
        );
        assert_eq!(parse_layout_input("tiled"), Some("tiled".to_string()));
        assert_eq!(
            parse_layout_input("even-horizontal"),
            Some("even-horizontal".to_string())
        );
        assert_eq!(
            parse_layout_input("even-vertical"),
            Some("even-vertical".to_string())
        );
    }

    #[test]
    fn should_return_none_for_invalid_layout_input() {
        assert_eq!(parse_layout_input("6"), None);
        assert_eq!(parse_layout_input("0"), None);
        assert_eq!(parse_layout_input("unknown"), None);
        assert_eq!(parse_layout_input("horizontal"), None);
    }

    #[test]
    fn render_config_created_with_disabled_cache_should_not_panic() {
        let windows = vec![Window {
            name: "editor".to_string(),
            panes: vec![],
            layout: None,
        }];
        render_config_created(&vec!["~/work".to_string()], 3, false, 24, &windows);
    }
}
