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
