# イベントフック機能 要件定義書

## メタ情報

| 項目 | 内容 |
|------|------|
| ドキュメントID | REQ-CLI-003 |
| バージョン | 1.1.0 |
| ステータス | ドラフト |
| 作成日 | 2026-01-06 |
| 最終更新日 | 2026-01-06 |
| 作成者 | - |
| 承認者 | - |
| 親要件 | REQ-CLI-001 |

---

## 1. 概要

### 1.1 背景

既存のポモドーロタイマーCLI（REQ-CLI-001）では、タイマー完了時にmacOS通知とサウンド再生が行われるが、ユーザー独自の処理を実行する仕組みがない。ユーザーからのフィードバックにより、以下の要望が寄せられた：

1. **外部システム連携**: Slack通知、メール送信、他のツールとの連携を実現したい
2. **カスタム処理**: タイマーイベントに応じて任意のスクリプトを実行したい
3. **柔軟性**: 1つのイベントに対して複数の処理を登録できるようにしたい
4. **コンテキスト情報**: スクリプトにタスク名やフェーズ情報を渡したい

これらの要望に応えるため、イベント駆動型のフック機構を導入し、ユーザーがBashスクリプトを通じてタイマーイベントに独自の処理を追加できるようにする。

### 1.2 目的

本要件は、ポモドーロタイマーCLIにイベントフック機能を追加し、拡張性とカスタマイズ性を向上させることを目的とする：

1. **拡張性の向上**: ユーザーが独自の処理を追加可能にする
2. **外部連携の実現**: Slack、メール、Webhook等との連携を可能にする
3. **柔軟な設定**: 複数のスクリプトを登録し、イベントごとに実行できる
4. **コンテキスト情報の提供**: スクリプトにタスク情報を環境変数で渡す

### 1.3 ゴール

| 指標 | 目標値 | 測定方法 |
|------|--------|----------|
| フック実行遅延 | 500ms以内 | イベント発生からスクリプト実行開始までの時間 |
| スクリプト実行成功率 | 95%以上 | 正常に実行完了したスクリプトの割合 |
| 設定の容易性 | 5分以内で設定完了 | ユーザーテスト |
| エラーハンドリング | すべてのエラーでログ記録 | エラーログの網羅性確認 |

### 1.4 スコープ

#### 対象範囲（Phase 1）

- イベント種別の定義（作業開始、作業終了、休憩開始、休憩終了、一時停止、再開、停止）
- Bashスクリプトの非同期実行機能（タイマーをブロックしない）
- 設定ファイルによるフック登録（`~/.pomodoro/hooks.json`）
- 1つのイベントに対する複数スクリプト登録
- 環境変数によるコンテキスト情報の提供
- スクリプト実行のタイムアウト制御
- エラーハンドリングとログ記録

#### 対象外（Phase 2以降）

- フック実行履歴の記録・閲覧機能
- フックのプレビュー・テスト機能
- GUI設定ツール
- スクリプトのバリデーション機能
- フック実行の優先順位制御
- 条件付きフック実行（特定のタスク名のみ等）
- カスタムイベントの定義

### 1.5 前提条件（スコープ拡張）

本要件は以下の既存設計を拡張する：

| 既存設計書 | 拡張内容 | 影響度 |
|-----------|---------|--------|
| DETAILED-CLI-001-TIMER (タイマーエンジン詳細設計) | イベント通知機構の追加 | 高 |
| BASIC-CLI-001 (基本設計書) | フック実行モジュールの追加 | 中 |

> **重要**: 本要件の実装前に、上記詳細設計書の更新が必要です。

---

## 2. ステークホルダー

### 2.1 ステークホルダー一覧

REQ-CLI-001 セクション2.1を参照。本要件に特有のステークホルダーは追加なし。

### 2.2 ユーザーペルソナ

REQ-CLI-001 セクション2.2を参照。特に以下のペルソナに有用：

- **ペルソナ2（マルチタスク在宅ワーカー）**: Slack通知でチーム連携
- **ペルソナ3（macOSパワーユーザー）**: 高度なカスタマイズと自動化

---

## 3. 機能要件

### 3.1 機能一覧

