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
State: Working 🍅
Task: ドキュメント作成
Time Remaining: 12:34
Cycle: 1/4
```

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

## 設定オプション

現在、設定ファイルによるカスタマイズ機能は開発中です。
将来のバージョンで `~/.config/pomodoro/config.toml` による設定が可能になる予定です。
