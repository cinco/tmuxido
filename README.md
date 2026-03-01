<div align="center">
  <img src="docs/assets/tmuxido-logo.png" alt="tmuxido logo" width="200"/>
</div>
<div align="center">

[![Build Status](https://drone.cincoeuzebio.com/api/badges/cinco/Tmuxido/status.svg)](https://drone.cincoeuzebio.com/cinco/Tmuxido)
[![Coverage](https://git.cincoeuzebio.com/cinco/Tmuxido/raw/branch/badges/coverage.svg)](https://drone.cincoeuzebio.com/cinco/Tmuxido)
[![Version](https://img.shields.io/gitea/v/release/cinco/Tmuxido?gitea_url=https%3A%2F%2Fgit.cincoeuzebio.com&label=version)](https://git.cincoeuzebio.com/cinco/Tmuxido/releases)
![Rust 2026](https://img.shields.io/badge/rust-edition_2026-orange?logo=rust)

</div>

# tmuxido

A Rust-based tool to quickly find and open projects in tmux using fzf. No external dependencies except tmux and fzf!

## Features

- Search for git repositories in configurable paths
- Interactive selection using fzf
- Native tmux session creation (no tmuxinator required!)
- Support for project-specific `.tmuxido.toml` configs
- Smart session switching (reuses existing sessions)
- TOML-based configuration
- Smart caching system for fast subsequent runs
- Configurable cache TTL
- Self-update capability (`tmuxido --update`)
- Zero external dependencies (except tmux and fzf)

## Installation

```sh
curl -fsSL https://git.cincoeuzebio.com/cinco/Tmuxido/raw/branch/main/install.sh | sh
```

Installs the latest release binary to `~/.local/bin/tmuxido`. On first run, the config file is created automatically at `~/.config/tmuxido/tmuxido.toml`.

### Build from source

```bash
cargo build --release
cp target/release/tmuxido ~/.local/bin/
```

## Configuration

The configuration file is located at `~/.config/tmuxido/tmuxido.toml`.

On first run, a default configuration will be created automatically.

Example configuration:
```toml
# List of paths where to search for projects (git repositories)
paths = [
    "~/Projects",
    # "~/work",
]

# Maximum depth to search for .git directories
max_depth = 5

# Enable project caching (default: true)
cache_enabled = true

# Cache TTL in hours (default: 24)
cache_ttl_hours = 24

# Default session configuration (used when project has no .tmuxido.toml)
[default_session]

[[default_session.windows]]
name = "editor"
panes = []

[[default_session.windows]]
name = "terminal"
panes = []
```

## Usage

Run without arguments to search all configured paths and select with fzf:
```bash
tmuxido
```

Or provide a specific directory:
```bash
tmuxido /path/to/project
```

Force refresh the cache (useful after adding new projects):
```bash
tmuxido --refresh
# or
tmuxido -r
```

Check cache status:
```bash
tmuxido --cache-status
```

Update tmuxido to the latest version:
```bash
tmuxido --update
```

View help:
```bash
tmuxido --help
```

## Requirements

- [tmux](https://github.com/tmux/tmux) - Terminal multiplexer
- [fzf](https://github.com/junegunn/fzf) - For interactive selection

## How it works

1. Searches for git repositories (directories containing `.git`) in configured paths
2. Caches the results for faster subsequent runs
3. Presents them using fzf for selection
4. Creates or switches to a tmux session for the selected project
5. If a `.tmuxido.toml` config exists in the project, uses it to set up custom windows and panes
6. Otherwise, creates a default session with two windows: "editor" and "terminal"

## Caching

The tool uses an incremental cache to keep subsequent runs fast:

- **Cache location**: `~/.cache/tmuxido/projects.json`
- **Incremental updates**: On each run, only directories whose mtime changed are rescanned — no full rescans
- **Manual refresh**: Use `--refresh` to force a full rescan
- **Cache status**: Use `--cache-status` to inspect the cache

The cache persists indefinitely and is updated automatically when the filesystem changes.

## Project-specific Configuration

You can customize the tmux session layout for individual projects by creating a `.tmuxido.toml` file in the project root.

Example `.tmuxido.toml`:
```toml
[[windows]]
name = "editor"
panes = ["nvim"]
layout = "main-horizontal"

[[windows]]
name = "server"
panes = ["npm run dev"]

[[windows]]
name = "git"
panes = []
```

### Available Layouts

- `main-horizontal` - Main pane on top, others below
- `main-vertical` - Main pane on left, others on right
- `tiled` - All panes tiled
- `even-horizontal` - All panes in horizontal row
- `even-vertical` - All panes in vertical column

### Panes

Each window can have multiple panes with commands that run automatically:
- First pane is the main window pane
- Additional panes are created by splitting
- Empty panes array = just open the window in the project directory

## Author

<div align="center">
  <a href="https://github.com/cinco">
    <img src="https://github.com/cinco.png" width="100" height="100" style="border-radius: 50%;" alt="Cinco avatar"/>
  </a>
  <br><br>
  <strong>Cinco</strong>
  <br>
  <a href="https://github.com/cinco">@cinco</a>
</div>
