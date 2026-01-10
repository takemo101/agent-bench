# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2026-01-10

### Added
- **ビジュアル強化機能**: ステータス表示を全面刷新 (#121)
  - `TimeFormatter`: 残り時間の見やすいフォーマット表示 (#122)
  - `AnimationEngine`: フェーズ別アニメーション（作業中🏃💨、休憩中🧘、長期休憩😴💤） (#123)
  - `TerminalController`: ANSI制御によるちらつき防止レンダリング (#124)
  - `LayoutRenderer`: 3行レイアウトによる統一表示 (#125)
  - `EnhancedDisplayState`: 全コンポーネントの統合 (#126, #127)
  - `main.rs` への統合: 200ms更新間隔でスムーズなアニメーション (#134)

### Fixed
- **作業アニメーション方向修正**: 走る人の絵文字が正しい方向に移動するよう修正 (#136, #137)
- **休憩アニメーション改善**: 波模様が滑らかに左右に揺れるよう修正 (#138, #139)
- **休憩アニメーション呼吸効果**: 左右の波が対称的に動く呼吸アニメーションに改善 (#140, #141)

## [0.4.0] - 2026-01-08

### Added
- **ビジュアル強化基盤**: Epic #121 の基盤コンポーネント実装
  - 詳細設計ドキュメント追加

### Changed
- **ワークフロー改善**: 開発ワークフローの最適化 (v3.16.0 - v3.21.0)

## [0.3.0] - 2026-01-07

### Added
- **イベントフック機能**: タイマーイベント発生時にカスタムBashスクリプトを非同期実行 (#91)
  - 9種類のイベントタイプをサポート: `work_start`, `work_end`, `break_start`, `break_end`, `long_break_start`, `long_break_end`, `pause`, `resume`, `stop`
  - `~/.pomodoro/hooks.json` による設定管理
  - 1イベントあたり最大10個のフックを登録可能
  - 11種類の環境変数でコンテキスト情報を提供
  - タイムアウト制御（1〜300秒）
  - fire-and-forget 方式の非同期実行（タイマーをブロックしない）
- **サンプルスクリプト**: イベントフック用サンプルスクリプト4種を追加 (#107)
  - Slack通知 (`examples/hooks/slack-notify.sh`)
  - デスクトップ通知 (`examples/hooks/desktop-notify.sh`)
  - 統計記録 (`examples/hooks/record-stats.sh`)
  - BGM制御 (`examples/hooks/bgm-control.sh`)

### Changed
- **ドキュメント更新**: USAGE.md, README.md にイベントフック機能のドキュメントを追加 (#105, #106)

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