| ID | 機能名 | 概要 | 優先度 | フェーズ | 親機能 |
|----|--------|------|--------|---------|--------|
| F-028 | イベント定義 | タイマーイベントの種別を定義 | 必須 | Phase 1 | - |
| F-029 | フック登録 | 設定ファイルでフックスクリプトを登録 | 必須 | Phase 1 | - |
| F-030 | スクリプト実行 | Bashスクリプトを実行 | 必須 | Phase 1 | - |
| F-031 | 環境変数提供 | スクリプトにコンテキスト情報を渡す | 必須 | Phase 1 | F-030 |
| F-032 | 複数フック対応 | 1イベントに複数スクリプトを登録 | 重要 | Phase 1 | F-029 |
| F-033 | タイムアウト制御 | スクリプト実行のタイムアウト設定 | 重要 | Phase 1 | F-030 |
| F-034 | エラーハンドリング | スクリプト失敗時の処理 | 必須 | Phase 1 | F-030 |

### 3.2 ユーザーストーリー

#### US-016: Slack通知の実現

- **ユーザー**: マルチタスク在宅ワーカー
- **したいこと**: 作業終了時にSlackで通知を送りたい
- **理由**: チームメンバーに作業状況を共有したい
- **受け入れ基準**:
  - [ ] 作業終了イベントにSlack通知スクリプトを登録できる
  - [ ] スクリプトにタスク名が環境変数で渡される
  - [ ] Slack APIを呼び出すスクリプトが正常に実行される
  - [ ] スクリプト実行失敗時もタイマーは継続する
- **関連機能**: F-028, F-029, F-030, F-031

#### US-017: 複数の通知手段の併用

- **ユーザー**: macOSパワーユーザー
- **したいこと**: 作業終了時にSlack通知とメール送信を同時に行いたい
- **理由**: 複数のチャネルで確実に通知を受け取りたい
- **受け入れ基準**:
  - [ ] 1つのイベントに複数のスクリプトを登録できる
  - [ ] 登録順にスクリプトが実行される
  - [ ] 1つのスクリプトが失敗しても他のスクリプトは実行される
- **関連機能**: F-032, F-034

#### US-018: タスク情報の活用

- **ユーザー**: ターミナル常駐開発者
- **したいこと**: スクリプトで現在のタスク名を取得したい
- **理由**: タスクごとに異なる処理を実行したい
- **受け入れ基準**:
  - [ ] 環境変数 `POMODORO_TASK` でタスク名を取得できる
  - [ ] 環境変数 `POMODORO_PHASE` でフェーズを取得できる
  - [ ] 環境変数 `POMODORO_COUNT` でポモドーロ回数を取得できる
- **関連機能**: F-031

#### US-019: スクリプトのタイムアウト制御

- **ユーザー**: macOSパワーユーザー
- **したいこと**: スクリプトが長時間実行されないようにしたい
- **理由**: スクリプトのバグでタイマーが止まるのを防ぎたい
- **受け入れ基準**:
  - [ ] スクリプトにタイムアウト時間を設定できる
  - [ ] タイムアウト時はスクリプトが強制終了される
  - [ ] タイムアウト発生時はログに記録される
- **関連機能**: F-033, F-034

### 3.3 機能詳細

#### F-028: イベント定義

**概要**: タイマーで発生するイベントの種別を定義する

**イベント種別**:

| イベント名 | 説明 | 発生タイミング | 環境変数 `POMODORO_EVENT` |
|-----------|------|---------------|--------------------------|
| work_start | 作業開始 | 作業タイマー開始時 | `work_start` |
| work_end | 作業終了 | 作業タイマー完了時 | `work_end` |
| break_start | 休憩開始 | 休憩タイマー開始時 | `break_start` |
| break_end | 休憩終了 | 休憩タイマー完了時 | `break_end` |
| long_break_start | 長い休憩開始 | 長い休憩タイマー開始時 | `long_break_start` |
| long_break_end | 長い休憩終了 | 長い休憩タイマー完了時 | `long_break_end` |
| pause | 一時停止 | タイマー一時停止時 | `pause` |
| resume | 再開 | タイマー再開時 | `resume` |
| stop | 停止 | タイマー停止時 | `stop` |

**処理概要**:
1. タイマーエンジンでイベント発生を検知
2. イベント種別を判定
3. フック実行モジュールにイベントを通知
4. 登録されたフックスクリプトを実行

