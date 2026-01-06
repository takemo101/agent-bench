# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-01-06

### Added
- **インジケーター表示**: 作業/休憩フェーズに応じた色分けプログレスバーを追加 (#77, #83)
- **サウンド設定CLI**: `pomodoro config sound` コマンドでサウンドをmacOSシステムサウンドから選択可能に (#75, #76, #79, #80, #81)
- **設定ファイル保存**: サウンド設定の永続化をサポート (#79)

### Fixed
- **statusコマンドの自動更新**: `pomodoro status` がリアルタイムで残り時間を更新するよう修正 (#85, #86)
- **デーモンカウントダウン**: タイマーの残り時間が正しくカウントダウンされるよう修正 (#67, #68)
- **サウンド音量**: 埋め込みサウンドの音量計算を修正 (#71, #72)
- **macOSシステムサウンド再生**: `.aiff` 形式のサウンドファイルが正しくデコードされるよう修正 (#87, #88)
- **起動パラメータ**: startコマンドの時間設定オプションがデーモンに正しく渡されるよう修正 (#66)
- **通知クラッシュ防止**: バンドルコンテキストなしでの通知センターアクセス時のクラッシュを防止 (#63)
- **埋め込みサウンドデコード**: 埋め込みサウンドのデコードエラーを修正 (#70)

### Changed
- **ドキュメント更新**: README.mdとUSAGE.mdを新機能に合わせて更新 (#78, #84)

## [0.1.0] - 2026-01-04

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
