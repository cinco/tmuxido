# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.2.4] - 2026-03-01

### Fixed
- Cálculo do percentual de cobertura no CI (campo correto do JSON do tarpaulin)
- Pipeline `release` restrito a tags com formato de versão (`[0-9]*`)

## [0.2.2] - 2026-02-28

### Added
- Coverage badge gerado por `cargo-tarpaulin` no CI e hospedado no Gitea Generic Package Registry
- CI status, coverage, version e Rust edition badges no README

## [0.2.1] - 2026-02-28

### Added
- Drone CI pipeline (`ci`) that runs `cargo fmt --check`, `cargo clippy`, and `cargo test` on every push and pull request

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