**ビジネスルール**:
- BR-070: イベント名は小文字とアンダースコアのみ使用
- BR-071: イベントは非同期的に処理される（タイマーをブロックしない）
- BR-072: イベント処理中もタイマーは継続する（UX最優先）

**制約事項**:
- Phase 1では上記9種類のイベントのみサポート
- カスタムイベントの定義はPhase 2

---

#### F-029: フック登録

**概要**: 設定ファイルでイベントに対するフックスクリプトを登録する

**設定ファイル**: `~/.pomodoro/hooks.json`

**設定ファイル形式**:
```json
{
  "hooks": {
    "work_end": [
      {
        "name": "Slack通知",
        "script": "/Users/user/.pomodoro/scripts/notify-slack.sh",
        "enabled": true,
        "timeout": 10
      },
      {
        "name": "メール送信",
        "script": "/Users/user/.pomodoro/scripts/send-email.sh",
        "enabled": true,
        "timeout": 15
      }
    ],
    "break_start": [
      {
        "name": "音楽再生",
        "script": "/Users/user/.pomodoro/scripts/play-music.sh",
        "enabled": false,
        "timeout": 5
      }
    ]
  },
  "global_timeout": 30
}
```

**設定項目**:

| 項目 | 型 | 必須 | 説明 | デフォルト値 |
|------|-----|------|------|-------------|
| `hooks` | Object | ✓ | イベントごとのフック定義 | - |
| `hooks.<event>` | Array | - | イベントに登録するフックのリスト | `[]` |
| `hooks.<event>[].name` | String | ✓ | フックの名前（識別用） | - |
| `hooks.<event>[].script` | String | ✓ | スクリプトの絶対パス | - |
| `hooks.<event>[].enabled` | Boolean | - | フックの有効/無効 | `true` |
| `hooks.<event>[].timeout` | Number | - | タイムアウト時間（秒） | `global_timeout` |
| `global_timeout` | Number | - | グローバルタイムアウト（秒） | `30` |

**処理概要**:
1. 起動時に設定ファイルを読み込み
2. JSON形式をパースして検証
3. フック定義をメモリに保持
4. イベント発生時に該当するフックを取得

**ビジネスルール**:
- BR-073: 設定ファイルが存在しない場合はフック機能を無効化
- BR-074: 設定ファイルが破損している場合は警告を出力し、フック機能を無効化
- BR-075: スクリプトパスは絶対パスのみ許可（セキュリティ上の理由）
- BR-076: `enabled: false` のフックは実行しない
- BR-077: フックは登録順に実行される

**制約事項**:
- スクリプトパスは実行可能ファイルである必要がある
- 設定ファイルのサイズは1MB以下
- 1イベントあたり最大10個のフックまで登録可能

---

#### F-030: スクリプト実行

**概要**: 登録されたBashスクリプトを実行する

**入力**:
- イベント種別
- フック定義（スクリプトパス、タイムアウト等）
- コンテキスト情報（タスク名、フェーズ等）

**出力**:
- スクリプトの標準出力（ログに記録）
- スクリプトの標準エラー出力（ログに記録）
- 実行結果（成功/失敗/タイムアウト）

**処理概要**:
1. スクリプトファイルの存在と実行権限を確認
2. 環境変数を設定（F-031参照）
3. `tokio::process::Command` を使用してスクリプトを実行
4. タイムアウト時間を監視
5. スクリプトの終了コードを確認
6. 標準出力・標準エラー出力をログに記録

**実行コマンド例**:
```bash
# 内部的に以下のように実行される
/bin/bash /Users/user/.pomodoro/scripts/notify-slack.sh
```

**ビジネスルール**:
- BR-078: スクリプトは `/bin/bash` で実行される
- BR-079: スクリプトの終了コード0を成功、それ以外を失敗とする
- BR-080: タイムアウト時はプロセスを強制終了（SIGTERM → SIGKILL）
- BR-081: スクリプトは非同期実行され、タイマーをブロックしない（UX最優先）
- BR-082: 標準出力・標準エラー出力は最大10KBまでログに記録

**制約事項**:
- スクリプトは非同期実行（fire-and-forget方式、結果はログで確認）
- スクリプトの実行権限はユーザーと同じ
- スクリプトの作業ディレクトリは `~/.pomodoro/`

---

#### F-031: 環境変数提供

