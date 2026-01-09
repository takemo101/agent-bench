# 表示改善機能 詳細設計書インデックス

## メタ情報

| 項目 | 内容 |
|------|------|
| 基本設計書 | [BASIC-CLI-004_visual-enhancement.md](../../basic/BASIC-CLI-004_visual-enhancement.md) |
| 対応要件 | REQ-CLI-004 |
| 作成日 | 2026-01-10 |
| 最終更新日 | 2026-01-10 |

---

## 詳細設計書一覧

### 機能別

| # | 詳細設計書 | 対象機能 | 優先度 | ステータス |
|---|-----------|---------|--------|-----------|
| 1 | [TimeFormatter詳細設計](./time-formatter.md) | F-035 | 高 | 作成済み |
| 2 | [AnimationEngine詳細設計](./animation-engine.md) | F-036 | 高 | 作成済み |
| 3 | [LayoutRenderer詳細設計](./layout-renderer.md) | F-037 | 高 | 作成済み |
| 4 | [TerminalController詳細設計](./terminal-controller.md) | F-037 | 高 | 作成済み |

### 共通設計

| # | 詳細設計書 | 対象 | 優先度 | ステータス |
|---|-----------|------|--------|-----------|
| 1 | [統合テスト仕様書](./共通/test-specification.md) | F-035, F-036, F-037 | 高 | 作成済み |
| 2 | [Issue計画書（REQ-CLI-004）](./共通/issue-plan.md) | タスク分解・依存関係 | 高 | 作成済み |

---

## 既存設計書への影響（更新対象）

本機能の実装に伴い、以下の既存設計書の更新も必要となる。

| 設計書 | パス | 更新内容 |
|--------|------|---------|
| インジケーター表示・色分け詳細設計 | `../display-and-sound-enhancement/ui-indicator.md` | 統合レイアウトへの移行、indicatifテンプレート更新 |
| CLIクライアント詳細設計 | `../pomodoro-timer/cli-client.md` | 表示ロジック更新、アニメーションループ統合 |

---

## フォルダ構成

```
docs/designs/detailed/visual-enhancement/
├── README.md                           # このファイル
├── time-formatter.md                   # TimeFormatter詳細設計（F-035）
├── animation-engine.md                 # AnimationEngine詳細設計（F-036）
├── layout-renderer.md                  # LayoutRenderer詳細設計（F-037）
├── terminal-controller.md              # TerminalController詳細設計（F-037）
└── 共通/
    ├── test-specification.md           # 統合テスト仕様書
    └── issue-plan.md                   # Issue計画書
```

---

## 未解決課題（基本設計から継承）

| ID | 課題 | 期限 | ステータス |
|----|------|------|-----------|
| I-020 | 絵文字幅計算の検証（iTerm2, Terminal.app） | 2026-01-15 | 未着手 |
| I-021 | アニメーションFPS最適値の検証 | 2026-01-15 | 未着手 |
| I-022 | indicatifとの統合方式の検討 | 2026-01-12 | 未着手 |
| I-023 | ターミナル幅の動的取得方法 | 2026-01-12 | 未着手 |
| I-024 | 絵文字非対応ターミナルの検出方法 | 2026-01-13 | 未着手 |
| I-025 | アニメーション更新とタイマー更新の同期方法 | 2026-01-14 | 未着手 |
| I-026 | ちらつき防止の実機検証 | 2026-01-15 | 未着手 |
| I-027 | indicatifとの競合解消 | 2026-01-12 | 未着手 |

---

## 技術的検証項目

| 項目 | 検証内容 | 期限 | ステータス |
|------|---------|------|-----------|
| indicatifとの統合 | カスタムテンプレートで3行レイアウトが実現可能か | 2026-01-12 | 未着手 |
| unicode-width | 絵文字（🏃💨等）の幅が正確に計算できるか | 2026-01-11 | 未着手 |
| ANSIエスケープシーケンス | iTerm2, Terminal.app, Alacrittyでちらつきなく動作するか | 2026-01-15 | 未着手 |
| tokio::time::interval | 200ms間隔のアニメーション更新でドリフトが発生しないか | 2026-01-13 | 未着手 |
| ターミナル幅取得 | `terminal_size`クレートでリアルタイム取得が可能か | 2026-01-12 | 未着手 |
