# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial implementation of Pomodoro Timer CLI
- Core timer engine with state machine (Working, Breaking, Stopped)
- IPC server/client architecture using Unix Domain Sockets
- CLI commands: `start`, `pause`, `resume`, `stop`, `status`
- Native macOS Notification Center integration
- Menu bar icon with real-time countdown
- Sound playback on timer completion (System sound & embedded)
- Focus Mode integration (via Shortcuts.app)
- LaunchAgent support for auto-start on login
- Shell completion generation (bash, zsh, fish)
- E2E and Performance tests
