# コマンドリファレンス

`pomodoro` コマンドの基本的な使用方法を説明します。

## コマンド一覧

### `start`
タイマーを開始します。

```bash
pomodoro start [OPTIONS]
```

**オプション:**
- `--task <NAME>`: タスク名を指定します（通知やログに表示されます）。
- `--duration <MINUTES>`: 作業時間を分単位で指定します（デフォルト: 25分）。

**例:**
```bash
# 基本的な開始
pomodoro start

# タスク名を指定して開始
pomodoro start --task "メール返信"

# 50分作業で開始
pomodoro start --duration 50
```

### `pause`
実行中のタイマーを一時停止します。

```bash
pomodoro pause
```

### `resume`
一時停止中のタイマーを再開します。

```bash
pomodoro resume
```

### `stop`
タイマーを停止し、初期状態（0分）に戻します。

```bash
pomodoro stop
```

### `status`
現在のタイマーの状態を表示します。

```bash
pomodoro status
```

**出力例:**
```text
🍅 ポモドーロ #1 - 作業中
  ████████████████████░░░░░░░░░░  15:30 / 25:00 (62%)

  タスク: ドキュメント作成
```

※ 作業中は赤/オレンジ、休憩中は緑/青、一時停止中は黄色で色分け表示されます。

### `install`
LaunchAgentを使用して、ログイン時にデーモンを自動起動するように設定します。

```bash
pomodoro install
```

### `uninstall`
自動起動設定を解除します。

```bash
pomodoro uninstall
```

### `completions`
シェル補完スクリプトを生成します。

```bash
pomodoro completions <SHELL>
```

**対応シェル:**
- bash
- zsh
- fish

### `sounds`
利用可能なmacOSシステムサウンドの一覧を表示します。

```bash
pomodoro sounds
```

**出力例:**
```text
利用可能なサウンド:
  - Basso
  - Blow
  - Funk
  - Glass
  - Hero
  - Ping
  - Pop
  ...
```

### `config`
サウンド設定を確認・変更します。

```bash
# 現在の設定を表示
pomodoro config --show

# 作業終了サウンドを "Pop" に変更
pomodoro config --work-sound Pop

# 休憩終了サウンドを "Basso" に変更
pomodoro config --break-sound Basso
```

## イベントフック

タイマーイベント発生時にカスタムスクリプトを実行できます。Slack通知、統計記録、BGM制御など、自由に拡張可能です。

### 対応イベント

| イベント名 | 説明 |
|------------|------|
| `work_start` | 作業セッション開始時 |
| `work_end` | 作業セッション終了時 |
| `break_start` | 短い休憩開始時 |
| `break_end` | 短い休憩終了時 |
| `long_break_start` | 長い休憩開始時 |
| `long_break_end` | 長い休憩終了時 |
| `pause` | タイマー一時停止時 |
| `resume` | タイマー再開時 |
| `stop` | タイマー停止時 |

### 設定ファイル

`~/.pomodoro/hooks.json` にフック定義を記述します。

```json
{
  "version": "1.0",
  "hooks": [
    {
      "name": "slack-notify",
      "event": "work_end",
      "script": "~/.pomodoro/scripts/slack-notify.sh",
      "timeout_secs": 30,
      "enabled": true
    },
    {
      "name": "log-stats",
      "event": "work_end",
      "script": "~/.pomodoro/scripts/log-stats.sh",
      "timeout_secs": 10,
      "enabled": true
    }
  ],
  "defaults": {
    "timeout_secs": 30
  }
}
```

### フック定義の各フィールド

| フィールド | 必須 | デフォルト | 説明 |
|------------|------|------------|------|
| `name` | ○ | - | フックの識別名（ログ出力に使用） |
| `event` | ○ | - | トリガーするイベント名 |
| `script` | ○ | - | 実行するスクリプトのパス（絶対パスまたは`~/`形式） |
| `timeout_secs` | - | 30 | タイムアウト秒数（1-300） |
| `enabled` | - | true | 有効/無効フラグ |

### 環境変数

スクリプト実行時に以下の環境変数が設定されます。

| 変数名 | 説明 | 例 |
|--------|------|-----|
| `POMODORO_EVENT` | イベント種別 | `work_end` |
| `POMODORO_PHASE` | 現在のフェーズ | `Working`, `ShortBreak` |
| `POMODORO_TASK_NAME` | タスク名（設定時のみ） | `ドキュメント作成` |
| `POMODORO_CYCLE` | 現在のサイクル番号 | `2` |
| `POMODORO_TOTAL_CYCLES` | 総サイクル数 | `4` |
| `POMODORO_DURATION_SECS` | セッション全体の秒数 | `1500` |
| `POMODORO_ELAPSED_SECS` | 経過秒数 | `1500` |
| `POMODORO_REMAINING_SECS` | 残り秒数 | `0` |
| `POMODORO_TIMESTAMP` | イベント発生時刻（ISO8601） | `2026-01-06T15:30:00Z` |
| `POMODORO_SESSION_ID` | セッションID（UUID） | `550e8400-e29b-41d4-...` |
| `POMODORO_HOOK_NAME` | 実行中のフック名 | `slack-notify` |

### 使用例：Slack通知

`~/.pomodoro/scripts/slack-notify.sh`:

```bash
#!/bin/bash

# Slack Webhook URL（環境変数で設定）
WEBHOOK_URL="${SLACK_WEBHOOK_URL}"

case "$POMODORO_EVENT" in
  work_end)
    MESSAGE=":tomato: ポモドーロ #${POMODORO_CYCLE} 完了！休憩しましょう。"
    ;;
  break_end)
    MESSAGE=":muscle: 休憩終了！次のポモドーロを始めましょう。"
    ;;
  *)
    exit 0
    ;;
esac

curl -s -X POST -H 'Content-type: application/json' \
  --data "{\"text\": \"${MESSAGE}\"}" \
  "$WEBHOOK_URL"
```

スクリプトに実行権限を付与:

```bash
chmod +x ~/.pomodoro/scripts/slack-notify.sh
```

### 注意事項

- フックは**非同期**で実行されます（タイマーをブロックしません）
- スクリプトは**絶対パス**または**`~/`で始まるパス**で指定してください
- 相対パスは使用できません（セキュリティ対策）
- 1イベントあたり最大**10個**のフックを登録可能
- タイムアウトは**1〜300秒**の範囲で指定

### サンプルスクリプト

`examples/hooks/` ディレクトリにサンプルスクリプトが用意されています:

| スクリプト | 説明 |
|-----------|------|
| `slack-notify.sh` | Slack Webhook通知 |
| `desktop-notify.sh` | macOSデスクトップ通知 |
| `record-stats.sh` | CSV統計記録 |
| `bgm-control.sh` | BGM自動制御（Spotify/Music） |

## 設定オプション

### サウンド設定
設定は `~/.pomodoro/sound-config.json` に保存されます。

```json
{
  "work_end_sound": "Funk",
  "break_end_sound": "Glass"
}
```