**概要**: スクリプトにコンテキスト情報を環境変数で提供する

**提供する環境変数**:

| 環境変数名 | 説明 | 値の例 | 必須 |
|-----------|------|--------|------|
| `POMODORO_EVENT` | イベント種別 | `work_end`, `break_start` | ✓ |
| `POMODORO_PHASE` | 現在のフェーズ | `working`, `breaking`, `long_breaking`, `stopped`, `paused` | ✓ |
| `POMODORO_TASK` | タスク名 | `API実装`, `ドキュメント作成` | - |
| `POMODORO_COUNT` | 現在のポモドーロ回数 | `1`, `2`, `3`, `4` | ✓ |
| `POMODORO_REMAINING` | 残り時間（秒） | `1500`, `300` | ✓ |
| `POMODORO_TOTAL` | 総時間（秒） | `1500`, `300` | ✓ |
| `POMODORO_WORK_MINUTES` | 作業時間設定（分） | `25`, `30` | ✓ |
| `POMODORO_BREAK_MINUTES` | 休憩時間設定（分） | `5`, `10` | ✓ |
| `POMODORO_LONG_BREAK_MINUTES` | 長い休憩時間設定（分） | `15`, `20` | ✓ |
| `POMODORO_AUTO_CYCLE` | 自動サイクル有効化 | `true`, `false` | ✓ |
| `POMODORO_FOCUS_MODE` | フォーカスモード有効化 | `true`, `false` | ✓ |

**スクリプト例**:
```bash
#!/bin/bash
# notify-slack.sh

# 環境変数を取得
EVENT=$POMODORO_EVENT
TASK=$POMODORO_TASK
COUNT=$POMODORO_COUNT

# Slack通知を送信
if [ "$EVENT" = "work_end" ]; then
  MESSAGE="🍅 ポモドーロ #${COUNT} 完了: ${TASK}"
  curl -X POST https://hooks.slack.com/services/YOUR/WEBHOOK/URL \
    -H 'Content-Type: application/json' \
    -d "{\"text\": \"${MESSAGE}\"}"
fi
```

**ビジネスルール**:
- BR-083: タスク名が未設定の場合、`POMODORO_TASK` は空文字列
- BR-084: 環境変数はすべてUTF-8エンコーディング
- BR-085: 環境変数名は `POMODORO_` プレフィックスで統一

**制約事項**:
- 環境変数の値は最大1024文字まで
- 環境変数の総サイズは16KBまで

---

#### F-032: 複数フック対応

**概要**: 1つのイベントに対して複数のスクリプトを登録し、並列で非同期実行する

**処理概要**:
1. イベント発生時、該当するフックのリストを取得
2. `enabled: true` のフックのみをフィルタリング
3. すべてのスクリプトを並列で非同期実行（tokio::spawn）
4. 各スクリプトの実行結果をログに記録
5. すべてのフック実行はタイマーをブロックしない

**実行例**:
```
[INFO] イベント 'work_end' のフックを非同期実行開始 (2件)
[INFO] フック 'Slack通知' を非同期実行中...
[INFO] フック 'メール送信' を非同期実行中...
[INFO] フック 'Slack通知' が成功しました (実行時間: 1.2秒)
[ERROR] フック 'メール送信' が失敗しました (終了コード: 1)
```

**ビジネスルール**:
- BR-086: フックは並列で非同期実行される（実行順序は保証しない）
- BR-087: 1つのフックの失敗は他のフックに影響しない（独立実行）
- BR-088: （削除：非同期実行では不要）
- BR-089: すべてのフックの実行結果をログに記録

**制約事項**:
- 1イベントあたり最大10個のフックまで
- 並列実行数の制限はなし（10個まで同時実行可能）

---

#### F-033: タイムアウト制御

**概要**: スクリプト実行のタイムアウト時間を設定し、長時間実行を防ぐ

**タイムアウト設定の優先順位**:
1. フック個別の `timeout` 設定
2. グローバルの `global_timeout` 設定
3. デフォルト値（30秒）

**処理概要**:
1. スクリプト実行開始時にタイマーを開始
2. タイムアウト時間経過後、プロセスにSIGTERMを送信
3. 5秒待機してもプロセスが終了しない場合、SIGKILLを送信
4. タイムアウト発生をログに記録

