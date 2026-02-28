use anyhow::{Context, Result};
use clap::Parser;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tmuxido::config::Config;
use tmuxido::{get_projects, launch_tmux_session, show_cache_status};

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
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Ensure config exists
    Config::ensure_config_exists()?;

    // Load config
    let config = Config::load()?;

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
