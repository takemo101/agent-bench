# インストールガイド

## 前提条件

- **OS**: macOS 11.0 (Big Sur) 以上
- **Rust**: 1.71 以上（ソースコードからビルドする場合）

## インストール方法

### 1. ソースコードからビルド・インストール

現在、バイナリ配布前の開発段階のため、Rustの `cargo` コマンドを使用してインストールします。

#### 通常のインストール

```bash
# リポジトリのクローン
git clone https://github.com/takemo101/agent-bench.git
cd agent-bench

# ビルドとインストール (~/.cargo/bin にインストール)
cargo install --path .
```

#### ユニバーサルバイナリのインストール（推奨）

Intel Mac と Apple Silicon Mac の両方に対応した最適化済みバイナリをインストールする場合は、`make` コマンドを使用します。

```bash
# 1. 依存関係（クロスコンパイル用ターゲット）のインストール
# ※ 初回のみ必要です
make install

# 2. ビルドとインストール (/usr/local/bin にインストール)
make install-local
```

> **Note**: `make install` を実行せずに `make install-local` を実行すると、`can't find crate for core` などのエラーが発生する場合があります。必ず最初に `make install` を実行してください。

インストールが完了すると、`pomodoro` コマンドが利用可能になります。

### 2. 自動起動設定（LaunchAgent）

ログイン時にポモドーロタイマーのデーモン（バックグラウンドプロセス）を自動起動するように設定します。

```bash
pomodoro install
```

設定が完了すると、次回ログイン時から自動的にデーモンが起動します。
すぐに起動したい場合は、以下のコマンドを実行するか、一度ログアウトして再ログインしてください。

```bash
# デーモンの手動起動（デバッグ用）
pomodoro daemon
```
※ 通常は `pomodoro install` 後はOSによって管理されるため、手動起動は不要です。

### 3. シェル補完の設定

コマンドの入力補完を有効にします。

**Bash:**
```bash
pomodoro completions bash > ~/.bash_completion
source ~/.bash_completion
```

**Zsh:**
```zsh
pomodoro completions zsh > /usr/local/share/zsh/site-functions/_pomodoro
# または
pomodoro completions zsh > ~/.zfunc/_pomodoro
# .zshrc に fpath+=~/.zfunc を追加
```

**Fish:**
```fish
pomodoro completions fish > ~/.config/fish/completions/pomodoro.fish
```

## アンインストール

### 1. 自動起動設定の解除

```bash
pomodoro uninstall
```

### 2. バイナリの削除

**cargo install でインストールした場合:**
```bash
cargo uninstall pomodoro
```

**make install-local でインストールした場合:**
```bash
make uninstall-local
```
