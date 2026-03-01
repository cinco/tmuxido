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

/// Renders a success message after config is created
pub fn render_config_created(paths: &[String]) {
    let success_style = Style::new().bold(true).foreground(color_green());

    let path_style = Style::new().foreground(color_blue());

    let info_style = Style::new().foreground(color_dark_gray());

    println!();
    println!("{}", success_style.render("  ✅ Configuration saved!"));
    println!();
    println!("{}", info_style.render("  📂 Watching directories:"));
    for path in paths {
        println!("    {}", path_style.render(&format!("• {}", path)));
    }
    println!();
    println!(
        "{}",
        info_style
            .render("  ⚙️  You can edit ~/.config/tmuxido/tmuxido.toml later to add more paths.")
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
