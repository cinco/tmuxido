# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.9.1] - 2026-03-01

### Fixed
- Shortcut and desktop integration wizards are now offered regardless of whether the user chose the interactive wizard or the default config on first run; previously they were only offered in the wizard path

## [0.9.0] - 2026-03-01

### Added
- First-run setup choice prompt: when no configuration file exists, tmuxido now asks whether to run the interactive wizard or apply sensible defaults immediately
- `SetupChoice` enum and `parse_setup_choice_input` in `ui` module (pure, fully tested)
- `Config::write_default_config` helper for writing defaults without any prompts
- `Config::run_wizard` extracted from `ensure_config_exists` for clarity and testability
- `render_setup_choice_prompt` and `render_default_config_saved` render functions

## [0.8.3] - 2026-03-01

### Fixed
- `Cargo.lock` now committed alongside version bumps

## [0.8.2] - 2026-03-01

### Fixed
- `install.sh`: grep pattern for `tag_name` now handles the space GitHub includes after the colon in JSON (`"tag_name": "x"` instead of `"tag_name":"x"`)

## [0.8.1] - 2026-03-01

### Fixed
- `install.sh`: removed `-f` flag from GitHub API `curl` call so HTTP error responses (rate limits, 404s) are printed instead of silently discarded; shows up to 400 bytes of the raw API response when the release tag cannot be parsed

## [0.8.0] - 2026-03-01

### Added
- Keyboard shortcut setup wizard on first run and via `tmuxido --setup-shortcut`
- Auto-detects desktop environment from `XDG_CURRENT_DESKTOP` / `HYPRLAND_INSTANCE_SIGNATURE`
- Hyprland: appends `bindd` entry to `~/.config/hypr/bindings.conf`; prefers `omarchy-launch-tui` when available, falls back to `xdg-terminal-exec`
- GNOME: registers a custom keybinding via `gsettings`
- KDE: appends a `[tmuxido]` section to `~/.config/kglobalshortcutsrc`
- Conflict detection per DE (Hyprland via `hyprctl binds -j`, KDE via config file, GNOME via gsettings); suggests next free combo from a fallback list
- `--setup-desktop-shortcut` flag to (re-)install the `.desktop` entry and icon at any time
- `shortcut` module (`src/shortcut.rs`) with full unit and integration test coverage
- Icon and `.desktop` file installed by `install.sh` and offered in the first-run wizard

## [0.7.1] - 2026-03-01

### Fixed
- Interactive setup wizard now asks for a tmux layout when a window has 2 or more panes
- Layout selection shown in post-wizard summary

### Changed
- README: Added ASCII art previews for each available tmux layout

## [0.7.0] - 2026-03-01

### Changed
- `install.sh` now downloads from GitHub Releases
- Self-update now queries the GitHub Releases API for new versions
- Releases are published to both Gitea and GitHub

## [0.6.0] - 2026-03-01

### Added
- Periodic update check: on startup, if `update_check_interval_hours` have elapsed since
  the last check, tmuxido fetches the latest release tag from the Gitea API and prints a
  notice when a newer version is available (silent on network failure or no update found)
- New `update_check` module (`src/update_check.rs`) with injected fetcher for testability
- `update_check_interval_hours` config field (default 24, set to 0 to disable)
- Cache file `~/.cache/tmuxido/update_check.json` tracks last-checked timestamp and
  latest known version across runs

## [0.5.2] - 2026-03-01

### Added
- Test for `detect_arch` asserting asset name follows `tmuxido-{arch}-linux` format

## [0.5.1] - 2026-03-01

### Fixed
- Tmux window creation now targets windows by name instead of numeric index, eliminating
  "index in use" and "can't find window" errors when `base-index` is not 0
- Self-update asset name corrected from `x86_64-linux` to `tmuxido-x86_64-linux` to match
  what CI actually uploads, fixing 404 on `--update`
- CI release pipeline now deletes any existing release for the tag before recreating,
  preventing 409 Conflict errors on retagged releases

## [0.5.0] - 2026-03-01

### Added
- Interactive configuration wizard on first run with styled prompts
- `lipgloss` dependency for beautiful terminal UI with Tokyo Night theme colors
- Emoji-enhanced prompts and feedback during setup
- Configure project paths interactively with comma-separated input
- Configure `max_depth` for project discovery scanning
- Configure cache settings (`cache_enabled`, `cache_ttl_hours`)
- Configure default session windows interactively
- Configure panes within each window with custom names
- Configure startup commands for each pane (e.g., `nvim .`, `npm run dev`)
- New `ui` module with styled render functions for all prompts
- Comprehensive summary showing all configured settings after setup

## [0.4.2] - 2026-03-01

### Fixed
- Version mismatch: bumped Cargo.toml version to match release tag, fixing `--update` false positive

## [0.4.1] - 2026-03-01

### Added
- Self-update feature (`tmuxido --update`) to update binary from latest GitHub release

## [0.4.0] - 2026-03-01

### Added
- Self-update feature (`tmuxido --update`) to update binary from latest GitHub release
- New `self_update` module with version comparison and atomic binary replacement
- `--update` CLI flag for in-place binary updates
- Backup and rollback mechanism if update fails

## [0.3.0] - 2026-03-01

### Added
- Dependency check for `fzf` and `tmux` at startup, before any operation
- Automatic Linux package manager detection (apt, pacman, dnf, yum, zypper, emerge, xbps, apk)
- Interactive installation prompt when required tools are missing
- `deps` module with injectable `BinaryChecker` trait for unit testing without hitting the real system
- Integration tests in `tests/deps.rs` (11 tests using real `SystemBinaryChecker`)
- Docker test suite in `tests/docker/` with 15 scenarios simulating a fresh Ubuntu 24.04 user

### Fixed
- Release pipeline `publish` step now reads `DRONE_TAG` via awk `ENVIRON` to prevent Drone's
  `${VAR}` substitution from wiping local shell variables before the shell runs

## [0.2.4] - 2026-03-01

### Fixed
- Coverage percentage calculation in CI (correct field from tarpaulin JSON output)
- Release pipeline trigger now matches `v*` tag format instead of `[0-9]*`

## [0.2.2] - 2026-02-28

### Added
- Coverage badge generated by `cargo-tarpaulin` in CI, hosted in Gitea Generic Package Registry
- CI status, coverage, version, and Rust edition badges in README

## [0.2.1] - 2026-02-28

### Added
- Drone CI pipeline (`ci`) running `cargo fmt --check`, `cargo clippy`, and `cargo test` on every push and pull request

## [0.2.0] - 2026-02-28

### Added
- Unit tests for `cache`, `session`, and `config` modules
- Integration tests for scan, session config, and cache lifecycle

### Changed
- Refactored business logic into `lib.rs` for better testability; `main.rs` is now a thin entrypoint

## [0.1.1] - 2026-02-28

### Fixed
- Removed personal path references from default configuration and examples

## [0.1.0] - 2026-02-28

### Added
- Initial release of tmuxido

