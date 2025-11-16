# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial implementation of commit wizard
- Interactive TUI for commit group management
- Automatic file grouping by type and scope
- External editor integration
- **AI-powered commit message generation using GitHub Copilot API**
- **`--ai` / `--copilot` CLI flag to enable AI features**
- **`a` keyboard shortcut in TUI to generate commit messages**
- Ticket extraction from branch names
- Security hardening (path validation, command injection prevention)
- Comprehensive documentation
- Library pattern with `lib.rs` for external usage
- Integration tests in `tests/` directory (66 tests total)

### Changed

- N/A

### Deprecated

- N/A

### Removed

- N/A

### Fixed

- N/A

### Security

- Path traversal prevention
- Command injection prevention in editor calls
- Timeout protection for external processes

## [0.1.0] - 2025-11-16

### Added

- Initial release
- Basic TUI functionality
- Git integration via libgit2
- Conventional commit support