**ビジネスルール**:
- BR-090: タイムアウト時間は1-300秒の範囲で設定可能
- BR-091: タイムアウト発生時はエラーとして扱う
- BR-092: タイムアウト時はプロセスを強制終了

**制約事項**:
- タイムアウト精度は±1秒程度
- タイムアウト時のクリーンアップはスクリプト側で実装する必要がある

---

#### F-034: エラーハンドリング

**概要**: スクリプト実行失敗時の処理とログ記録

**エラー種別**:

| エラー種別 | 説明 | 対処 |
|-----------|------|------|
| スクリプト不存在 | スクリプトファイルが見つからない | ログに記録、次のフックを実行 |
| 実行権限なし | スクリプトに実行権限がない | ログに記録、次のフックを実行 |
| 実行失敗 | スクリプトが非0の終了コードで終了 | ログに記録、`continue_on_error` に従う |
| タイムアウト | スクリプトがタイムアウト時間内に終了しない | プロセス強制終了、ログに記録 |
| 設定エラー | 設定ファイルが破損している | フック機能を無効化、警告を出力 |

**ログ出力例**:
```
[2026-01-06 10:30:00] [INFO] イベント 'work_end' のフックを実行開始
[2026-01-06 10:30:00] [INFO] フック 'Slack通知' を実行中...
[2026-01-06 10:30:01] [INFO] フック 'Slack通知' が成功しました (実行時間: 1.2秒)
[2026-01-06 10:30:01] [INFO] フック 'メール送信' を実行中...
[2026-01-06 10:30:11] [ERROR] フック 'メール送信' がタイムアウトしました (タイムアウト: 10秒)
[2026-01-06 10:30:11] [INFO] イベント 'work_end' のフック実行完了 (成功: 1, 失敗: 1)
```

**ビジネスルール**:
- BR-093: すべてのエラーはログファイル（`~/.pomodoro/logs/hooks.log`）に記録
- BR-094: エラー発生時もタイマーは継続する
- BR-095: 設定ファイルエラー時はフック機能全体を無効化
- BR-096: スクリプトエラー時は該当フックのみスキップ

**制約事項**:
- ログファイルは最大10MBまで（ローテーション）
- エラー通知機能はPhase 2

---

## 4. 非機能要件

### 4.1 性能要件

| ID | 要件 | 目標値 | 測定方法 |
|----|------|--------|----------|
| NFR-P-020 | フック実行遅延 | 500ms以内 | イベント発生からスクリプト実行開始までの時間 |
| NFR-P-021 | 設定ファイル読み込み | 100ms以内 | 起動時の設定ファイル読み込み時間 |
| NFR-P-022 | 追加メモリ使用量 | 5MB以内 | フック機能追加によるメモリ増加量 |

### 4.2 セキュリティ要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-S-010 | スクリプトパス検証 | 絶対パスのみ許可、相対パスは拒否 |
| NFR-S-011 | 実行権限確認 | スクリプトファイルの実行権限を確認 |
| NFR-S-012 | 環境変数サニタイズ | 環境変数の値をエスケープ処理 |
| NFR-S-013 | ログファイル権限 | ログファイルは0600（所有者のみ読み書き可） |
| NFR-S-014 | 設定ファイル権限 | 設定ファイルは0600推奨（警告のみ） |

### 4.3 可用性要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-A-010 | フック失敗時の継続性 | フック失敗時もタイマーは継続する |
| NFR-A-011 | 設定エラー時の動作 | 設定ファイルエラー時はフック機能を無効化し、タイマーは正常動作 |
| NFR-A-012 | ログローテーション | ログファイルが10MBを超えたら自動ローテーション |

### 4.4 保守性要件

| ID | 要件 | 詳細 |
|----|------|------|
| NFR-M-010 | 詳細ログ | フック実行の開始・終了・エラーをすべてログに記録 |
| NFR-M-011 | デバッグモード | `--verbose` フラグでスクリプトの標準出力・標準エラー出力を表示 |
| NFR-M-012 | 設定検証 | 起動時に設定ファイルの妥当性を検証 |

---

## 5. 制約条件

### 5.1 技術的制約

