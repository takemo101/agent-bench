# 技術調査レポート: confy / config-rs (設定永続化)

| 項目 | 内容 |
|------|------|
| 調査日 | 2026-01-05 |
| 調査深度 | standard |
| 対象バージョン | confy v2.0.0, config v0.15.19 |
| 現行バージョン | 新規導入 (現在は serde_json 直接操作) |

## エグゼクティブサマリー

CLIツールの設定永続化において、`confy` は「設定ファイルパスの自動解決」と「デフォルト値の自動生成」に特化した、CLIツールに最適なライブラリである。macOSにおいては `~/Library/Application Support/` への保存をサポートする `Native` 戦略が利用可能。一方、`config-rs` や `figment` は階層的な設定（ファイル + 環境変数 + CLI引数）が必要な場合に適している。本プロジェクトの Phase 1 では、シンプルさと macOS ネイティブな体験を重視し、`confy` の採用を推奨する。

## バージョン情報

| ライブラリ | 最新バージョン | リリース日 | 特徴 |
|-----------|--------------|-----------|------|
| confy | v2.0.0 | 2025-10 | ゼロボイラープレート。CLIに最適。 |
| config-rs | v0.15.19 | 2025-xx | 階層的設定。多機能。 |
| figment | v0.10.19 | 2025-xx | 高度なプロバイダーモデル。 |

## Breaking Changes (confy v2.0)

### 🚨 高影響

| 変更内容 | 影響範囲 | 移行方法 |
|---------|---------|---------|
| `Strategy` の導入 | 保存パスの決定 | `confy::load` や `store` 時に `Strategy::Native` を選択することで macOS 標準パスに対応。 |

## 新機能・改善点

| 機能 | 概要 | 活用シーン |
|------|------|----------|
| macOS Native Strategy | `~/Library/Application Support/` への保存 | macOS ネイティブなアプリ体験 |
| TOML/YAML/JSON 対応 | 複数のシリアライズ形式をサポート | ユーザー編集のしやすさで選択可能 |

## 設計への影響

### 必須対応

- [ ] 設定ファイルの保存場所として `~/Library/Application Support/pomodoro/` を使用するように `confy` を設定。
- [ ] `PomodoroConfig` に `Default` トレイトを実装し、初回起動時の自動生成に対応。

### 推奨対応

- [ ] `serde_json` の直接操作を `confy` の高レベル API に置き換え、エラーハンドリングとパス解決を簡素化。

## 参考リンク

- [confy Documentation](https://docs.rs/confy/latest/confy/)
- [config-rs GitHub](https://github.com/mehcode/config-rs)
