use anyhow::{Context, Result};
use clap::Parser;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tmuxido::config::Config;
use tmuxido::deps::ensure_dependencies;
use tmuxido::self_update;
use tmuxido::update_check;
use tmuxido::{
    get_projects, launch_tmux_session, setup_desktop_integration_wizard, setup_shortcut_wizard,
    show_cache_status,
};

#[derive(Parser, Debug)]
#[command(
    name = "tmuxido",
    about = "Quickly find and open projects in tmux",
    version
)]
struct Args {
    /// Project path to open directly (skips selection)
    project_path: Option<PathBuf>,

    /// Force refresh the project cache
    #[arg(short, long)]
    refresh: bool,

    /// Show cache status and exit
    #[arg(long)]
    cache_status: bool,

    /// Update tmuxido to the latest version
    #[arg(long)]
    update: bool,

    /// Set up a keyboard shortcut to launch tmuxido
    #[arg(long)]
    setup_shortcut: bool,

    /// Install the .desktop entry and icon for app launcher integration
    #[arg(long)]
    setup_desktop_shortcut: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Handle self-update before anything else
    if args.update {
        return self_update::self_update();
    }

    // Handle standalone shortcut setup
    if args.setup_shortcut {
        return setup_shortcut_wizard();
    }

    // Handle standalone desktop integration setup
    if args.setup_desktop_shortcut {
        return setup_desktop_integration_wizard();
    }

    // Check that fzf and tmux are installed; offer to install if missing
    ensure_dependencies()?;

    // Ensure config exists
    Config::ensure_config_exists()?;

    // Load config
    let config = Config::load()?;

    // Periodic update check (silent on failure or no update)
    update_check::check_and_notify(&config);

    // Handle cache status command
    if args.cache_status {
        show_cache_status(&config)?;
        return Ok(());
    }

    let selected = if let Some(path) = args.project_path {
        path
    } else {
        // Get projects (from cache or scan)
        let projects = get_projects(&config, args.refresh)?;

        if projects.is_empty() {
            eprintln!("No projects found in configured paths");
            std::process::exit(1);
        }

        // Use fzf to select a project
        select_project_with_fzf(&projects)?
    };

    if !selected.exists() {
        eprintln!("Selected path does not exist: {}", selected.display());
        std::process::exit(1);
    }

    // Launch tmux session
    launch_tmux_session(&selected, &config)?;

    Ok(())
}

fn select_project_with_fzf(projects: &[PathBuf]) -> Result<PathBuf> {
    let mut child = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to spawn fzf. Make sure fzf is installed.")?;

    {
        let stdin = child.stdin.as_mut().context("Failed to open stdin")?;
        for project in projects {
            writeln!(stdin, "{}", project.display())?;
        }
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        std::process::exit(0);
    }

    let selected = String::from_utf8(output.stdout)?.trim().to_string();

    if selected.is_empty() {
        std::process::exit(0);
    }

    Ok(PathBuf::from(selected))
}