| 制約 | 詳細 | 理由 |
|------|------|------|
| スクリプト言語 | Bashのみサポート | シンプルさとmacOS標準環境 |
| 実行方式 | 非同期実行（fire-and-forget） | UX最優先、タイマーをブロックしない |
| 設定ファイル形式 | JSON | 既存の設定ファイル形式と統一 |
| ログライブラリ | tracing（既存依存関係） | 既存実装との整合性 |
| プロセス実行 | tokio::spawn + tokio::process::Command | 非同期ランタイムとの統合 |

### 5.2 ビジネス制約

REQ-CLI-001 セクション5.2を参照。

### 5.3 セキュリティ制約

| 制約 | 詳細 | 理由 |
|------|------|------|
| スクリプトパス | 絶対パスのみ許可 | パストラバーサル攻撃の防止 |
| 実行権限 | ユーザー権限で実行 | 権限昇格の防止 |
| 環境変数 | サニタイズ処理必須 | インジェクション攻撃の防止 |
| ログファイル | 所有者のみ読み書き可 | 情報漏洩の防止 |

---

## 6. 外部インターフェース

### 6.1 設定ファイル

**ファイルパス**: `~/.pomodoro/hooks.json`

**ファイル形式**: JSON（UTF-8エンコーディング）

**スキーマ定義**:
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "hooks": {
      "type": "object",
      "patternProperties": {
        "^(work_start|work_end|break_start|break_end|long_break_start|long_break_end|pause|resume|stop)$": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "name": { "type": "string", "minLength": 1, "maxLength": 100 },
              "script": { "type": "string", "pattern": "^/" },
              "enabled": { "type": "boolean" },
              "timeout": { "type": "number", "minimum": 1, "maximum": 300 }
            },
            "required": ["name", "script"]
          },
          "maxItems": 10
        }
      }
    },
    "global_timeout": { "type": "number", "minimum": 1, "maximum": 300 }
  },
  "required": ["hooks"]
}
```

### 6.2 ログファイル

**ファイルパス**: `~/.pomodoro/logs/hooks.log`

**ファイル形式**: テキスト（UTF-8エンコーディング）

**ログレベル**:
- `INFO`: 通常の実行ログ
- `WARN`: 警告（設定ファイルの権限等）
- `ERROR`: エラー（スクリプト失敗、タイムアウト等）

### 6.3 スクリプトインターフェース

**実行方法**: `/bin/bash <スクリプトパス>`

**環境変数**: F-031参照

**終了コード**:
- `0`: 成功
- `1-255`: 失敗

**標準出力・標準エラー出力**: ログファイルに記録（最大10KB）

---

## 7. 前提条件と依存関係

### 7.1 前提条件

- REQ-CLI-001の基本機能が実装済み
- `/bin/bash` が利用可能
- ユーザーが `~/.pomodoro/` ディレクトリへの書き込み権限を持つ
- スクリプトファイルに実行権限が付与されている

### 7.2 依存関係

| 依存先 | 内容 | 影響 |
|--------|------|------|
| REQ-CLI-001 F-001 | タイマーイベント | フック実行のトリガー |
| tokio クレート | 非同期プロセス実行 | スクリプト実行に必須 |
| serde_json クレート | JSON解析 | 設定ファイル読み込みに必須 |
| tracing クレート | ログ記録 | エラーハンドリングに必須 |

---

## 8. リスクと課題

### 8.1 リスク一覧

| ID | リスク | 影響度 | 発生確率 | 対策 |
|----|--------|--------|---------|------|
| R-020 | スクリプトの無限ループ | 高 | 中 | タイムアウト制御の実装 |
| R-021 | スクリプトのセキュリティ脆弱性 | 高 | 中 | パス検証、権限確認の徹底 |
| R-022 | 設定ファイルの破損 | 中 | 低 | バリデーション、フォールバック処理 |
| R-023 | 非同期タスクの蓄積 | 中 | 低 | タイムアウト制御、最大同時実行数の制限 |
| R-024 | ログファイルの肥大化 | 低 | 中 | ログローテーション、サイズ制限 |

### 8.2 未解決課題

| ID | 課題 | 担当 | 期限 | ステータス |
|----|------|------|------|----------|
| I-020 | スクリプトのバリデーション機能の設計 | - | Phase 2 | 未着手 |
| I-021 | フック実行履歴の記録方式の検討 | - | Phase 2 | 未着手 |
| I-022 | 非同期タスクの最大同時実行数制限 | - | Phase 2 | 未着手 |
| I-023 | タイマーエンジン詳細設計書の更新 | - | 2026-01-15 | 未着手 |

---

## 9. 用語集

| 用語 | 定義 |
|------|------|
| フック | 特定のイベント発生時に実行されるスクリプト |
| イベント | タイマーで発生する状態変化（作業開始、作業終了等） |
| コンテキスト情報 | スクリプトに渡されるタスク名やフェーズ等の情報 |
| タイムアウト | スクリプトの最大実行時間 |
| 非同期実行 | スクリプトの実行完了を待たずに次の処理に進む方式（本要件のデフォルト） |
| fire-and-forget | スクリプトを起動後、完了を待たずに即座に制御を戻す実行パターン |

---

## 10. 既存設計書への影響

本要件の実装に伴い、以下の設計書の更新が必要：

### 10.1 更新が必要な設計書

| 設計書 | パス | 更新内容 | 優先度 |
|--------|------|---------|--------|
| タイマーエンジン詳細設計 | `docs/designs/detailed/pomodoro-timer/timer-engine.md` | イベント通知機構の追加 | 高 |
| 基本設計書 | `docs/designs/basic/BASIC-CLI-001_pomodoro-timer.md` | フック実行モジュールの追加 | 中 |

### 10.2 タイマーエンジン詳細設計への追加内容

#### HookEvent 列挙型（新規）

```rust
/// フックイベント種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookEvent {
    WorkStart,
    WorkEnd,
    BreakStart,
    BreakEnd,
    LongBreakStart,
    LongBreakEnd,
    Pause,
    Resume,
    Stop,
}

