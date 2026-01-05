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

## 設定オプション

### サウンド設定
設定は `~/.pomodoro/sound-config.json` に保存されます。

```json
{
  "work_end_sound": "Funk",
  "break_end_sound": "Glass"
}
```
