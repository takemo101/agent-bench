# 技術調査レポート: confy (v2.0.x)

| 項目 | 内容 |
|------|------|
| 調査日 | 2026-01-05 |
| 調査深度 | standard |
| 対象バージョン | v2.0.0 |
| 現行バージョン | 新規導入 (現状: `serde_json` による手動管理) |

## エグゼクティブサマリー

`confy` は、Rust製CLIアプリケーション向けに「設定の保存と読み込み」を極限まで簡略化するライブラリである。macOSの標準的な設定パス（`~/Library/Application Support/`）の自動解決をサポートしており、Phase 1 で求められている `~/.pomodoro/sound-config.json` のような要件を、ボイラープレートなしで実装可能にする。TOML（デフォルト）または YAML をサポートしているが、要件に合わせて JSON を使用することも可能（シリアライザの選択による）。

## バージョン情報

| バージョン | リリース日 | サポート状況 |
|-----------|-----------|-------------|
| v2.0.0 | 2025-10-24 | Active |
| v1.x | - | Maintenance |

## Breaking Changes (v2.0.0)

### 🚨 高影響

| 変更内容 | 影響範囲 | 移行方法 |
|---------|---------|---------|
| 設定ディレクトリのデフォルト変更 | 保存パス | macOSでは `Strategy::Native` を選択することで `~/Library/Application Support/` を使用。旧来の `~/.config` 形式も選択可能。 |

## 新機能・改善点

| 機能 | 概要 | 活用シーン |
|------|------|----------|
| ゼロ・ボイラープレート | `load`/`store` の2関数のみで完結 | 設定永続化の迅速な実装 |
| 型安全 | Serde による自動シリアライズ | 設定値のバリデーション |

## 設計への影響

### 必須対応

- [ ] 設定構造体への `Serialize`, `Deserialize`, `Default` トレイトの実装。
- [ ] `confy::load` による初期読み込みの実装。

### 推奨対応

- [ ] macOSネイティブな体験のため `confy::Strategy::Native` の採用を検討。

## マイグレーション手順（新規導入）

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default)]
struct SoundConfig {
    work_end_sound: String,
    break_end_sound: String,
}

// 読み込み
let cfg: SoundConfig = confy::load("pomodoro", None)?;
// 保存
confy::store("pomodoro", None, cfg)?;
```

## 参考リンク

- [confy 公式リポジトリ](https://github.com/rust-cli/confy)
- [docs.rs - confy](https://docs.rs/confy/latest/confy/)