impl HookEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookEvent::WorkStart => "work_start",
            HookEvent::WorkEnd => "work_end",
            HookEvent::BreakStart => "break_start",
            HookEvent::BreakEnd => "break_end",
            HookEvent::LongBreakStart => "long_break_start",
            HookEvent::LongBreakEnd => "long_break_end",
            HookEvent::Pause => "pause",
            HookEvent::Resume => "resume",
            HookEvent::Stop => "stop",
        }
    }
}
```

#### HookContext 構造体（新規）

```rust
/// フックスクリプトに渡すコンテキスト情報
#[derive(Debug, Clone)]
pub struct HookContext {
    pub event: HookEvent,
    pub phase: TimerPhase,
    pub task: Option<String>,
    pub count: u32,
    pub remaining: u32,
    pub total: u32,
    pub config: PomodoroConfig,
}

impl HookContext {
    /// 環境変数のマップを生成
    pub fn to_env_vars(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();
        env.insert("POMODORO_EVENT".to_string(), self.event.as_str().to_string());
        env.insert("POMODORO_PHASE".to_string(), self.phase.as_str().to_string());
        if let Some(task) = &self.task {
            env.insert("POMODORO_TASK".to_string(), task.clone());
        }
        env.insert("POMODORO_COUNT".to_string(), self.count.to_string());
        env.insert("POMODORO_REMAINING".to_string(), self.remaining.to_string());
        env.insert("POMODORO_TOTAL".to_string(), self.total.to_string());
        env.insert("POMODORO_WORK_MINUTES".to_string(), self.config.work_minutes.to_string());
        env.insert("POMODORO_BREAK_MINUTES".to_string(), self.config.break_minutes.to_string());
        env.insert("POMODORO_LONG_BREAK_MINUTES".to_string(), self.config.long_break_minutes.to_string());
        env.insert("POMODORO_AUTO_CYCLE".to_string(), self.config.auto_cycle.to_string());
        env.insert("POMODORO_FOCUS_MODE".to_string(), self.config.focus_mode.to_string());
        env
    }
}
```

#### TimerEngine への追加

```rust
impl TimerEngine {
    /// フックイベントを発火（非同期・ノンブロッキング）
    fn fire_hook(&self, event: HookEvent) {
        let context = HookContext {
            event,
            phase: self.state.phase,
            task: self.state.task.clone(),
            count: self.state.pomodoro_count,
            remaining: self.state.remaining_seconds,
            total: self.state.total_duration(),
            config: self.state.config.clone(),
        };
        
        // フック実行モジュールに非同期で通知（fire-and-forget）
        // タイマーをブロックしない
        let executor = self.hook_executor.clone();
        tokio::spawn(async move {
            if let Err(e) = executor.execute(context).await {
                tracing::error!("フック実行エラー: {:?}", e);
            }
        });
    }
}
```

---

## 11. 変更履歴

| バージョン | 日付 | 変更内容 | 作成者 |
|-----------|------|----------|--------|
| 1.0.0 | 2026-01-06 | 初版作成 | - |
| 1.1.0 | 2026-01-06 | フック実行を非同期（fire-and-forget）に変更：UX最優先でタイマーをブロックしない設計に修正 | - |

---

## 付録A: コマンドリファレンス（追加分）

### フック管理コマンド（Phase 2で検討）

```bash
# フック設定の検証
pomodoro hooks validate

