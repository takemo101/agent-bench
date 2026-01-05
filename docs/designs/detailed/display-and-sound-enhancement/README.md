# 表示とサウンド機能強化 詳細設計書インデックス

## メタ情報

| 項目 | 内容 |
|------|------|
| 基本設計書 | [BASIC-CLI-002_display-and-sound-enhancement.md](../../basic/BASIC-CLI-002_display-and-sound-enhancement.md) |
| 対応要件 | REQ-CLI-002 |
| 作成日 | 2026-01-05 |
| 最終更新日 | 2026-01-05 |

---

## 詳細設計書一覧

### 機能別

| # | 詳細設計書 | 対象機能 | 優先度 | ステータス |
|---|-----------|---------|--------|-----------|
| 1 | [インジケーター表示・色分け詳細設計](./ui-indicator.md) | F-024, F-027 | 高 | 未作成 |
| 2 | [サウンド設定・再生詳細設計](./sound-config.md) | F-025, F-026 | 高 | 未作成 |

### 共通設計

| # | 詳細設計書 | 対象 | 優先度 | ステータス |
|---|-----------|------|--------|-----------|
| 1 | [設定永続化設計（JSON）](./共通/settings-persistence.md) | SoundConfigの保存・読込 | 中 | 未作成 |
| 2 | [テスト項目書（強化機能）](./共通/test-specification.md) | 表示・サウンドの検証 | 高 | 未作成 |
| 3 | [Issue計画書（REQ-CLI-002）](./共通/issue-plan.md) | タスク分解・依存関係 | 高 | 未作成 |

---

## 既存設計書への影響（更新対象）

本機能の実装に伴い、以下の既存設計書の更新も必要となる。

| 設計書 | パス | 更新内容 |
|--------|------|---------|
| CLIクライアント詳細設計 | `../pomodoro-timer/cli-client.md` | サブコマンド追加、表示ロジック更新 |
| サウンド再生詳細設計 | `../pomodoro-timer/sound-playback.md` | rodio v0.21移行、AIFF対応 |
| Daemonサーバー詳細設計 | `../pomodoro-timer/daemon-server.md` | IPCメッセージ拡張、設定管理追加 |

---

## フォルダ構成

```
docs/designs/detailed/display-and-sound-enhancement/
├── README.md                           # このファイル
├── ui-indicator.md                     # インジケーター表示・色分け詳細設計
├── sound-config.md                     # サウンド設定・再生詳細設計
└── 共通/
    ├── settings-persistence.md         # 設定永続化設計
    ├── test-specification.md           # テスト項目書
    └── issue-plan.md                   # Issue計画書
```
