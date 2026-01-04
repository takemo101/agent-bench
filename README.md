# Pomodoro Timer CLI for macOS

Rust製の高機能なポモドーロタイマーCLIツールです。macOSにネイティブ統合され、メニューバー表示、通知、サウンド再生、フォーカスモード連携などの機能を提供します。

![Status](https://img.shields.io/badge/status-development-orange)
![Platform](https://img.shields.io/badge/platform-macOS-lightgrey)
![License](https://img.shields.io/badge/license-MIT-blue)

## 特徴

- 🍅 **ポモドーロテクニック**: 25分作業 + 5分休憩のサイクルを自動管理
- 🔔 **ネイティブ通知**: macOS通知センターを使用したアクション付き通知
- 🎵 **サウンド通知**: 作業・休憩完了時に通知音を再生（システムサウンド/埋め込み）
- 🖥 **メニューバー常駐**: 残り時間をメニューバーに表示し、クイック操作が可能
- 🧘 **フォーカスモード連携**: 作業開始時にmacOSの「集中モード」を自動でON/OFF
- 🚀 **自動起動**: LaunchAgentによるログイン時自動起動サポート
- ⌨️ **CLI操作**: 豊富なコマンドライン操作とシェル補完

## インストール

詳細なインストール手順は [INSTALL.md](INSTALL.md) を参照してください。

```bash
# ソースコードからビルド・インストール
cargo install --path .

# デーモンの自動起動設定
pomodoro install
```

## クイックスタート

```bash
# タイマーを開始（デフォルト: 25分作業）
pomodoro start --task "ドキュメント作成"

# ステータス確認
pomodoro status

# 一時停止
pomodoro pause

# 再開
pomodoro resume

# 停止
pomodoro stop
```

詳細な使用方法は [USAGE.md](USAGE.md) を参照してください。

## 必要要件

- macOS 11.0 (Big Sur) 以上
- Rust 1.71 以上 (ビルドする場合)

## ディレクトリ構造

- `src/cli`: CLIクライアント実装
- `src/daemon`: タイマーロジックとIPCサーバー
- `src/notification`: macOS通知制御
- `src/menubar`: メニューバーUI制御
- `src/sound`: サウンド再生
- `src/focus`: フォーカスモード連携
- `src/launchagent`: 自動起動管理

## ライセンス

MIT License