# フック一覧の表示
pomodoro hooks list

# フックのテスト実行
pomodoro hooks test <event>

# フックの有効化/無効化
pomodoro hooks enable <event> <name>
pomodoro hooks disable <event> <name>
```

---

## 付録B: エラーコード一覧（追加分）

| コード | メッセージ | 原因 | 対処法 |
|--------|-----------|------|--------|
| E030 | フック設定ファイルが見つかりません | 設定ファイルが存在しない | 設定ファイルを作成 |
| E031 | フック設定ファイルの解析に失敗しました | JSON形式が不正 | 設定ファイルの構文を確認 |
| E032 | スクリプトファイルが見つかりません | スクリプトパスが不正 | スクリプトパスを確認 |
| E033 | スクリプトに実行権限がありません | 実行権限がない | `chmod +x` で実行権限を付与 |
| E034 | スクリプトの実行に失敗しました | スクリプトエラー | スクリプトのログを確認 |
| E035 | スクリプトがタイムアウトしました | タイムアウト時間超過 | タイムアウト時間を延長 |

---

## 付録C: スクリプトサンプル

### C.1 Slack通知スクリプト

```bash
#!/bin/bash
# notify-slack.sh

WEBHOOK_URL="https://hooks.slack.com/services/YOUR/WEBHOOK/URL"

case "$POMODORO_EVENT" in
  work_end)
    MESSAGE="🍅 ポモドーロ #${POMODORO_COUNT} 完了: ${POMODORO_TASK}"
    ;;
  break_end)
    MESSAGE="☕ 休憩終了。作業を再開しましょう！"
    ;;
  *)
    exit 0
    ;;
esac

curl -X POST "$WEBHOOK_URL" \
  -H 'Content-Type: application/json' \
  -d "{\"text\": \"${MESSAGE}\"}" \
  --max-time 10 \
  --silent \
  --show-error
```

### C.2 メール送信スクリプト

```bash
#!/bin/bash
# send-email.sh

TO="user@example.com"
SUBJECT="ポモドーロタイマー通知"

case "$POMODORO_EVENT" in
  work_end)
    BODY="ポモドーロ #${POMODORO_COUNT} が完了しました。\nタスク: ${POMODORO_TASK}"
    ;;
  long_break_end)
    BODY="長い休憩が終了しました。作業を再開しましょう！"
    ;;
  *)
    exit 0
    ;;
esac

echo -e "$BODY" | mail -s "$SUBJECT" "$TO"
```

### C.3 音楽再生スクリプト

```bash
#!/bin/bash
# play-music.sh

case "$POMODORO_EVENT" in
  break_start)
    # 休憩開始時に音楽を再生
    osascript -e 'tell application "Music" to play playlist "Relax"'
    ;;
  break_end)
    # 休憩終了時に音楽を停止
    osascript -e 'tell application "Music" to pause'
    ;;
esac
```

### C.4 統計記録スクリプト

```bash
#!/bin/bash
# record-stats.sh

LOG_FILE="$HOME/.pomodoro/stats.csv"

# CSVヘッダーがなければ作成
if [ ! -f "$LOG_FILE" ]; then
  echo "timestamp,event,task,count" > "$LOG_FILE"
fi

# イベントを記録
if [ "$POMODORO_EVENT" = "work_end" ]; then
  TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")
  echo "$TIMESTAMP,$POMODORO_EVENT,$POMODORO_TASK,$POMODORO_COUNT" >> "$LOG_FILE"
fi
```

---

**要件定義書の作成完了**

このドキュメントは、プロジェクトの進行に伴い更新される可能性があります。変更がある場合は、変更履歴セクションに記録してください。
